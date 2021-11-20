use crate::data_types::{fr::Fr, g1::G1, g2::G2};
use crate::fk20_fft::{FFTSettings};
use crate::kzg10::Curve;
use crate::kzg_settings::KZGSettings;
use kzg::KZGSettings as CommonKZGSettings;
use crate::kzg10::Polynomial;


impl CommonKZGSettings<Fr, G1, G2, FFTSettings, Polynomial> for KZGSettings {
    fn default() -> Self {
        todo!()
    }

    fn new(secret_g1: &Vec<G1>, secret_g2: &Vec<G2>, length: usize, fs: &FFTSettings) -> Result<Self, String> {
       Ok(KZGSettings::new(secret_g1, secret_g2, length, fs))
    }


    fn commit_to_poly(&self, p: &Polynomial) -> Result<G1, String>{
        Ok(p.commit(&self.secret1))
    }

    fn compute_proof_single(&self, p: &Polynomial, x: &Fr) -> Result<G1, String>{
        Ok(p.gen_proof_at(&self.secret1, x))
    }

    fn check_proof_single(&self, com: &G1, proof: &G1, x: &Fr, value: &Fr) -> Result<bool, String>{
        let g1_gen = G1::gen();
        let g2_gen = G2::gen();
        let secret_minus_x = &self.secret2[1] - &(&g2_gen * x); // g2 * x to get x on g2
        let commitment_minus_y = com - &(&g1_gen * value);

        Ok(Curve::verify_pairing(&commitment_minus_y, &g2_gen, proof, &secret_minus_x))
    }

    fn compute_proof_multi(&self, p: &Polynomial, x: &Fr, n: usize) -> Result<G1, String>{
        todo!();
    }

    fn check_proof_multi(
        &self,
        com: &G1,
        proof: &G1,
        x: &Fr,
        values: &Vec<Fr>,
        n: usize,
    ) -> Result<bool, String>{
        todo!();
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> Fr{
        todo!();
    }
}