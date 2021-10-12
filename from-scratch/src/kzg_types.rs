use kzg::{Fr, G1, G2};

pub struct Poly {
    pub coeffs: Vec<Fr>,
}

pub struct FFTSettings {
    pub max_width: u64,
    pub root_of_unity: Fr,
    pub expanded_roots_of_unity: Vec<Fr>,
    pub reverse_roots_of_unity: Vec<Fr>,
}

pub struct KZGSettings {
    pub fs: FFTSettings,
    // Both secret_g1 and secret_g2 have the same number of elements
    pub secret_g1: Vec<G1>,
    pub secret_g2: Vec<G2>,
}