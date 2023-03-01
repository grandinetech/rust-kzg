use kzg::{FFTFr, FFTSettings, Fr, G1Mul, G2Mul, KZGSettings, Poly, G1, G2};

use crate::consts::{G1_GENERATOR, G2_GENERATOR};
use crate::kzg_proofs::{g1_linear_combination, pairings_verify};
use crate::types::fft_settings::FsFFTSettings;
use crate::types::fr::FsFr;
use crate::types::g1::FsG1;
use crate::types::g2::FsG2;
use crate::types::poly::FsPoly;
use crate::utils::is_power_of_two;

pub struct FsKZGSettings {
    pub fs: FsFFTSettings,
    // Both secret_g1 and secret_g2 have the same number of elements
    pub secret_g1: Vec<FsG1>,
    pub secret_g2: Vec<FsG2>,
}

impl Clone for FsKZGSettings {
    fn clone(&self) -> Self {
        Self {
            fs: self.fs.clone(),
            secret_g1: self.secret_g1.clone(),
            secret_g2: self.secret_g2.clone(),
        }
    }
}

impl KZGSettings<FsFr, FsG1, FsG2, FsFFTSettings, FsPoly> for FsKZGSettings {
    fn default() -> Self {
        Self {
            secret_g1: Vec::new(),
            secret_g2: Vec::new(),
            fs: FsFFTSettings::default(),
        }
    }

    fn new(
        secret_g1: &[FsG1],
        secret_g2: &[FsG2],
        length: usize,
        fft_settings: &FsFFTSettings,
    ) -> Result<Self, String> {
        let mut kzg_settings = Self::default();

        if secret_g1.len() < fft_settings.max_width {
            return Err(String::from(
                "secret_g1 must have a length equal to or greater than fft_settings roots",
            ));
        } else if secret_g2.len() < fft_settings.max_width {
            return Err(String::from(
                "secret_g2 must have a length equal to or greater than fft_settings roots",
            ));
        } else if length < fft_settings.max_width {
            return Err(String::from(
                "length must be equal to or greater than number of fft_settings roots",
            ));
        }

        for i in 0..length {
            kzg_settings.secret_g1.push(secret_g1[i]);
            kzg_settings.secret_g2.push(secret_g2[i]);
        }
        kzg_settings.fs = fft_settings.clone();

        Ok(kzg_settings)
    }

    fn commit_to_poly(&self, poly: &FsPoly) -> Result<FsG1, String> {
        if poly.coeffs.len() > self.secret_g1.len() {
            return Err(String::from("Polynomial is longer than secret g1"));
        }

        let mut out = FsG1::default();
        g1_linear_combination(&mut out, &self.secret_g1, &poly.coeffs, poly.coeffs.len());

        Ok(out)
    }

    fn compute_proof_single(&self, p: &FsPoly, x: &FsFr) -> Result<FsG1, String> {
        self.compute_proof_multi(p, x, 1)
    }

    fn check_proof_single(
        &self,
        com: &FsG1,
        proof: &FsG1,
        x: &FsFr,
        y: &FsFr,
    ) -> Result<bool, String> {
        let x_g2: FsG2 = G2_GENERATOR.mul(x);
        let s_minus_x: FsG2 = self.secret_g2[1].sub(&x_g2);
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
        if !is_power_of_two(n) {
            return Err(String::from("n must be a power of two"));
        }

        // Construct x^n - x0^n = (x - x0.w^0)(x - x0.w^1)...(x - x0.w^(n-1))
        let mut divisor: FsPoly = FsPoly { coeffs: Vec::new() };

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
        let q = new_polina.div(&divisor).unwrap();

        let ret = self.commit_to_poly(&q).unwrap();

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
        if !is_power_of_two(n) {
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
        let xn_minus_yn = self.secret_g2[n].sub(&xn2);

        // [interpolation_polynomial(s)]_1
        let is1 = self.commit_to_poly(&interp).unwrap();

        // [commitment - interpolation_polynomial(s)]_1 = [commit]_1 - [interpolation_polynomial(s)]_1
        let commit_minus_interp = com.sub(&is1);

        let ret = pairings_verify(&commit_minus_interp, &G2_GENERATOR, proof, &xn_minus_yn);

        Ok(ret)
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> FsFr {
        self.fs.get_expanded_roots_of_unity_at(i)
    }
}
