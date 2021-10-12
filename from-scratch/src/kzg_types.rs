use kzg::{Fr, G1, G2};

pub struct Poly {
    pub coeffs: Vec<Fr>,
}

pub struct FFTSettings {
    max_width: u64,
    root_of_unity: Fr,
    expanded_roots_of_unity: Vec<Fr>,
    reverse_roots_of_unity: Vec<Fr>,
}

pub struct KZGSettings {
    fs: FFTSettings,
    // Both secret_g1 and secret_g2 have the same number of elements
    secret_g1: Vec<G1>,
    secret_g2: Vec<G2>,
}