use crate::data_types::{fr::Fr, g1::G1, g2::G2};
use crate::fk20_fft::FFTSettings;
use crate::kzg10::Polynomial;
use crate::kzg_settings::KZGSettings;
use crate::utilities::is_power_of_2;
use kzg::KZGSettings as CommonKZGSettings;

impl CommonKZGSettings<Fr, G1, G2, FFTSettings, Polynomial> for KZGSettings {
    fn default() -> Self {
        KZGSettings::default()
    }

    fn new(
        secret_g1: &Vec<G1>,
        secret_g2: &Vec<G2>,
        length: usize,
        fs: &FFTSettings,
    ) -> Result<Self, String> {
        Ok(KZGSettings::new(secret_g1, secret_g2, length, fs))
    }

    fn commit_to_poly(&self, p: &Polynomial) -> Result<G1, String> {
        Ok(p.commit(&self.curve.g1_points))
    }

    fn compute_proof_single(&self, p: &Polynomial, x: &Fr) -> Result<G1, String> {
        Ok(p.gen_proof_at(&self.curve.g1_points, x))
    }

    fn check_proof_single(&self, com: &G1, proof: &G1, x: &Fr, value: &Fr) -> Result<bool, String> {
        Ok(KZGSettings::check_proof_single(self, com, proof, x, value))
    }

    fn compute_proof_multi(&self, p: &Polynomial, x: &Fr, n: usize) -> Result<G1, String> {
        Ok(KZGSettings::compute_proof_multi(self, p, x, n))
    }

    fn check_proof_multi(
        &self,
        com: &G1,
        proof: &G1,
        x: &Fr,
        values: &Vec<Fr>,
        n: usize,
    ) -> Result<bool, String> {
        if !is_power_of_2(n) {
            return Err(String::from("Provided size is not power of 2!"));
        }

        Ok(KZGSettings::check_proof_multi(
            self, com, proof, x, values, n,
        ))
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> Fr {
        self.fft_settings.exp_roots_of_unity[i]
    }
}
