extern crate alloc;

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use kzg::eip_4844::{FIELD_ELEMENTS_PER_BLOB, FIELD_ELEMENTS_PER_CELL, FIELD_ELEMENTS_PER_EXT_BLOB, TRUSTED_SETUP_NUM_G2_POINTS};
use kzg::msm::precompute::{precompute, PrecomputationTable};
use kzg::{FFTFr, FFTSettings, Fr, G1Mul, G2Mul, KZGSettings, Poly, G1, G2};

use crate::consts::{G1_GENERATOR, G2_GENERATOR};
use crate::fft_g1::fft_g1_fast;
use crate::kzg_proofs::{g1_linear_combination, pairings_verify};
use crate::types::fft_settings::FsFFTSettings;
use crate::types::fr::FsFr;
use crate::types::g1::FsG1;
use crate::types::g2::FsG2;
use crate::types::poly::FsPoly;

use super::fp::FsFp;
use super::g1::FsG1Affine;

#[derive(Debug, Clone, Default)]
pub struct FsKZGSettings {
    pub fs: FsFFTSettings,
    pub g1_values_monomial: Vec<FsG1>,
    pub g1_values_lagrange_brp: Vec<FsG1>,
    pub g2_values_monomial: Vec<FsG2>,
    pub precomputation: Option<Arc<PrecomputationTable<FsFr, FsG1, FsFp, FsG1Affine>>>,
    pub x_ext_fft_columns: Vec<Vec<FsG1>>,
}

fn g1_fft(output: &mut [FsG1], input: &[FsG1], s: &FsFFTSettings) -> Result<(), String> {
    // g1_t *out, const g1_t *in, size_t n, const KZGSettings *s

    /* Ensure the length is valid */
    if input.len() > FIELD_ELEMENTS_PER_EXT_BLOB || !input.len().is_power_of_two() {
        return Err("Invalid input size".to_string());
    }

    let roots_stride = FIELD_ELEMENTS_PER_EXT_BLOB / input.len();
    fft_g1_fast(output, input, 1, &s.roots_of_unity, roots_stride);

    return Ok(());
}

fn toeplitz_part_1(output: &mut [FsG1], x: &[FsG1], s: &FsFFTSettings) -> Result<(), String> {
    // g1_t *out, const g1_t *x, size_t n, const KZGSettings *s

    //     C_KZG_RET ret;
    //     size_t n2 = n * 2;
    let n = x.len();
    let n2 = n * 2;
    //     g1_t *x_ext;

    //     /* Create extended array of points */
    //     ret = new_g1_array(&x_ext, n2);
    let mut x_ext = vec![FsG1::identity(); n2];
    //     if (ret != C_KZG_OK) goto out;

    //     /* Copy x & extend with zero */
    //     for (size_t i = 0; i < n; i++) {
    //         x_ext[i] = x[i];
    //     }

    for i in 0..n {
        x_ext[i] = x[i];
    }

    //     for (size_t i = n; i < n2; i++) {
    //         x_ext[i] = G1_IDENTITY;
    //     }

    //     /* Peform forward transformation */
    g1_fft(output, &x_ext, s)?;
    //     ret = g1_fft(out, x_ext, n2, s);
    //     if (ret != C_KZG_OK) goto out;

    // out:
    //     c_kzg_free(x_ext);
    //     return ret;
    Ok(())
}

impl KZGSettings<FsFr, FsG1, FsG2, FsFFTSettings, FsPoly, FsFp, FsG1Affine> for FsKZGSettings {
    fn new(
        g1_monomial: &[FsG1],
        g1_lagrange_brp: &[FsG1],
        g2_monomial: &[FsG2],
        fft_settings: &FsFFTSettings,
    ) -> Result<Self, String> {
        if g1_monomial.len() != FIELD_ELEMENTS_PER_BLOB || g1_lagrange_brp.len() != FIELD_ELEMENTS_PER_BLOB || g2_monomial.len() != TRUSTED_SETUP_NUM_G2_POINTS {
            return Err("Length does not match FIELD_ELEMENTS_PER_BLOB".to_string());
        }

        let n = FIELD_ELEMENTS_PER_EXT_BLOB / 2;
        let k = n / FIELD_ELEMENTS_PER_CELL;
        let k2 = 2 * k;

        let mut points = vec![FsG1::default(); k2];
        let mut x = vec![FsG1::default(); k];
        let mut x_ext_fft_columns = vec![vec![FsG1::default(); FIELD_ELEMENTS_PER_CELL]; k2];

        for offset in 0..FIELD_ELEMENTS_PER_CELL {
            let start = n - FIELD_ELEMENTS_PER_CELL - 1 - offset;
            for i in 0..(k - 1) {
                let j = start - i * FIELD_ELEMENTS_PER_CELL;
                x[i] = g1_monomial[j];
            }
            x[k - 1] = FsG1::identity();

            toeplitz_part_1(&mut points, &x, &fft_settings)?;

            for row in 0..k2 {
                x_ext_fft_columns[row][offset] = points[row];
            }
        }

        // for (size_t offset = 0; offset < FIELD_ELEMENTS_PER_CELL; offset++) {
        //     /* Compute x, sections of the g1 values */
        //     size_t start = n - FIELD_ELEMENTS_PER_CELL - 1 - offset;
        //     for (size_t i = 0; i < k - 1; i++) {
        //         size_t j = start - i * FIELD_ELEMENTS_PER_CELL;
        //         x[i] = s->g1_values_monomial[j];
        //     }
        //     x[k - 1] = G1_IDENTITY;

        //     /* Compute points, the fft of an extended x */
        //     ret = toeplitz_part_1(points, x, k, s);
        //     if (ret != C_KZG_OK) goto out;

        //     /* Reorganize from rows into columns */
        //     for (size_t row = 0; row < k2; row++) {
        //         s->x_ext_fft_columns[row][offset] = points[row];
        //     }
        // }

        Ok(Self {
            g1_values_monomial: g1_monomial.to_vec(),
            g1_values_lagrange_brp: g1_lagrange_brp.to_vec(),
            g2_values_monomial: g2_monomial.to_vec(),
            fs: fft_settings.clone(),
            x_ext_fft_columns,
            precomputation: {
                #[cfg(feature = "sppark")]
                {
                    use blst::blst_p1_affine;
                    let points =
                        kzg::msm::msm_impls::batch_convert::<FsG1, FsFp, FsG1Affine>(secret_g1);
                    let points = unsafe {
                        alloc::slice::from_raw_parts(
                            points.as_ptr() as *const blst_p1_affine,
                            points.len(),
                        )
                    };
                    let prepared = rust_kzg_blst_sppark::prepare_multi_scalar_mult(points);
                    Some(Arc::new(PrecomputationTable::from_ptr(prepared)))
                }

                #[cfg(not(feature = "sppark"))]
                {
                    precompute(g1_lagrange_brp).ok().flatten().map(Arc::new)
                }
            },
        })
    }

    fn commit_to_poly(&self, poly: &FsPoly) -> Result<FsG1, String> {
        if poly.coeffs.len() > self.g1_values_lagrange_brp.len() {
            return Err(String::from("Polynomial is longer than secret g1"));
        }

        let mut out = FsG1::default();
        g1_linear_combination(
            &mut out,
            &self.g1_values_lagrange_brp,
            &poly.coeffs,
            poly.coeffs.len(),
            self.get_precomputation(),
        );

        Ok(out)
    }

    fn compute_proof_single(&self, p: &FsPoly, x: &FsFr) -> Result<FsG1, String> {
        if p.coeffs.is_empty() {
            return Err(String::from("Polynomial must not be empty"));
        }

        // `-(x0^n)`, where `n` is `1`
        let divisor_0 = x.negate();

        // Calculate `q = p / (x^n - x0^n)` for our reduced case (see `compute_proof_multi` for
        // generic implementation)
        let mut out_coeffs = Vec::from(&p.coeffs[1..]);
        for i in (1..out_coeffs.len()).rev() {
            let tmp = out_coeffs[i].mul(&divisor_0);
            out_coeffs[i - 1] = out_coeffs[i - 1].sub(&tmp);
        }

        let q = FsPoly { coeffs: out_coeffs };

        let ret = self.commit_to_poly(&q)?;

        Ok(ret)
    }

    fn check_proof_single(
        &self,
        com: &FsG1,
        proof: &FsG1,
        x: &FsFr,
        y: &FsFr,
    ) -> Result<bool, String> {
        let x_g2: FsG2 = G2_GENERATOR.mul(x);
        let s_minus_x: FsG2 = self.g2_values_monomial[1].sub(&x_g2);
        let y_g1 = G1_GENERATOR.mul(y);
        let commitment_minus_y: FsG1 = com.sub(&y_g1);

        Ok(pairings_verify(
            &commitment_minus_y,
            &G2_GENERATOR,
            proof,
            &s_minus_x,
        ))
    }

    fn compute_proof_multi(&self, p: &FsPoly, x0: &FsFr, n: usize) -> Result<FsG1, String> {
        if p.coeffs.is_empty() {
            return Err(String::from("Polynomial must not be empty"));
        }

        if !n.is_power_of_two() {
            return Err(String::from("n must be a power of two"));
        }

        // Construct x^n - x0^n = (x - x0.w^0)(x - x0.w^1)...(x - x0.w^(n-1))
        let mut divisor = FsPoly {
            coeffs: Vec::with_capacity(n + 1),
        };

        // -(x0^n)
        let x_pow_n = x0.pow(n);

        divisor.coeffs.push(x_pow_n.negate());

        // Zeros
        for _ in 1..n {
            divisor.coeffs.push(Fr::zero());
        }

        // x^n
        divisor.coeffs.push(Fr::one());

        let mut new_polina = p.clone();

        // Calculate q = p / (x^n - x0^n)
        // let q = p.div(&divisor).unwrap();
        let q = new_polina.div(&divisor)?;

        let ret = self.commit_to_poly(&q)?;

        Ok(ret)
    }

    fn check_proof_multi(
        &self,
        com: &FsG1,
        proof: &FsG1,
        x: &FsFr,
        ys: &[FsFr],
        n: usize,
    ) -> Result<bool, String> {
        if !n.is_power_of_two() {
            return Err(String::from("n is not a power of two"));
        }

        // Interpolate at a coset.
        let mut interp = FsPoly {
            coeffs: self.fs.fft_fr(ys, true)?,
        };

        let inv_x = x.inverse(); // Not euclidean?
        let mut inv_x_pow = inv_x;
        for i in 1..n {
            interp.coeffs[i] = interp.coeffs[i].mul(&inv_x_pow);
            inv_x_pow = inv_x_pow.mul(&inv_x);
        }

        // [x^n]_2
        let x_pow = inv_x_pow.inverse();

        let xn2 = G2_GENERATOR.mul(&x_pow);

        // [s^n - x^n]_2
        let xn_minus_yn = self.g2_values_monomial[n].sub(&xn2);

        // [interpolation_polynomial(s)]_1
        let is1 = self.commit_to_poly(&interp).unwrap();

        // [commitment - interpolation_polynomial(s)]_1 = [commit]_1 - [interpolation_polynomial(s)]_1
        let commit_minus_interp = com.sub(&is1);

        let ret = pairings_verify(&commit_minus_interp, &G2_GENERATOR, proof, &xn_minus_yn);

        Ok(ret)
    }

    fn get_roots_of_unity_at(&self, i: usize) -> FsFr {
        self.fs.get_roots_of_unity_at(i)
    }

    fn get_fft_settings(&self) -> &FsFFTSettings {
        &self.fs
    }

    fn get_g1_secret(&self) -> &[FsG1] {
        &self.g1_values_lagrange_brp
    }

    fn get_g2_secret(&self) -> &[FsG2] {
        &self.g2_values_monomial
    }

    fn get_precomputation(&self) -> Option<&PrecomputationTable<FsFr, FsG1, FsFp, FsG1Affine>> {
        self.precomputation.as_ref().map(|v| v.as_ref())
    }
}
