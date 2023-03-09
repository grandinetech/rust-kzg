use crate::data_types::{fr::Fr, g1::G1, g2::G2};
use crate::fk20_fft::FFTSettings;
use crate::kzg10::Polynomial;
use crate::kzg_settings::KZGSettings;
use kzg::KZGSettings as CommonKZGSettings;

impl CommonKZGSettings<Fr, G1, G2, FFTSettings, Polynomial> for KZGSettings {
    fn new(
        secret_g1: &[G1],
        secret_g2: &[G2],
        length: usize,
        fs: &FFTSettings,
    ) -> Result<Self, String> {
        KZGSettings::new(secret_g1, secret_g2, length, fs)
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

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> Fr {
        self.fft_settings.exp_roots_of_unity[i]
    }
}
