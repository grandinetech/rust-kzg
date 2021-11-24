use crate::data_types::{fr::Fr, g1::G1, g2::G2};
use crate::fk20_fft::FFTSettings;
use crate::fk20_matrix::{FK20Matrix, FK20SingleMatrix};
use crate::kzg10::Polynomial;
use crate::kzg_settings::KZGSettings;
use kzg::{FK20MultiSettings, FK20SingleSettings};

impl FK20SingleSettings<Fr, G1, G2, FFTSettings, Polynomial, KZGSettings> for FK20SingleMatrix {
    fn default() -> Self {
        FK20SingleMatrix::default()
    }

    fn new(ks: &KZGSettings, n2: usize) -> Result<Self, String> {
        FK20SingleMatrix::new(ks, n2)
    }

    fn data_availability(&self, p: &Polynomial) -> Result<Vec<G1>, String> {
        self.dau_using_fk20_single(p)
    }

    fn data_availability_optimized(&self, p: &Polynomial) -> Result<Vec<G1>, String> {
        self.fk20_single_dao_optimized(p)
    }
}

impl FK20MultiSettings<Fr, G1, G2, FFTSettings, Polynomial, KZGSettings> for FK20Matrix {
    fn default() -> Self {
        FK20Matrix::default()
    }

    fn new(ks: &KZGSettings, n2: usize, chunk_len: usize) -> Result<Self, String> {
        FK20Matrix::new(ks, n2, chunk_len)
    }

    fn data_availability(&self, p: &Polynomial) -> Result<Vec<G1>, String> {
        self.dau_using_fk20_multi(p)
    }

    fn data_availability_optimized(&self, p: &Polynomial) -> Result<Vec<G1>, String> {
        self.fk20_multi_dao_optimized(p)
    }
}
