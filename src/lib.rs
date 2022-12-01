extern crate rand;
#[macro_use]
extern crate ff;
use ff::*;

#[derive(PrimeField)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "7"]
#[PrimeFieldReprEndianness = "little"]
pub struct Fr([u64; 4]);

extern crate num;
extern crate num_bigint;
use num_bigint::{BigInt, Sign};
use tiny_keccak::Keccak;

const SEED: &str = "mimc";

pub struct Constants {
    n_rounds: usize,
    cts: Vec<Fr>,
}

pub fn generate_constants(n_rounds: usize) -> Constants {
    let cts = get_constants(SEED, n_rounds);

    Constants {
        n_rounds: n_rounds,
        cts: cts,
    }
}

pub fn get_constants(seed: &str, n_rounds: usize) -> Vec<Fr> {
    let mut cts: Vec<Fr> = Vec::new();
    cts.push(Fr::ZERO);

    let mut keccak = Keccak::new_keccak256();
    let mut h = [0u8; 32];
    keccak.update(seed.as_bytes());
    keccak.finalize(&mut h);

    let r: BigInt = BigInt::parse_bytes(
        b"21888242871839275222246405745257275088548364400416034343698204186575808495617",
        10,
    )
    .unwrap();

    let mut c = BigInt::from_bytes_be(Sign::Plus, &h);
    for _ in 1..n_rounds {
        let (_, c_bytes) = c.to_bytes_be();
        let mut c_bytes32: [u8; 32] = [0; 32];
        let diff = c_bytes32.len() - c_bytes.len();
        c_bytes32[diff..].copy_from_slice(&c_bytes[..]);

        let mut keccak = Keccak::new_keccak256();
        let mut h = [0u8; 32];
        keccak.update(&c_bytes[..]);
        keccak.finalize(&mut h);
        c = BigInt::from_bytes_be(Sign::Plus, &h);

        let n = modulus(&c, &r);
        cts.push(Fr::from_str_vartime(&n.to_string()).unwrap());
    }
    cts
}

pub fn modulus(a: &BigInt, m: &BigInt) -> BigInt {
    ((a % m) + m) % m
}

pub struct Mimc7 {
    constants: Constants,
}

impl Mimc7 {
    pub fn new(n_rounds: usize) -> Mimc7 {
        Mimc7 {
            constants: generate_constants(n_rounds),
        }
    }

    pub fn hash(&self, x_in: &Fr, k: &Fr) -> Fr {
        let mut h: Fr = Fr::ZERO;
        for i in 0..self.constants.n_rounds {
            let mut t: Fr;
            if i == 0 {
                t = x_in.clone();
                t += k;
            } else {
                t = h.clone();
                t += k;
                t += &self.constants.cts[i];
            }
            let mut t2 = t.clone();
            t2 = t2.square();
            let mut t7 = t2.clone();
            t7 = t7.square();
            t7 *= t2;
            t7 *= t;
            h = t7.clone();
        }
        h += k;
        h
    }

    pub fn multi_hash(&self, arr: Vec<Fr>, key: &Fr) -> Fr {
        let mut r = key.clone();
        for i in 0..arr.len() {
            let h = self.hash(&arr[i], &r);
            r += &arr[i];
            r += h;
        }
        r
    }
}
