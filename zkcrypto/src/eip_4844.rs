use crate::zkfr::blsScalar;

pub fn bytes_to_bls_field(bytes: &[u8; 32usize]) -> blsScalar {
    blsScalar::from_bytes(bytes).unwrap()
}

pub fn bytes_from_bls_field(fr: &blsScalar) -> [u8; 32usize] {
    fr.to_bytes()
}

pub fn compute_powers(base: &blsScalar, num_powers: usize) -> Vec<blsScalar> {
    let mut powers: Vec<blsScalar> = vec![blsScalar::default(); num_powers];
    powers[0] = blsScalar::one();
    for i in 1..num_powers {
        powers[i] = powers[i - 1].mul(base);
    }
    powers
}

pub fn vector_lincomb(vectors: &[Vec<blsScalar>], scalars: &[blsScalar]) -> Vec<blsScalar> {
    let mut tmp: blsScalar;
    let mut out: Vec<blsScalar> = vec![blsScalar::zero(); vectors[0].len()];
    for (v, s) in vectors.iter().zip(scalars.iter()) {
        for (i, x) in v.iter().enumerate() {
            tmp = x.mul(s);
            out[i] = out[i].add(&tmp);
        }
    }
}