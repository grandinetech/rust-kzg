use crate::data_types::{fr::Fr, g1::G1, g2::G2};
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
    pub fn generate_trusted_setup(n: usize, secret: [u8; 32usize]) -> (Vec<G1>, Vec<G2>) {
        let g1_gen = G1::gen();
        let g2_gen = G2::gen(); 

        let mut g1_points = vec!(G1::default(); n);
        let mut g2_points = vec!(G2::default(); n);
        let secretfr = Fr::from_scalar(&secret);
        let mut secret_to_power = Fr::one();
        for i in 0..n {
            G1::mul(&mut (g1_points[i]), &g1_gen, &secret_to_power);
            G2::mul(&mut (g2_points[i]), &g2_gen, &secret_to_power);

            secret_to_power *= &secretfr;
        }

        (g1_points, g2_points)
    }
}


