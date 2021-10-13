use blst::{blst_fr_add, blst_fr_mul};
use kzg::Fr;
use crate::fft_fr::fft_fr;
use crate::kzg_types::{create_fr_one, create_fr_zero, FFTSettings, negate_fr, Poly};
use crate::utils::is_power_of_two;

pub fn do_zero_poly_mul_partial(poly: &mut Poly, idxs: &[usize], stride: &usize, fft_settings: &FFTSettings) -> Result<(), String> {
    if idxs.len() == 0 {
        return Err(String::from("idx array must be non-zero"));
    } else if poly.coeffs.len() < idxs.len() + 1 {
        return Err(String::from("idx array must be non-zero"));
    }

    negate_fr(&mut poly.coeffs[0], &fft_settings.expanded_roots_of_unity[idxs[0] * stride]);

    for i in 1..idxs.len() {
        let mut neg_di: Fr = Fr::default();
        negate_fr(&mut neg_di, &fft_settings.expanded_roots_of_unity[idxs[i] * stride]);
        poly.coeffs[i] = neg_di.clone();
        unsafe {
            blst_fr_add(&mut poly.coeffs[i], &poly.coeffs[i], &poly.coeffs[i - 1]);
        }

        let mut j = i - 1;
        while j > 0 {
            unsafe {
                blst_fr_mul(&mut poly.coeffs[j], &poly.coeffs[j], &neg_di);
                blst_fr_add(&mut poly.coeffs[j], &poly.coeffs[j], &poly.coeffs[j - 1]);
            }
            j -= 1;
        }
        unsafe {
            blst_fr_mul(&mut poly.coeffs[0], &poly.coeffs[0], &neg_di);
        }
    }

    poly.coeffs[idxs.len()] = create_fr_one();
    for i in (idxs.len() + 1)..poly.coeffs.len() {
        poly.coeffs[i] = create_fr_zero();
    }

    return Ok(());
}

pub fn pad_poly(ret: &mut [Fr], poly: &Poly) -> Result<(), String> {
    if ret.len() < poly.coeffs.len() {
        return Err(String::from("Expected return array to be as lengthy as provided polynomial"));
    }

    for i in 0..poly.coeffs.len() {
        ret[i] = poly.coeffs[i].clone();
    }

    for i in poly.coeffs.len()..ret.len() {
        ret[i] = create_fr_zero();
    }

    return Ok(());
}

pub fn reduce_partials(ret: &mut Poly, scratch: &[Fr], partials: &[Poly], fft_settings: &FFTSettings) -> Result<(), String> {
    if !is_power_of_two(ret.coeffs.len()) {
        return Err(String::from("Expected poly needs to be a power of two length"));
    } else if scratch.len() < 3 * ret.coeffs.len() {
        return Err(String::from("Expected scratch length to be at least 3 times the polynomial buffer"));
    }

    let mut out_degree: usize = 0;
    for i in 0..partials.len() {
        out_degree += partials[i].coeffs.len() - 1;
    }

    if out_degree + 1 > ret.coeffs.len() {
        return Err(String::from("Out degree not expected to be more than return polynomial length"));
    }

    let mut p_padded = scratch[..ret.coeffs.len()].to_vec();
    let mut mul_eval_ps = scratch[ret.coeffs.len()..(ret.coeffs.len() * 2)].to_vec();
    let mut p_eval = scratch[(ret.coeffs.len() * 2)..].to_vec();

    pad_poly(&mut p_padded, &partials[partials.len() - 1])?;
    fft_fr(&mut mul_eval_ps, &p_padded, false, fft_settings)?;

    for i in 0..(partials.len() - 1) {
        pad_poly(&mut p_padded, &partials[i])?;
        fft_fr(&mut p_eval, &p_padded, false, fft_settings)?;
        for j in 0..ret.coeffs.len() {
            unsafe {
                blst_fr_mul(&mut mul_eval_ps[j], &mul_eval_ps[j], &p_eval[j]);
            }
        }
    }

    fft_fr(&mut ret.coeffs, &mul_eval_ps, true, fft_settings)?;

    return Ok(());
}


#[cfg(test)]
mod tests {
    use blst::blst_fr_from_uint64;
    use kzg::Fr;
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    use crate::fft_fr::fft_fr;
    use crate::kzg_types::{FFTSettings, fr_are_equal, fr_is_zero, Poly};
    use crate::zero_poly::{do_zero_poly_mul_partial, reduce_partials};

    #[test]
    fn test_reduce_partials() {
        let result = FFTSettings::from_scale(4);
        assert!(result.is_ok());
        let fft_settings = result.unwrap();

        let mut from_tree_reduction: Poly = Poly { coeffs: vec![Fr::default(); 16] };
        let mut from_direct: Poly = Poly { coeffs: vec![Fr::default(); 9] };

        let partial_idxs: [[usize; 2]; 4] = [[1, 3], [7, 8], [9, 10], [12, 13]];
        let mut poly_partials = [
            Poly { coeffs: vec![Fr::default(); 3] },
            Poly { coeffs: vec![Fr::default(); 3] },
            Poly { coeffs: vec![Fr::default(); 3] },
            Poly { coeffs: vec![Fr::default(); 3] },
        ];

        for i in 0..4 {
            let result = do_zero_poly_mul_partial(&mut poly_partials[i], &partial_idxs[i], &1, &fft_settings);
            assert!(result.is_ok());
        }

        let scratch = vec![Fr::default(); from_tree_reduction.coeffs.len() * 3];
        let result = reduce_partials(&mut from_tree_reduction, &scratch, &poly_partials, &fft_settings);
        assert!(result.is_ok());

        let idxs = [1, 3, 7, 8, 9, 10, 12, 13];
        let result = do_zero_poly_mul_partial(&mut from_direct, &idxs, &1, &fft_settings);
        assert!(result.is_ok());

        for i in 0..9 {
            assert!(fr_are_equal(&mut from_tree_reduction.coeffs[i], &from_direct.coeffs[i]));
        }
    }

    #[test]
    fn reduce_partials_random() {
        for scale in 5..13 {
            for ii in 1..8 {
                let missing_ratio = 0.1 * ii as f32;

                let result = FFTSettings::from_scale(scale);
                assert!(result.is_ok());
                let fft_settings = result.unwrap();

                let point_count = fft_settings.max_width;
                let missing_count = (point_count as f32 * missing_ratio) as usize;

                let mut missing = vec![0; point_count];
                for i in 0..point_count {
                    missing[i] = i;
                }
                missing.shuffle(&mut thread_rng());

                let missing_per_partial = 63;
                let partial_count = (missing_count + missing_per_partial - 1) / missing_per_partial;

                let mut idxs = vec![0usize; missing_per_partial];
                let mut partials = Vec::new();
                for i in 0..partial_count {
                    let start = i * missing_per_partial;
                    let mut end = start + missing_per_partial;

                    if end > missing_count {
                        end = missing_count;
                    }

                    let partial_size = end - start;
                    partials.push(Poly { coeffs: vec![Fr::default(); partial_size + 1] });

                    for j in 0..partial_size {
                        idxs[j] = missing[i * missing_per_partial + j];
                    }

                    let result = do_zero_poly_mul_partial(&mut partials[i], &idxs[..partial_size], &1, &fft_settings);
                    assert!(result.is_ok());
                }

                let mut from_tree_reduction: Poly = Poly { coeffs: vec![Fr::default(); point_count] };
                let mut scratch = vec![Fr::default(); point_count * 3];

                let result = reduce_partials(&mut from_tree_reduction, &scratch, &partials, &fft_settings);
                assert!(result.is_ok());

                let mut from_direct: Poly = Poly { coeffs: vec![Fr::default(); missing_count + 1] };

                let result = do_zero_poly_mul_partial(&mut from_direct, &missing[..missing_count], &(fft_settings.max_width / point_count), &fft_settings);
                assert!(result.is_ok());

                for i in 0..(missing_count + 1) {
                    assert!(fr_are_equal(&from_tree_reduction.coeffs[i], &from_direct.coeffs[i]));
                }
            }
        }
    }

    #[test]
    fn check_test_data() {
        let mut expected_eval = Poly { coeffs: vec![Fr::default(); 16] };
        let mut expected_poly = Poly { coeffs: vec![Fr::default(); 16] };
        let mut tmp_poly = Poly { coeffs: vec![Fr::default(); 16] };

        let result = FFTSettings::from_scale(4);
        assert!(result.is_ok());
        let fft_settings = result.unwrap();

        for i in 0..16 {
            unsafe {
                blst_fr_from_uint64(&mut expected_eval.coeffs[i], expected_eval.coeffs.as_ptr() as *const u64);
                blst_fr_from_uint64(&mut expected_poly.coeffs[i], expected_poly.coeffs.as_ptr() as *const u64);
            }
        }

        for i in 0..16 {
            let tmp = expected_poly.eval(&fft_settings.expanded_roots_of_unity[i]);
            assert!(fr_is_zero(&tmp));
        }

        for i in 0..8 {
            let tmp = expected_eval.eval(&fft_settings.expanded_roots_of_unity[i]);
            assert!(fr_is_zero(&tmp));
        }

        let result = fft_fr(&mut tmp_poly.coeffs, &expected_eval.coeffs, true, &fft_settings);
        for i in 0..16 {
            assert!(fr_are_equal(&tmp_poly.coeffs[i], &expected_poly.coeffs[i]));
        }
    }

    #[test]
    fn zero_poly_known() {}

    #[test]
    fn zero_poly_random() {}

    #[test]
    fn zero_poly_all_but_one() {}

    #[test]
    fn zero_poly_252() {}
}