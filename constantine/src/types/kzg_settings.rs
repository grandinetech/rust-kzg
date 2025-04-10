extern crate alloc;

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use kzg::eip_4844::{FIELD_ELEMENTS_PER_BLOB, TRUSTED_SETUP_NUM_G2_POINTS};
use kzg::eth::c_bindings::CKZGSettings;
use kzg::msm::precompute::{precompute, PrecomputationTable};
use kzg::{eth, FFTFr, FFTSettings, Fr, G1Mul, G2Mul, KZGSettings, Poly, G1, G2};

use crate::consts::{G1_GENERATOR, G2_GENERATOR};
use crate::fft_g1::fft_g1_fast;
use crate::kzg_proofs::{g1_linear_combination, pairings_verify};
use crate::types::fft_settings::CtFFTSettings;
use crate::types::fr::CtFr;
use crate::types::g1::CtG1;
use crate::types::g2::CtG2;
use crate::types::poly::CtPoly;
use crate::utils::{fft_settings_to_rust, PRECOMPUTATION_TABLES};

use super::fp::CtFp;
use super::g1::{CtG1Affine, CtG1ProjAddAffine};

#[derive(Clone, Default)]
#[allow(clippy::type_complexity)]
pub struct CtKZGSettings {
    pub fs: CtFFTSettings,
    pub g1_values_monomial: Vec<CtG1>,
    pub g1_values_lagrange_brp: Vec<CtG1>,
    pub g2_values_monomial: Vec<CtG2>,
    pub precomputation:
        Option<Arc<PrecomputationTable<CtFr, CtG1, CtFp, CtG1Affine, CtG1ProjAddAffine>>>,
    pub x_ext_fft_columns: Vec<Vec<CtG1>>,
    pub cell_size: usize,
}

fn toeplitz_part_1(
    field_elements_per_ext_blob: usize,
    output: &mut [CtG1],
    x: &[CtG1],
    s: &CtFFTSettings,
) -> Result<(), String> {
    let n = x.len();
    let n2 = n * 2;
    let mut x_ext = vec![CtG1::identity(); n2];

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

impl KZGSettings<CtFr, CtG1, CtG2, CtFFTSettings, CtPoly, CtFp, CtG1Affine, CtG1ProjAddAffine>
    for CtKZGSettings
{
    fn new(
        g1_monomial: &[CtG1],
        g1_lagrange_brp: &[CtG1],
        g2_monomial: &[CtG2],
        fft_settings: &CtFFTSettings,
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

        let mut points = vec![CtG1::default(); k2];
        let mut x = vec![CtG1::default(); k];
        let mut x_ext_fft_columns = vec![vec![CtG1::default(); cell_size]; k2];

        for offset in 0..cell_size {
            let start = n - cell_size - 1 - offset;
            for (i, p) in x.iter_mut().enumerate().take(k - 1) {
                let j = start - i * cell_size;
                *p = g1_monomial[j];
            }
            x[k - 1] = CtG1::identity();

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
            precomputation: precompute(g1_lagrange_brp, &x_ext_fft_columns)
                .ok()
                .flatten()
                .map(Arc::new),
            x_ext_fft_columns,
            cell_size,
        })
    }

    fn commit_to_poly(&self, poly: &CtPoly) -> Result<CtG1, String> {
        if poly.coeffs.len() > self.g1_values_monomial.len() {
            return Err(String::from("Polynomial is longer than secret g1"));
        }

        let mut out = CtG1::default();
        g1_linear_combination(
            &mut out,
            &self.g1_values_monomial,
            &poly.coeffs,
            poly.coeffs.len(),
            None,
        );

        Ok(out)
    }

    fn compute_proof_single(&self, p: &CtPoly, x: &CtFr) -> Result<CtG1, String> {
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

        let q = CtPoly { coeffs: out_coeffs };

        let ret = self.commit_to_poly(&q)?;

        Ok(ret)
    }

    fn check_proof_single(
        &self,
        com: &CtG1,
        proof: &CtG1,
        x: &CtFr,
        y: &CtFr,
    ) -> Result<bool, String> {
        let x_g2: CtG2 = G2_GENERATOR.mul(x);
        let s_minus_x: CtG2 = self.g2_values_monomial[1].sub(&x_g2);
        let y_g1 = G1_GENERATOR.mul(y);
        let commitment_minus_y: CtG1 = com.sub(&y_g1);

        Ok(pairings_verify(
            &commitment_minus_y,
            &G2_GENERATOR,
            proof,
            &s_minus_x,
        ))
    }
    fn compute_proof_multi(&self, p: &CtPoly, x0: &CtFr, n: usize) -> Result<CtG1, String> {
        if p.coeffs.is_empty() {
            return Err(String::from("Polynomial must not be empty"));
        }

        if !n.is_power_of_two() {
            return Err(String::from("n must be a power of two"));
        }

        // Construct x^n - x0^n = (x - x0.w^0)(x - x0.w^1)...(x - x0.w^(n-1))
        let mut divisor = CtPoly {
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
        com: &CtG1,
        proof: &CtG1,
        x: &CtFr,
        ys: &[CtFr],
        n: usize,
    ) -> Result<bool, String> {
        if !n.is_power_of_two() {
            return Err(String::from("n is not a power of two"));
        }

        // Interpolate at a coset.
        let mut interp = CtPoly {
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

    fn get_roots_of_unity_at(&self, i: usize) -> CtFr {
        self.fs.get_roots_of_unity_at(i)
    }

    fn get_fft_settings(&self) -> &CtFFTSettings {
        &self.fs
    }

    fn get_g1_lagrange_brp(&self) -> &[CtG1] {
        &self.g1_values_lagrange_brp
    }

    fn get_g1_monomial(&self) -> &[CtG1] {
        &self.g1_values_monomial
    }

    fn get_g2_monomial(&self) -> &[CtG2] {
        &self.g2_values_monomial
    }

    fn get_precomputation(
        &self,
    ) -> Option<&PrecomputationTable<CtFr, CtG1, CtFp, CtG1Affine, CtG1ProjAddAffine>> {
        self.precomputation.as_ref().map(|v| v.as_ref())
    }

    fn get_x_ext_fft_columns(&self) -> &[Vec<CtG1>] {
        &self.x_ext_fft_columns
    }

    fn get_cell_size(&self) -> usize {
        self.cell_size
    }
}

impl<'a> TryFrom<&'a CKZGSettings> for CtKZGSettings {
    type Error = String;

    fn try_from(c_settings: &'a CKZGSettings) -> Result<Self, Self::Error> {
        Ok(CtKZGSettings {
            fs: fft_settings_to_rust(c_settings)?,
            g1_values_monomial: unsafe {
                core::slice::from_raw_parts(c_settings.g1_values_monomial, FIELD_ELEMENTS_PER_BLOB)
            }
            .iter()
            .map(|r| CtG1::from_blst_p1(*r))
            .collect::<Vec<_>>(),
            g1_values_lagrange_brp: unsafe {
                core::slice::from_raw_parts(
                    c_settings.g1_values_lagrange_brp,
                    FIELD_ELEMENTS_PER_BLOB,
                )
            }
            .iter()
            .map(|r| CtG1::from_blst_p1(*r))
            .collect::<Vec<_>>(),
            g2_values_monomial: unsafe {
                core::slice::from_raw_parts(
                    c_settings.g2_values_monomial,
                    TRUSTED_SETUP_NUM_G2_POINTS,
                )
            }
            .iter()
            .map(|r| CtG2::from_blst_p2(*r))
            .collect::<Vec<_>>(),
            x_ext_fft_columns: unsafe {
                core::slice::from_raw_parts(
                    c_settings.x_ext_fft_columns,
                    2 * ((eth::FIELD_ELEMENTS_PER_EXT_BLOB / 2) / eth::FIELD_ELEMENTS_PER_CELL),
                )
            }
            .iter()
            .map(|it| {
                unsafe { core::slice::from_raw_parts(*it, eth::FIELD_ELEMENTS_PER_CELL) }
                    .iter()
                    .map(|it| CtG1::from_blst_p1(*it))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
            #[allow(static_mut_refs)]
            precomputation: unsafe { PRECOMPUTATION_TABLES.get_precomputation(c_settings) },
            cell_size: eth::FIELD_ELEMENTS_PER_CELL,
        })
    }
}
