use crate::data_types::{fr::Fr, g1::G1, g2::G2};
use crate::fk20_fft::FFTSettings;
use crate::kzg10::Polynomial;
use crate::kzg_settings::KZGSettings;
use kzg::KZGSettings as CommonKZGSettings;

impl CommonKZGSettings<Fr, G1, G1, G2, FFTSettings, Polynomial> for KZGSettings {
    fn new(
        secret_g1: &[G1],
        secret_g1_lagrange: &[G1],
        secret_g2: &[G2],
        length: usize,
        fs: &FFTSettings,
    ) -> Result<Self, String> {
        KZGSettings::new(secret_g1, secret_g1_lagrange, secret_g2,  length, fs)
    }

    fn commit_to_poly(&self, polynomial: &Polynomial) -> Result<G1, String> {
        polynomial.commit(&self.curve.g1_points)
    }

    fn compute_proof_single(&self, polynomial: &Polynomial, x: &Fr) -> Result<G1, String> {
        polynomial.gen_proof_at(&self.curve.g1_points, x)
    }

    fn check_proof_single(&self, com: &G1, proof: &G1, x: &Fr, value: &Fr) -> Result<bool, String> {
        Ok(KZGSettings::check_proof_single(self, com, proof, x, value))
    }

    fn compute_proof_multi(&self, p: &Polynomial, x: &Fr, n: usize) -> Result<G1, String> {
        KZGSettings::compute_proof_multi(self, p, x, n)
    }

    fn check_proof_multi(
        &self,
        com: &G1,
        proof: &G1,
        x: &Fr,
        values: &[Fr],
        n: usize,
    ) -> Result<bool, String> {
        KZGSettings::check_proof_multi(self, com, proof, x, values, n)
    }

    // fn get_expanded_roots_of_unity_at(&self, i: usize) -> Fr {
    //     self.fft_settings.expanded_roots_of_unity[i]
    // }

    fn get_roots_of_unity_at(&self, i: usize) -> Fr {
        self.fft_settings.roots_of_unity[i]
    }

    fn get_fft_settings(&self) -> &FFTSettings {
        &self.fs
    }
    
    fn get_g1_lagrange_brp(&self) -> &[G1] {
        &self.g1_values_lagrange_brp
    }

    fn get_g1_monomial(&self) -> &[G1] {
        &self.g1_values_monomial
    }

    fn get_g2_monomial(&self) -> &[G2] {
        &self.g2_values_monomial
    }

    fn get_precomputation(&self) -> Option<&PrecomputationTable<Fr, G1, Fp, G1Affine>> {
        self.precomputation.as_ref().map(|v| v.as_ref())
    }

    fn get_x_ext_fft_column(&self, index: usize) -> &[G1] {
        &self.x_ext_fft_columns[index]
    }
}
