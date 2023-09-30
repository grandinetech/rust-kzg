use kzg::{FFTFr, FFTSettings, Fr, Poly, ZeroPoly};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{thread_rng, RngCore, SeedableRng};

#[rustfmt::skip]
const EXISTS: [bool; 16] = [
    true, false, false, true,
    false, true, true, false,
    false, false, true, true,
    false, true, false, true,
];

#[rustfmt::skip]
const EXPECTED_EVAL_U64: [[u64; 4]; 16] = [
    [0xfd5a5130b97ce0c3, 0xb4748a4cb0f90e6d, 0x12a1ab34b25b18c1, 0x5a5ac0c81c9f7ea8],
    [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000],
    [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000],
    [0xaa385cbce3dd1657, 0x2fdab57a38bdb514, 0x20e022e205dafa53, 0x14077dd3f5d996b1],
    [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000],
    [0x194018614b6f7276, 0xdf2b18f870532376, 0x1ff427cd5b583fe6, 0x014d6444ff03dd09],
    [0xcc84c2de684c0dde, 0xf1e7ab32aa830d02, 0x967bf35a2a691f20, 0x046109731cdf0d3c],
    [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000],
    [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000],
    [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000],
    [0x96cddd2924212afb, 0xeaa4c1f51421d8d8, 0x3ae969cfa34d0ed1, 0x6b6c5e876bc3916d],
    [0x449310802f74ad49, 0x47c940979163037a, 0x10d311564afb9b2a, 0x269b8531c369bafb],
    [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000],
    [0xd9af75fe35c16cf1, 0x068bb140cea92f75, 0xe769811965e10a47, 0x48ed97e6745612f2],
    [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000],
    [0x7ef1f59bb1677685, 0x33a637296680e8ce, 0xaaf62b3f6e016709, 0x454a299178a4dba9]];

#[rustfmt::skip]
const EXPECTED_POLY_U64: [[u64; 4]; 16] = [
    [0xac159e2688bd4333, 0x3bfef0f00df2ec88, 0x561dcd0fd4d314d9, 0x533bd8c1e977024e],
    [0x18bc6eedc010ef8d, 0xc731a3eb4ea2ab70, 0x5c2589357ae121a8, 0x04f9108d308f7016],
    [0x232759f49556ac08, 0x9776fe2e9f4c613c, 0x74d5bed4eb2de960, 0x1f6cf6719bfa0e68],
    [0xf2f3461e8ab1ae34, 0xeb220fcc11ef1c80, 0x7a4637d3a637739b, 0x19901a58cd177c53],
    [0x9340f62465a1f4fe, 0xd9cb3ea6de494a11, 0xee92ebc763cdff5d, 0x5443e89811b5b9f5],
    [0x269a255e2e4e48a4, 0xfadae7a89d9b2f2b, 0xb5515799b41e1a88, 0x2e990979a0ffcee5],
    [0x1c2f3a5759088c29, 0x2a958d654cf1795f, 0x9ca121fa43d152d1, 0x1425239535953093],
    [0x4c634e2d63ad89fd, 0xd6ea7bc7da4ebe1a, 0x9730a8fb88c7c895, 0x1a01ffae0477c2a8],
    [0x0000000000000001, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000],
    [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000],
    [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000],
    [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000],
    [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000],
    [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000],
    [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000],
    [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000]];

/// First, create polynomials that evaluate to zero at given roots.
/// Check that multiplying them together (via reduce_partials) equals
/// constructing a polynomial that evaluates to zero at all given roots.
pub fn test_reduce_partials<
    TFr: Fr,
    TFFTSettings: FFTSettings<TFr> + ZeroPoly<TFr, TPoly>,
    TPoly: Poly<TFr>,
>() {
    let fft_settings = TFFTSettings::new(4).unwrap();

    let partial_idxs: [[usize; 2]; 4] = [[1, 3], [7, 8], [9, 10], [12, 13]];
    let mut poly_partials = Vec::new();

    for partial_idxs in partial_idxs.iter() {
        let temp = fft_settings
            .do_zero_poly_mul_partial(partial_idxs, 1)
            .unwrap();
        poly_partials.push(temp);
    }

    let from_tree_reduction = fft_settings.reduce_partials(16, &poly_partials).unwrap();

    let idxs = [1, 3, 7, 8, 9, 10, 12, 13];
    let from_direct = fft_settings.do_zero_poly_mul_partial(&idxs, 1).unwrap();

    for i in 0..9 {
        from_tree_reduction
            .get_coeff_at(i)
            .equals(&from_direct.get_coeff_at(i));
    }
}

/// Create random partial polynomials that equal 0 at given roots.
/// Check that multiplying them together (via reduce_partials) equals
/// constructing a polynomial that evaluates to zero at all given roots.
pub fn reduce_partials_random<
    TFr: Fr,
    TFFTSettings: FFTSettings<TFr> + ZeroPoly<TFr, TPoly>,
    TPoly: Poly<TFr>,
>() {
    for scale in 5..13 {
        for ii in 1..8 {
            let missing_ratio = 0.1 * ii as f32;

            let fft_settings = TFFTSettings::new(scale).unwrap();

            let point_count = fft_settings.get_max_width();
            let missing_count = (point_count as f32 * missing_ratio) as usize;

            let mut missing = (0..point_count).collect::<Vec<_>>();
            missing.shuffle(&mut thread_rng());

            let missing_per_partial = 63;
            let partial_count = (missing_count + missing_per_partial - 1) / missing_per_partial;

            let mut idxs = vec![0usize; missing_per_partial];
            let mut partials = Vec::new();
            for i in 0..partial_count {
                let start = i * missing_per_partial;
                let end = if start + missing_per_partial > missing_count {
                    missing_count
                } else {
                    start + missing_per_partial
                };

                let partial_size = end - start;
                for j in 0..partial_size {
                    idxs[j] = missing[i * missing_per_partial + j];
                }

                let partial = fft_settings
                    .do_zero_poly_mul_partial(&idxs[..partial_size], 1)
                    .unwrap();
                partials.push(partial);
            }

            let from_tree_reduction = fft_settings
                .reduce_partials(point_count, &partials)
                .unwrap();
            let from_direct = fft_settings
                .do_zero_poly_mul_partial(
                    &missing[..missing_count],
                    fft_settings.get_max_width() / point_count,
                )
                .unwrap();

            for i in 0..(missing_count + 1) {
                from_tree_reduction
                    .get_coeff_at(i)
                    .equals(&from_direct.get_coeff_at(i));
            }
        }
    }
}

/// Check that polynomial evaluation works against precomputed values
pub fn check_test_data<TFr: Fr, TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>, TPoly: Poly<TFr>>() {
    let mut expected_eval = TPoly::new(16);
    let mut expected_poly = TPoly::new(16);

    let fft_settings = TFFTSettings::new(4).unwrap();

    for i in 0..16 {
        expected_eval.set_coeff_at(i, &TFr::from_u64_arr(&EXPECTED_EVAL_U64[i]));
        expected_poly.set_coeff_at(i, &TFr::from_u64_arr(&EXPECTED_POLY_U64[i]));
    }

    for (i, &exists) in EXISTS.iter().enumerate() {
        if !exists {
            let tmp = expected_poly.eval(&fft_settings.get_expanded_roots_of_unity_at(i));
            assert!(tmp.is_zero());
        }
    }

    for i in 1..8 {
        let tmp = expected_eval.eval(&fft_settings.get_expanded_roots_of_unity_at(i));
        assert!(tmp.is_zero());
    }

    let tmp_poly = fft_settings
        .fft_fr(expected_eval.get_coeffs(), true)
        .unwrap();

    assert_eq!(tmp_poly.len(), 16);
    for (i, tmp_poly) in tmp_poly.iter().enumerate() {
        assert!(tmp_poly.equals(&expected_poly.get_coeff_at(i)));
    }
}

/// Check if zero polynomial is calculated and evaluated as expected against precomputed values
pub fn zero_poly_known<
    TFr: Fr,
    TFFTSettings: FFTSettings<TFr> + ZeroPoly<TFr, TPoly>,
    TPoly: Poly<TFr>,
>() {
    let fft_settings = TFFTSettings::new(4).unwrap();

    let mut missing_idxs = Vec::new();
    let mut expected_eval = TPoly::new(16);
    let mut expected_poly = TPoly::new(16);

    for (i, &exists) in EXISTS.iter().enumerate() {
        expected_eval.set_coeff_at(i, &TFr::from_u64_arr(&EXPECTED_EVAL_U64[i]));
        expected_poly.set_coeff_at(i, &TFr::from_u64_arr(&EXPECTED_POLY_U64[i]));

        if !exists {
            missing_idxs.push(i);
        }
    }

    let (zero_eval, zero_poly) = fft_settings
        .zero_poly_via_multiplication(16, &missing_idxs)
        .unwrap();

    assert_eq!(zero_eval.len(), 16);
    for (i, zero_eval) in zero_eval.iter().enumerate() {
        assert!(expected_eval.get_coeff_at(i).equals(zero_eval));
        assert!(expected_poly
            .get_coeff_at(i)
            .equals(&zero_poly.get_coeff_at(i)));
    }
}

/// Generate random series of missing indices and check if they are multiplied correctly
pub fn zero_poly_random<
    TFr: Fr,
    TFFTSettings: FFTSettings<TFr> + ZeroPoly<TFr, TPoly> + FFTFr<TFr>,
    TPoly: Poly<TFr>,
>() {
    for its in 0..8 {
        let mut rng = StdRng::seed_from_u64(its);
        for scale in 3..13 {
            let fft_settings = TFFTSettings::new(scale).unwrap();

            let mut missing_idxs = Vec::new();
            for i in 0..fft_settings.get_max_width() {
                if rng.next_u64() % 2 == 1 {
                    missing_idxs.push(i);
                }
            }

            if missing_idxs.len() == fft_settings.get_max_width() {
                continue;
            }

            let (zero_eval, zero_poly) = fft_settings
                .zero_poly_via_multiplication(fft_settings.get_max_width(), &missing_idxs)
                .unwrap();

            for missing_idx in missing_idxs {
                let out = zero_poly.eval(&fft_settings.get_expanded_roots_of_unity_at(missing_idx));
                assert!(out.is_zero());
            }

            let zero_eval_fft = fft_settings.fft_fr(&zero_eval, true).unwrap();

            #[allow(clippy::needless_range_loop)]
            for i in 0..zero_poly.len() {
                assert!(zero_poly.get_coeff_at(i).equals(&zero_eval_fft[i]));
            }

            #[allow(clippy::needless_range_loop)]
            for i in zero_poly.len()..fft_settings.get_max_width() {
                assert!(zero_eval_fft[i].is_zero());
            }
        }
    }
}

/// Check that roots of unity loop correctly and the first value equals the last one
/// by evaluating a polynomial created to equal zero for the last value
pub fn zero_poly_all_but_one<
    TFr: Fr,
    TFFTSettings: FFTSettings<TFr> + ZeroPoly<TFr, TPoly> + FFTFr<TFr>,
    TPoly: Poly<TFr>,
>() {
    let fft_settings = TFFTSettings::new(8).unwrap();

    let mut missing_idxs = Vec::new();
    for i in 0..(fft_settings.get_max_width() - 1) {
        missing_idxs.push(i + 1);
    }

    let (zero_eval, zero_poly) = fft_settings
        .zero_poly_via_multiplication(fft_settings.get_max_width(), &missing_idxs)
        .unwrap();

    for missing_idx in missing_idxs {
        let ret = zero_poly.eval(&fft_settings.get_expanded_roots_of_unity_at(missing_idx));
        assert!(ret.is_zero());
    }

    let zero_eval_fft = fft_settings.fft_fr(&zero_eval, true).unwrap();

    #[allow(clippy::needless_range_loop)]
    for i in 0..zero_poly.len() {
        assert!(zero_poly.get_coeff_at(i).equals(&zero_eval_fft[i]));
    }

    #[allow(clippy::needless_range_loop)]
    for i in zero_poly.len()..fft_settings.get_max_width() {
        assert!(zero_eval_fft[i].is_zero());
    }
}

/// Check an edge case where 252 is missing with width 8
pub fn zero_poly_252<
    TFr: Fr,
    TFFTSettings: FFTSettings<TFr> + ZeroPoly<TFr, TPoly> + FFTFr<TFr>,
    TPoly: Poly<TFr>,
>() {
    let fft_settings = TFFTSettings::new(8).unwrap();

    let missing_idxs = (0..252).collect::<Vec<_>>();

    let (zero_eval, zero_poly) = fft_settings
        .zero_poly_via_multiplication(fft_settings.get_max_width(), &missing_idxs)
        .unwrap();

    for missing_idx in missing_idxs {
        let ret = zero_poly.eval(&fft_settings.get_expanded_roots_of_unity_at(missing_idx));
        assert!(ret.is_zero());
    }

    let zero_eval_fft = fft_settings.fft_fr(&zero_eval, true).unwrap();

    #[allow(clippy::needless_range_loop)]
    for i in 0..zero_poly.len() {
        assert!(zero_poly.get_coeff_at(i).equals(&zero_eval_fft[i]));
    }

    #[allow(clippy::needless_range_loop)]
    for i in zero_poly.len()..fft_settings.get_max_width() {
        assert!(zero_eval_fft[i].is_zero());
    }
}
