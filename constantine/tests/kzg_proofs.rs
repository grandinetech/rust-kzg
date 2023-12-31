#[cfg(test)]
mod tests {
    use blst::{
        blst_final_exp, blst_fp12, blst_fp12_mul, blst_miller_loop, blst_p1_affine, blst_p1_cneg,
        blst_p1_to_affine, blst_p2_affine, blst_p2_to_affine, Pairing,
    };
    use kzg::G1;
    use kzg_bench::tests::kzg_proofs::{
        commit_to_nil_poly, commit_to_too_long_poly_returns_err, proof_multi, proof_single,
    };

    use rust_kzg_constantine::types::fft_settings::CtFFTSettings;
    use rust_kzg_constantine::types::fp::CtFp;
    use rust_kzg_constantine::types::fr::CtFr;
    use rust_kzg_constantine::types::g1::{CtG1, CtG1Affine};
    use rust_kzg_constantine::types::g2::CtG2;
    use rust_kzg_constantine::types::kzg_settings::CtKZGSettings;
    use rust_kzg_constantine::types::poly::CtPoly;
    use rust_kzg_constantine::utils::generate_trusted_setup;

    #[test]
    pub fn test_proof_single() {
        proof_single::<CtFr, CtG1, CtG2, CtPoly, CtFFTSettings, CtKZGSettings, CtFp, CtG1Affine>(
            &generate_trusted_setup,
        );
    }

    #[test]
    pub fn test_commit_to_nil_poly() {
        commit_to_nil_poly::<CtFr, CtG1, CtG2, CtPoly, CtFFTSettings, CtKZGSettings, CtFp, CtG1Affine>(
            &generate_trusted_setup,
        );
    }

    #[test]
    pub fn test_commit_to_too_long_poly() {
        commit_to_too_long_poly_returns_err::<CtFr, CtG1, CtG2, CtPoly, CtFFTSettings, CtKZGSettings, CtFp, CtG1Affine>(
            &generate_trusted_setup,
        );
    }

    #[test]
    pub fn test_proof_multi() {
        proof_multi::<CtFr, CtG1, CtG2, CtPoly, CtFFTSettings, CtKZGSettings, CtFp, CtG1Affine>(
            &generate_trusted_setup,
        );
    }
}
