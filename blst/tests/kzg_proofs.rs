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

    use rust_kzg_blst::types::fft_settings::FsFFTSettings;
    use rust_kzg_blst::types::fp::FsFp;
    use rust_kzg_blst::types::fr::FsFr;
    use rust_kzg_blst::types::g1::{FsG1, FsG1Affine};
    use rust_kzg_blst::types::g2::FsG2;
    use rust_kzg_blst::types::kzg_settings::FsKZGSettings;
    use rust_kzg_blst::types::poly::FsPoly;
    use rust_kzg_blst::utils::generate_trusted_setup;

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    pub fn test_proof_single() {
        proof_single::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings, FsFp, FsG1Affine>(
            &generate_trusted_setup,
        );
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    pub fn test_commit_to_nil_poly() {
        commit_to_nil_poly::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
            FsFp,
            FsG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    pub fn test_commit_to_too_long_poly() {
        commit_to_too_long_poly_returns_err::<
            FsFr,
            FsG1,
            FsG2,
            FsPoly,
            FsFFTSettings,
            FsKZGSettings,
            FsFp,
            FsG1Affine,
        >(&generate_trusted_setup);
    }

    #[ignore = "KZG settings loading doesn't support trusted setup sizes other than FIELD_ELEMENTS_PER_BLOB (4096 points)"]
    #[test]
    pub fn test_proof_multi() {
        proof_multi::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings, FsFp, FsG1Affine>(
            &generate_trusted_setup,
        );
    }

    // This aims at showing that the use of the blst::Pairing engine in pairings_verify
    // has the desired semantics.
    #[cfg(feature = "rand")]
    fn og_pairings_verify() {
        let a1 = FsG1::rand();
        let a2 = FsG2::rand();
        let b1 = FsG1::rand();
        let b2 = FsG2::rand();

        let mut loop0 = blst_fp12::default();
        let mut loop1 = blst_fp12::default();
        let mut gt_point = blst_fp12::default();

        let mut aa1 = blst_p1_affine::default();
        let mut bb1 = blst_p1_affine::default();

        let mut aa2 = blst_p2_affine::default();
        let mut bb2 = blst_p2_affine::default();

        // As an optimisation, we want to invert one of the pairings,
        // so we negate one of the points.
        let mut a1neg: FsG1 = a1;
        unsafe {
            blst_p1_cneg(&mut a1neg.0, true);
            blst_p1_to_affine(&mut aa1, &a1neg.0);

            blst_p1_to_affine(&mut bb1, &b1.0);
            blst_p2_to_affine(&mut aa2, &a2.0);
            blst_p2_to_affine(&mut bb2, &b2.0);

            blst_miller_loop(&mut loop0, &aa2, &aa1);
            blst_miller_loop(&mut loop1, &bb2, &bb1);

            blst_fp12_mul(&mut gt_point, &loop0, &loop1);
            blst_final_exp(&mut gt_point, &gt_point);

            let dst = [0u8; 3];
            let mut pairing_blst = Pairing::new(false, &dst);
            pairing_blst.raw_aggregate(&aa2, &aa1);
            pairing_blst.raw_aggregate(&bb2, &bb1);

            assert_eq!(gt_point, pairing_blst.as_fp12().final_exp());
        }
    }

    #[cfg(feature = "rand")]
    #[test]
    pub fn test_pairings_verify() {
        for _i in 0..100 {
            og_pairings_verify();
        }
    }
}
