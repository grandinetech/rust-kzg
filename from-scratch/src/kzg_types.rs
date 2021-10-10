pub enum STATUS {
    SUCCESS,
    BAD_ARGS,
    ERROR
}

pub struct Poly {
    pub coeffs: Fr,
    pub length: u64,
}

pub struct FFTSettings {
    max_width: u64,
    root_of_unity: Fr,
    expanded_roots_of_unity: Fr,
    reverse_roots_of_unity: Fr
}

pub struct KZGSettings {
    pub fs: FFTSettings,
    pub secret_g1: P1,
    pub secret_g2: P1,
    pub length: u64
}