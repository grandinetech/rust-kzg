
use kzg::{G1, Fr, FFTFr, Poly};
use blst::{blst_p1_add_or_double,
            blst_p1s_to_affine,
            blst_scalar,
            blst_scalar_from_fr,
            blst_p1_mult,
            blst_p1s_mult_pippenger,
            blst_p1s_mult_pippenger_scratch_sizeof,
            blst_p2_mult,
            blst_p1_cneg,
            blst_p2_cneg,
            blst_p2_add_or_double,
            blst_fp12,
            blst_p1_to_affine,
            blst_p2_to_affine,
            blst_miller_loop,
            blst_fp12_mul,
            blst_final_exp,
            blst_fp12_is_one,
            blst_p1,
            blst_p1_affine,
            blst_p2_affine,
            blst_fp
};
use crate::kzg_types::{FsKZGSettings, FsFFTSettings, FsPoly, FsFr, FsG1, FsG2};
use crate::utils::{is_power_of_two, log_2_byte};
use crate::consts::{G1_GENERATOR, G2_GENERATOR};

const SCALE_FACTOR: u64 = 5;

pub fn scale_poly(p: Vec<FsFr>, len_p: usize) {
    let scale_factor = FsFr::from_u64(SCALE_FACTOR);
    let inv_factor = FsFr::inverse(&scale_factor);
    let factor_power = FsFr::one();

    for i in 1..len_p {
        factor_power.mul(&inv_factor);
        p[i].mul(&factor_power);
    }
}

pub fn unscale_poly(p: Vec<FsFr>, len_p: usize) {
    let scale_factor = FsFr::from_u64(SCALE_FACTOR);
    let factor_power = FsFr::one();

    for i in 1..len_p {
        factor_power.mul(&scale_factor);
        p[i].mul(&factor_power);
    }
}

// pub fn recover_poly_from_samples(samples: &[FsFr], len_samples: usize, fs: &FsFFTSettings) -> Result<Vec<FsFr>, String> {

//     assert!(is_power_of_two(len_samples));

//     // uint64_t *missing;
//     // TRY(new_uint64_array(&missing, len_samples));
//     let missing: Vec<usize>;
//     for _ in 0..len_samples {
//         missing.push(0);
//     }

//     let len_missing: usize = 0;
//     for i in 0..len_samples {
//         if samples[i].is_null() {
//             missing[len_missing] = i;
//             len_missing += 1;
//         }
//     }

//     // Make scratch areas, each of size len_samples. Cuts space required by 57%.
//     fr_t *scratch;
//     TRY(new_fr_array(&scratch, 3 * len_samples));
//     fr_t *scratch0 = scratch;
//     fr_t *scratch1 = scratch0 + len_samples;
//     fr_t *scratch2 = scratch1 + len_samples;

//     // Assign meaningful names to scratch spaces
//     fr_t *zero_eval = scratch0;
//     fr_t *poly_evaluations_with_zero = scratch2;
//     fr_t *poly_with_zero = scratch0;
//     fr_t *eval_scaled_poly_with_zero = scratch2;
//     fr_t *eval_scaled_zero_poly = scratch0;
//     fr_t *scaled_reconstructed_poly = scratch1;

//     poly zero_poly;
//     zero_poly.length = len_samples;
//     zero_poly.coeffs = scratch1;

//     // Calculate `Z_r,I`
//     TRY(zero_polynomial_via_multiplication(zero_eval, &zero_poly, len_samples, missing, len_missing, fs));

//     // Check all is well
//     // for (uint64_t i = 0; i < len_samples; i++) {
//     //     ASSERT(fr_is_null(&samples[i]) == fr_is_zero(&zero_eval[i]));
//     // }
//     for i in 0..len_samples {
//         assert!(samples[i].is_null() == zero_eval[i].is_zero());
//     }

//     // Construct E * Z_r,I: the loop makes the evaluation polynomial
//     for (uint64_t i = 0; i < len_samples; i++) {
//         if (fr_is_null(&samples[i])) {
//             poly_evaluations_with_zero[i] = fr_zero;
//         } else {
//             fr_mul(&poly_evaluations_with_zero[i], &samples[i], &zero_eval[i]);
//         }
//     }
//     // Now inverse FFT so that poly_with_zero is (E * Z_r,I)(x) = (D * Z_r,I)(x)
//     TRY(fft_fr(poly_with_zero, poly_evaluations_with_zero, true, len_samples, fs));

//     // x -> k * x
//     scale_poly(poly_with_zero, len_samples);
//     scale_poly(zero_poly.coeffs, zero_poly.length);

//     // Q1 = (D * Z_r,I)(k * x)
//     fr_t *scaled_poly_with_zero = poly_with_zero; // Renaming
//     // Q2 = Z_r,I(k * x)
//     fr_t *scaled_zero_poly = zero_poly.coeffs; // Renaming

//     // Polynomial division by convolution: Q3 = Q1 / Q2
//     TRY(fft_fr(eval_scaled_poly_with_zero, scaled_poly_with_zero, false, len_samples, fs));
//     TRY(fft_fr(eval_scaled_zero_poly, scaled_zero_poly, false, len_samples, fs));

//     fr_t *eval_scaled_reconstructed_poly = eval_scaled_poly_with_zero;
//     for (uint64_t i = 0; i < len_samples; i++) {
//         fr_div(&eval_scaled_reconstructed_poly[i], &eval_scaled_poly_with_zero[i], &eval_scaled_zero_poly[i]);
//     }

//     // The result of the division is D(k * x):
//     TRY(fft_fr(scaled_reconstructed_poly, eval_scaled_reconstructed_poly, true, len_samples, fs));

//     // k * x -> x
//     unscale_poly(scaled_reconstructed_poly, len_samples);

//     // Finally we have D(x) which evaluates to our original data at the powers of roots of unity
//     fr_t *reconstructed_poly = scaled_reconstructed_poly; // Renaming

//     // The evaluation polynomial for D(x) is the reconstructed data:
//     TRY(fft_fr(reconstructed_data, reconstructed_poly, false, len_samples, fs));

//     // Check all is well
//     // for (uint64_t i = 0; i < len_samples; i++) {
//     //     ASSERT(fr_is_null(&samples[i]) || fr_equal(&reconstructed_data[i], &samples[i]));
//     // }
//     for i in 0..len_samples {
//         assert!(samples[i].is_null() || reconstructed_data[i].equals(&samples[i]));
//     }
// }