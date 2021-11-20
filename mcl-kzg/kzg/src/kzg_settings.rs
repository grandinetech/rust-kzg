use crate::data_types::{g1::G1, g2::G2};
use crate::fk20_fft::FFTSettings;

#[derive(Clone)]
pub struct KZGSettings {
    pub fs: FFTSettings,
    pub secret1: Vec<G1>,
    pub secret2: Vec<G2>
}

impl KZGSettings {
    pub fn new(secret_g1: &Vec<G1>, secret_g2: &Vec<G2>, length: usize, fs: &FFTSettings) -> Self {
        let mut sec1: Vec<G1> = vec![];
        let mut sec2: Vec<G2> = vec![];
        for i in 0..length {
            sec1.push(secret_g1[i].clone());
            sec2.push(secret_g2[i].clone());
        }
        
        KZGSettings{
            fs: fs.clone(),
            secret1: sec1,
            secret2: sec2
        }
    }
}


