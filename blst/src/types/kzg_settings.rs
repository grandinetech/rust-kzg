extern crate alloc;

use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::{vec, vec::Vec};

use kzg::eth::c_bindings::CKZGSettings;
use kzg::eth::{self, FIELD_ELEMENTS_PER_EXT_BLOB};
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
use crate::utils::PRECOMPUTATION_TABLES;

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
    pub cell_size: usize,
}

fn toeplitz_part_1(
    field_elements_per_ext_blob: usize,
    output: &mut [FsG1],
    x: &[FsG1],
    s: &FsFFTSettings,
) -> Result<(), String> {
    let n = x.len();
    let n2 = n * 2;
    let mut x_ext = vec![FsG1::identity(); n2];

    x_ext[..n].copy_from_slice(x);

    let x_ext = &x_ext[..];

    /* Ensure the length is valid */
    if x_ext.len() > field_elements_per_ext_blob || !x_ext.len().is_power_of_two() {
        return Err("Invalid input size".to_string());
    }

    let roots_stride = field_elements_per_ext_blob / x_ext.len();
    fft_g1_fast(output, x_ext, 1, &s.roots_of_unity, roots_stride);

    Ok(())
}

impl KZGSettings<FsFr, FsG1, FsG2, FsFFTSettings, FsPoly, FsFp, FsG1Affine> for FsKZGSettings {
    fn new(
        g1_monomial: &[FsG1],
        g1_lagrange_brp: &[FsG1],
        g2_monomial: &[FsG2],
        fft_settings: &FsFFTSettings,
        cell_size: usize,
    ) -> Result<Self, String> {
        if g1_monomial.len() != g1_lagrange_brp.len() {
            return Err("G1 point length mismatch".to_string());
        }

        let field_elements_per_blob = g1_monomial.len();
        let field_elements_per_ext_blob = field_elements_per_blob * 2;

        let n = field_elements_per_ext_blob / 2;
        let k = n / cell_size;
        let k2 = 2 * k;

        let mut points = vec![FsG1::default(); k2];
        let mut x = vec![FsG1::default(); k];
        let mut x_ext_fft_columns = vec![vec![FsG1::default(); cell_size]; k2];

        for offset in 0..cell_size {
            let start = n - cell_size - 1 - offset;
            for (i, p) in x.iter_mut().enumerate().take(k - 1) {
                let j = start - i * cell_size;
                *p = g1_monomial[j];
            }
            x[k - 1] = FsG1::identity();

            toeplitz_part_1(field_elements_per_ext_blob, &mut points, &x, fft_settings)?;

            for row in 0..k2 {
                x_ext_fft_columns[row][offset] = points[row];
            }
        }

        Ok(Self {
            g1_values_monomial: g1_monomial.to_vec(),
            g1_values_lagrange_brp: g1_lagrange_brp.to_vec(),
            g2_values_monomial: g2_monomial.to_vec(),
            fs: fft_settings.clone(),
            precomputation: {
                #[cfg(feature = "sppark")]
                {
                    use blst::blst_p1_affine;
                    let points = kzg::msm::msm_impls::batch_convert::<FsG1, FsFp, FsG1Affine>(
                        g1_lagrange_brp,
                    );
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
                    precompute(g1_lagrange_brp, &x_ext_fft_columns)
                        .ok()
                        .flatten()
                        .map(Arc::new)
                }
            },
            x_ext_fft_columns,
            cell_size,
        })
    }

    fn commit_to_poly(&self, poly: &FsPoly) -> Result<FsG1, String> {
        if poly.coeffs.len() > self.g1_values_monomial.len() {
            return Err(String::from("Polynomial is longer than secret g1"));
        }

        let mut out = FsG1::default();
        g1_linear_combination(
            &mut out,
            &self.g1_values_monomial,
            &poly.coeffs,
            poly.coeffs.len(),
            None,
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

    fn get_g1_lagrange_brp(&self) -> &[FsG1] {
        &self.g1_values_lagrange_brp
    }

    fn get_g1_monomial(&self) -> &[FsG1] {
        &self.g1_values_monomial
    }

    fn get_g2_monomial(&self) -> &[FsG2] {
        &self.g2_values_monomial
    }

    fn get_precomputation(&self) -> Option<&PrecomputationTable<FsFr, FsG1, FsFp, FsG1Affine>> {
        self.precomputation.as_ref().map(|v| v.as_ref())
    }

    fn get_x_ext_fft_columns(&self) -> &[Vec<FsG1>] {
        &self.x_ext_fft_columns
    }

    fn get_cell_size(&self) -> usize {
        self.cell_size
    }
}

impl<'a> TryFrom<&'a CKZGSettings> for FsKZGSettings {
    type Error = String;

    fn try_from(settings: &'a CKZGSettings) -> Result<Self, Self::Error> {
        let roots_of_unity = unsafe {
            core::slice::from_raw_parts(settings.roots_of_unity, FIELD_ELEMENTS_PER_EXT_BLOB + 1)
                .iter()
                .map(|r| FsFr(blst::blst_fr { l: r.l }))
                .collect::<Vec<FsFr>>()
        };

        let brp_roots_of_unity = unsafe {
            core::slice::from_raw_parts(settings.brp_roots_of_unity, FIELD_ELEMENTS_PER_EXT_BLOB)
                .iter()
                .map(|r| FsFr(blst::blst_fr { l: r.l }))
                .collect::<Vec<FsFr>>()
        };

        let reverse_roots_of_unity = unsafe {
            core::slice::from_raw_parts(
                settings.reverse_roots_of_unity,
                FIELD_ELEMENTS_PER_EXT_BLOB + 1,
            )
            .iter()
            .map(|r| FsFr(blst::blst_fr { l: r.l }))
            .collect::<Vec<FsFr>>()
        };

        let fft_settings = FsFFTSettings {
            max_width: FIELD_ELEMENTS_PER_EXT_BLOB,
            root_of_unity: roots_of_unity[1],
            roots_of_unity,
            brp_roots_of_unity,
            reverse_roots_of_unity,
        };

        Ok(FsKZGSettings {
            fs: fft_settings,
            g1_values_monomial: unsafe {
                core::slice::from_raw_parts(
                    settings.g1_values_monomial,
                    eth::FIELD_ELEMENTS_PER_BLOB,
                )
            }
            .iter()
            .map(|r| {
                FsG1(blst::blst_p1 {
                    x: blst::blst_fp { l: r.x.l },
                    y: blst::blst_fp { l: r.y.l },
                    z: blst::blst_fp { l: r.z.l },
                })
            })
            .collect::<Vec<_>>(),
            g1_values_lagrange_brp: unsafe {
                core::slice::from_raw_parts(
                    settings.g1_values_lagrange_brp,
                    eth::FIELD_ELEMENTS_PER_BLOB,
                )
            }
            .iter()
            .map(|r| {
                FsG1(blst::blst_p1 {
                    x: blst::blst_fp { l: r.x.l },
                    y: blst::blst_fp { l: r.y.l },
                    z: blst::blst_fp { l: r.z.l },
                })
            })
            .collect::<Vec<_>>(),
            g2_values_monomial: unsafe {
                core::slice::from_raw_parts(
                    settings.g2_values_monomial,
                    eth::TRUSTED_SETUP_NUM_G2_POINTS,
                )
            }
            .iter()
            .map(|r| {
                FsG2(blst::blst_p2 {
                    x: blst::blst_fp2 {
                        fp: [
                            blst::blst_fp { l: r.x.fp[0].l },
                            blst::blst_fp { l: r.x.fp[1].l },
                        ],
                    },
                    y: blst::blst_fp2 {
                        fp: [
                            blst::blst_fp { l: r.y.fp[0].l },
                            blst::blst_fp { l: r.y.fp[1].l },
                        ],
                    },
                    z: blst::blst_fp2 {
                        fp: [
                            blst::blst_fp { l: r.z.fp[0].l },
                            blst::blst_fp { l: r.z.fp[1].l },
                        ],
                    },
                })
            })
            .collect::<Vec<_>>(),
            x_ext_fft_columns: unsafe {
                core::slice::from_raw_parts(
                    settings.x_ext_fft_columns,
                    2 * ((FIELD_ELEMENTS_PER_EXT_BLOB / 2) / eth::FIELD_ELEMENTS_PER_CELL),
                )
            }
            .iter()
            .map(|it| {
                unsafe { core::slice::from_raw_parts(*it, eth::FIELD_ELEMENTS_PER_CELL) }
                    .iter()
                    .map(|r| {
                        FsG1(blst::blst_p1 {
                            x: blst::blst_fp { l: r.x.l },
                            y: blst::blst_fp { l: r.y.l },
                            z: blst::blst_fp { l: r.z.l },
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
            #[allow(static_mut_refs)]
            precomputation: unsafe { PRECOMPUTATION_TABLES.get_precomputation(settings) },
            cell_size: eth::FIELD_ELEMENTS_PER_CELL,
        })
    }
}
