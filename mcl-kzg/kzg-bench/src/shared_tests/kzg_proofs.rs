#[cfg(test)]
pub mod kzg_proofs_tests {
    use kzg_bench::tests::kzg_proofs::*;
    use mcl_rust::data_types::{fr::Fr, g1::G1, g2::G2};
    use mcl_rust::fk20_fft::{FFTSettings};
    use mcl_rust::mcl_methods::init;
    use mcl_rust::CurveType;
    use mcl_rust::kzg_settings::KZGSettings;
    use mcl_rust::kzg10::Polynomial;


    #[test]
    fn proof_single_() {
        assert!(init(CurveType::BLS12_381));
        proof_single::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings>(&generate_trusted_setup);
    }

    pub fn generate_trusted_setup(n: usize, secret: [u8; 32usize]) -> (Vec<G1>, Vec<G2>) {
        let g1_gen = G1::gen();
        let g2_gen = G2::gen(); 

        let mut g1_points = vec!(G1::default(); n);
        let mut g2_points = vec!(G2::default(); n);
        let secret = Fr::from_str("1927409816240961209460912649124", 10).unwrap();
        let mut secret_to_power = Fr::one();
        for i in 0..n {
            G1::mul(&mut (g1_points[i]), &g1_gen, &secret_to_power);
            G2::mul(&mut (g2_points[i]), &g2_gen, &secret_to_power);

            secret_to_power *= &secret;
        }

        (g1_points, g2_points)
    }

}
