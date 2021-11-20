use blst::{
    blst_fp, blst_fp2, blst_fr, blst_fr_add, blst_fr_cneg, blst_fr_eucl_inverse,
    blst_fr_from_scalar, blst_fr_from_uint64, blst_fr_inverse, blst_fr_mul, blst_fr_sqr,
    blst_fr_sub, blst_p1, blst_p1_add_or_double, blst_p1_cneg, blst_p1_double, blst_p1_is_equal,
    blst_p1_is_inf, blst_p1_mult, blst_p2, blst_p2_add_or_double, blst_p2_cneg, blst_p2_double,
    blst_p2_is_equal, blst_p2_mult, blst_scalar, blst_scalar_from_fr, blst_uint64_from_fr,
};
use kzg::{
    FFTFr, FFTSettings, FK20MultiSettings, FK20SingleSettings, Fr, G1Mul, G2Mul, KZGSettings, Poly,
    PolyRecover, ZeroPoly, FFTG1, G1, G2,
};

use crate::consts::{
    G1_GENERATOR, G1_IDENTITY, G1_NEGATIVE_GENERATOR, G2_GENERATOR, G2_NEGATIVE_GENERATOR,
    SCALE2_ROOT_OF_UNITY, SCALE_FACTOR,
};
use crate::kzg_proofs::{g1_linear_combination, pairings_verify};
use crate::recovery::{scale_poly, unscale_poly};
use crate::types::fft_settings::FsFFTSettings;
use crate::types::fr::FsFr;
use crate::utils::reverse_bit_order;
use crate::utils::{is_power_of_two, log_2_byte};
use crate::utils::{log2_pow2, log2_u64, min_u64, next_power_of_two};

pub struct FsPoly {
    pub coeffs: Vec<FsFr>,
}

impl Poly<FsFr> for FsPoly {
    fn default() -> Self {
        // Not perfect, but shouldn't fail
        Self::new(0).unwrap()
    }

    fn new(size: usize) -> Result<Self, String> {
        Ok(Self {
            coeffs: vec![FsFr::default(); size],
        })
    }

    fn get_coeff_at(&self, i: usize) -> FsFr {
        self.coeffs[i]
    }

    fn set_coeff_at(&mut self, i: usize, x: &FsFr) {
        self.coeffs[i] = x.clone()
    }

    fn get_coeffs(&self) -> &[FsFr] {
        &self.coeffs
    }

    fn len(&self) -> usize {
        self.coeffs.len()
    }

    fn eval(&self, x: &FsFr) -> FsFr {
        if self.coeffs.is_empty() {
            return FsFr::zero();
        } else if x.is_zero() {
            return self.coeffs[0];
        }

        let mut ret = self.coeffs[self.coeffs.len() - 1];
        let mut i = self.coeffs.len() - 2;
        loop {
            let temp = ret.mul(&x);
            ret = temp.add(&self.coeffs[i]);

            if i == 0 {
                break;
            }
            i -= 1;
        }

        ret
    }

    fn scale(&mut self) {
        let scale_factor = FsFr::from_u64(SCALE_FACTOR);
        let inv_factor = scale_factor.inverse();

        let mut factor_power = FsFr::one();
        for i in 0..self.coeffs.len() {
            factor_power = factor_power.mul(&inv_factor);
            self.coeffs[i] = self.coeffs[i].mul(&factor_power);
        }
    }

    fn unscale(&mut self) {
        let scale_factor = FsFr::from_u64(SCALE_FACTOR);

        let mut factor_power = FsFr::one();
        for i in 0..self.coeffs.len() {
            factor_power = factor_power.mul(&scale_factor);
            self.coeffs[i] = self.coeffs[i].mul(&factor_power);
        }
    }

    // TODO: analyze how algo works
    fn inverse(&mut self, output_len: usize) -> Result<Self, String> {
        if output_len == 0 {
            return Err(String::from("Can't produce a zero-length result"));
        } else if self.coeffs.len() == 0 {
            return Err(String::from("Can't inverse a zero-length poly"));
        } else if self.coeffs[0].is_zero() {
            return Err(String::from(
                "First coefficient of polynomial mustn't be zero",
            ));
        }

        let mut ret = FsPoly {
            coeffs: vec![FsFr::zero(); output_len],
        };
        // If the input polynomial is constant, the remainder of the series is zero
        if self.coeffs.len() == 1 {
            ret.coeffs[0] = self.coeffs[0].eucl_inverse();

            return Ok(ret);
        }

        let maxd = output_len - 1;

        // Max space for multiplications is (2 * length - 1)
        // Don't need the following as its recalculated inside
        // let scale: usize = log2_pow2(next_power_of_two(2 * output_len - 1));
        // let fft_settings = FsFFTSettings::new(scale).unwrap();

        // To store intermediate results

        // Base case for d == 0
        ret.coeffs[0] = self.coeffs[0].eucl_inverse();
        let mut d: usize = 0;
        let mut mask: usize = 1 << log2_u64(maxd);
        while mask != 0 {
            d = 2 * d + (if (maxd & mask) != 0 { 1 } else { 0 });
            mask >>= 1;

            // b.c -> tmp0 (we're using out for c)
            // tmp0.length = min_u64(d + 1, b->length + output->length - 1);
            let len_temp = min_u64(d + 1, self.len() + output_len - 1);
            let mut tmp0 = self.mul(&ret, len_temp).unwrap();

            // 2 - b.c -> tmp0
            for i in 0..tmp0.len() {
                tmp0.coeffs[i] = tmp0.coeffs[i].negate();
            }
            let fr_two = Fr::from_u64(2);
            tmp0.coeffs[0] = tmp0.coeffs[0].add(&fr_two);

            // c.(2 - b.c) -> tmp1;
            let tmp1 = ret.mul(&tmp0, d + 1).unwrap();

            for i in 0..tmp1.len() {
                ret.coeffs[i] = tmp1.coeffs[i];
            }
        }

        if d + 1 != output_len {
            return Err(String::from("D + 1 must be equal to output_len"));
        }

        Ok(ret)
    }

    fn div(&mut self, divisor: &Self) -> Result<Self, String> {
        return if divisor.len() >= self.len() || divisor.len() < 128 {
            // Tunable parameter
            self.long_div(&divisor)
        } else {
            self.fast_div(&divisor)
        };
    }

    fn long_div(&mut self, divisor: &Self) -> Result<Self, String> {
        if divisor.coeffs.len() == 0 {
            return Err(String::from("Can't divide by zero"));
        } else if divisor.coeffs[divisor.coeffs.len() - 1].is_zero() {
            return Err(String::from("Highest coefficient must be non-zero"));
        }

        let out_length = self.poly_quotient_length(&divisor);
        let mut out: FsPoly = FsPoly {
            coeffs: vec![FsFr::default(); out_length],
        };
        if out_length == 0 {
            return Ok(out);
        }

        let mut a_pos = self.len() - 1;
        let b_pos = divisor.len() - 1;
        let mut diff = a_pos - b_pos;

        let mut a = vec![FsFr::default(); self.len()];
        for i in 0..a.len() {
            a[i] = self.coeffs[i];
        }

        while diff > 0 {
            out.coeffs[diff] = a[a_pos].div(&divisor.coeffs[b_pos]).unwrap();

            for i in 0..(b_pos + 1) {
                let tmp = out.coeffs[diff].mul(&divisor.coeffs[i]);
                let tmp = a[diff + i].sub(&tmp);
                a[diff + i] = tmp;
            }

            diff -= 1;
            a_pos -= 1;
        }

        out.coeffs[0] = a[a_pos].div(&divisor.coeffs[b_pos]).unwrap();

        Ok(out)
    }

    fn fast_div(&mut self, divisor: &Self) -> Result<Self, String> {
        if divisor.coeffs.len() == 0 {
            return Err(String::from("Cant divide by zero"));
        } else if divisor.coeffs[divisor.coeffs.len() - 1].is_zero() {
            return Err(String::from("Highest coefficient must be non-zero"));
        }

        let m: usize = self.len() - 1;
        let n: usize = divisor.len() - 1;

        // If the divisor is larger than the dividend, the result is zero-length
        if n > m {
            return Ok(FsPoly { coeffs: Vec::new() });
        }

        // Special case for divisor.length == 1 (it's a constant)
        if divisor.len() == 1 {
            let mut out = FsPoly {
                coeffs: vec![FsFr::zero(); self.len()],
            };
            for i in 0..out.len() {
                out.coeffs[i] = self.coeffs[i].div(&divisor.coeffs[0]).unwrap();
            }
            return Ok(out);
        }

        let mut a_flip = self.flip().unwrap();
        let mut b_flip = divisor.flip().unwrap();

        let inv_b_flip = b_flip.inverse(m - n + 1).unwrap();
        let q_flip = a_flip.mul(&inv_b_flip, m - n + 1).unwrap();

        let out = q_flip.flip().unwrap();
        Ok(out)
    }

    fn mul_direct(&mut self, multiplier: &Self, output_len: usize) -> Result<Self, String> {
        if self.len() == 0 || multiplier.len() == 0 {
            return Ok(FsPoly::new(0).unwrap());
        }

        let a_degree = self.len() - 1;
        let b_degree = multiplier.len() - 1;

        let mut ret = FsPoly {
            coeffs: vec![Fr::zero(); output_len],
        };

        // Truncate the output to the length of the output polynomial
        for i in 0..(a_degree + 1) {
            let mut j = 0;
            while (j <= b_degree) && ((i + j) < output_len) {
                let tmp = self.coeffs[i].mul(&multiplier.coeffs[j]);
                let tmp = ret.coeffs[i + j].add(&tmp);
                ret.coeffs[i + j] = tmp;

                j += 1;
            }
        }

        Ok(ret)
    }
}

impl FsPoly {
    pub fn _poly_norm(&self) -> Self {
        let mut ret = self.clone();

        let mut temp_len: usize = ret.coeffs.len();
        while temp_len > 0 && ret.coeffs[temp_len - 1].is_zero() {
            temp_len -= 1;
        }

        if temp_len == 0 {
            ret.coeffs = Vec::new();
        } else {
            ret.coeffs = ret.coeffs[0..temp_len].to_vec();
        }

        ret
    }

    pub fn poly_quotient_length(&self, divisor: &Self) -> usize {
        if self.len() >= divisor.len() {
            self.len() - divisor.len() + 1
        } else {
            0
        }
    }

    pub fn pad(&self, out_length: usize) -> Self {
        let mut ret = Self {
            coeffs: vec![FsFr::zero(); out_length],
        };

        for i in 0..min_u64(self.len(), out_length) {
            ret.coeffs[i] = self.coeffs[i];
        }

        ret
    }

    pub fn flip(&self) -> Result<FsPoly, String> {
        let mut ret = FsPoly {
            coeffs: vec![FsFr::default(); self.len()],
        };
        for i in 0..self.len() {
            ret.coeffs[i] = self.coeffs[self.coeffs.len() - i - 1]
        }

        Ok(ret)
    }

    pub fn mul_fft(&self, multiplier: &Self, output_len: usize) -> Result<Self, String> {
        let length = next_power_of_two(self.len() + multiplier.len() - 1);

        let scale = log2_pow2(length);
        let fft_settings = FsFFTSettings::new(scale).unwrap();

        let a_pad = self.pad(length);
        let b_pad = multiplier.pad(length);
        // Convert Poly to values
        let a_fft = fft_settings.fft_fr(&a_pad.coeffs, false).unwrap();
        let b_fft = fft_settings.fft_fr(&b_pad.coeffs, false).unwrap();

        // Multiply two value ranges
        let mut ab_fft = vec![FsFr::default(); length];
        for i in 0..length {
            ab_fft[i] = a_fft[i].mul(&b_fft[i]);
        }

        // Convert value range multiplication to a resulting polynomial
        let ab = fft_settings.fft_fr(&ab_fft, true).unwrap();

        let mut ret = FsPoly {
            coeffs: vec![FsFr::zero(); output_len],
        };

        let range = ..min_u64(output_len, length);
        ret.coeffs[range].clone_from_slice(&ab[range]);

        Ok(ret)
    }

    pub fn mul(&mut self, multiplier: &Self, output_len: usize) -> Result<Self, String> {
        if self.len() < 64 || multiplier.len() < 64 || output_len < 128 {
            // Tunable parameter
            self.mul_direct(multiplier, output_len)
        } else {
            self.mul_fft(multiplier, output_len)
        }
    }
}

impl Clone for FsPoly {
    fn clone(&self) -> Self {
        FsPoly {
            coeffs: self.coeffs.clone(),
        }
    }
}

impl PolyRecover<FsFr, FsPoly, FsFFTSettings> for FsPoly {
    fn recover_poly_from_samples(samples: &[Option<FsFr>], fs: &FsFFTSettings) -> Self {
        let len_samples = samples.len();
        assert!(is_power_of_two(len_samples));

        let mut missing: Vec<usize> = Vec::new();

        for i in 0..len_samples {
            if samples[i].is_none() {
                missing.push(i);
            }
        }

        // Calculate `Z_r,I`
        let (zero_eval, mut zero_poly) = fs
            .zero_poly_via_multiplication(len_samples, &missing)
            .unwrap();

        for i in 0..len_samples {
            assert_eq!(samples[i].is_none(), zero_eval[i].is_zero());
        }

        let mut poly_evaluations_with_zero = FsPoly::default();

        // Construct E * Z_r,I: the loop makes the evaluation polynomial
        for i in 0..len_samples {
            if samples[i].is_none() {
                poly_evaluations_with_zero.coeffs.push(FsFr::zero());
            } else {
                poly_evaluations_with_zero
                    .coeffs
                    .push(samples[i].unwrap().mul(&zero_eval[i]));
            }
        }
        // Now inverse FFT so that poly_with_zero is (E * Z_r,I)(x) = (D * Z_r,I)(x)
        let mut poly_with_zero: FsPoly = FsPoly::default();
        poly_with_zero.coeffs = fs.fft_fr(&poly_evaluations_with_zero.coeffs, true).unwrap();

        // x -> k * x
        let len_zero_poly = zero_poly.coeffs.len();
        scale_poly(&mut poly_with_zero.coeffs, len_samples);
        scale_poly(&mut zero_poly.coeffs, len_zero_poly);

        // Q1 = (D * Z_r,I)(k * x)
        let scaled_poly_with_zero = poly_with_zero.coeffs;

        // Q2 = Z_r,I(k * x)
        let scaled_zero_poly = zero_poly.coeffs;

        // Polynomial division by convolution: Q3 = Q1 / Q2
        let eval_scaled_poly_with_zero: Vec<FsFr> =
            fs.fft_fr(&scaled_poly_with_zero, false).unwrap();
        let eval_scaled_zero_poly: Vec<FsFr> = fs.fft_fr(&scaled_zero_poly, false).unwrap();

        let mut eval_scaled_reconstructed_poly = FsPoly::default();
        eval_scaled_reconstructed_poly.coeffs = eval_scaled_poly_with_zero.clone();
        for i in 0..len_samples {
            eval_scaled_reconstructed_poly.coeffs[i] = eval_scaled_poly_with_zero[i]
                .div(&eval_scaled_zero_poly[i])
                .unwrap();
        }

        // The result of the division is D(k * x):
        let mut scaled_reconstructed_poly: Vec<FsFr> = fs
            .fft_fr(&eval_scaled_reconstructed_poly.coeffs, true)
            .unwrap();

        // k * x -> x
        unscale_poly(&mut scaled_reconstructed_poly, len_samples);

        // Finally we have D(x) which evaluates to our original data at the powers of roots of unity
        let reconstructed_poly = scaled_reconstructed_poly;

        // The evaluation polynomial for D(x) is the reconstructed data:
        let mut reconstr_poly = FsPoly::default();
        let reconstructed_data = fs.fft_fr(&reconstructed_poly, false).unwrap();

        // Check all is well
        for i in 0..len_samples {
            assert!(samples[i].is_none() || reconstructed_data[i].equals(&samples[i].unwrap()));
        }

        reconstr_poly.coeffs = reconstructed_data;
        reconstr_poly
    }
}

pub trait PolyStupidInterface<Coeff: Fr>: Clone {
    fn default() -> Self;

    fn new(size: usize) -> Result<Self, String>;

    fn get_coeff_at(&self, i: usize) -> Coeff;

    fn set_coeff_at(&mut self, i: usize, x: &Coeff);

    fn get_coeffs(&self) -> &[Coeff];

    fn len(&self) -> usize;

    fn eval(&self, x: &Coeff) -> Coeff;

    fn scale(&mut self);

    fn unscale(&mut self);

    fn inverse(&mut self, new_len: usize) -> Result<Self, String>;

    fn div(&mut self, x: &Self) -> Result<Self, String>;

    fn long_div(&mut self, x: &Self) -> Result<Self, String>;

    fn fast_div(&mut self, x: &Self) -> Result<Self, String>;

    fn mul_direct(&mut self, x: &Self, len: usize) -> Result<Self, String>;

    fn _poly_norm(&self) -> Self;

    fn poly_quotient_length(&self, divisor: &Self) -> usize;

    fn pad(&self, out_length: usize) -> Self;

    fn flip(&self) -> Result<Self, String>;

    fn mul_fft(&self, multiplier: &Self, output_len: usize) -> Result<Self, String>;

    fn mul(&mut self, multiplier: &Self, output_len: usize) -> Result<Self, String>;
}

pub struct StupidPoly {
    pub coeffs: Vec<FsFr>,
}

impl Clone for StupidPoly {
    fn clone(&self) -> Self {
        StupidPoly {
            coeffs: self.coeffs.clone(),
        }
    }
}

impl PolyStupidInterface<FsFr> for StupidPoly {
    fn default() -> Self {
        Self::new(0).unwrap()
    }

    fn new(size: usize) -> Result<Self, String> {
        Ok(Self {
            coeffs: vec![FsFr::default(); size],
        })
    }

    fn get_coeff_at(&self, i: usize) -> FsFr {
        self.coeffs[i]
    }

    fn set_coeff_at(&mut self, i: usize, x: &FsFr) {
        self.coeffs[i] = *x
    }

    fn get_coeffs(&self) -> &[FsFr] {
        &self.coeffs
    }

    fn len(&self) -> usize {
        self.coeffs.len()
    }

    fn eval(&self, x: &FsFr) -> FsFr {
        if self.coeffs.is_empty() {
            return FsFr::zero();
        } else if x.is_zero() {
            return self.coeffs[0];
        }

        let mut ret = self.coeffs[self.coeffs.len() - 1];
        let mut i = self.coeffs.len() - 2;
        loop {
            let temp = ret.mul(x);
            ret = temp.add(&self.coeffs[i]);

            if i == 0 {
                break;
            }
            i -= 1;
        }

        return ret;
    }

    fn scale(&mut self) {
        let scale_factor = FsFr::from_u64(SCALE_FACTOR);
        let inv_factor = scale_factor.inverse();

        let mut factor_power = FsFr::one();
        for i in 0..self.coeffs.len() {
            factor_power = factor_power.mul(&inv_factor);
            self.coeffs[i] = self.coeffs[i].mul(&factor_power);
        }
    }

    fn unscale(&mut self) {
        let scale_factor = FsFr::from_u64(SCALE_FACTOR);

        let mut factor_power = FsFr::one();
        for i in 0..self.coeffs.len() {
            factor_power = factor_power.mul(&scale_factor);
            self.coeffs[i] = self.coeffs[i].mul(&factor_power);
        }
    }

    // TODO: analyze how algo works
    fn inverse(&mut self, output_len: usize) -> Result<Self, String> {
        if output_len == 0 {
            return Err(String::from("Can't produce a zero-length result"));
        } else if self.coeffs.len() == 0 {
            return Err(String::from("Can't inverse a zero-length poly"));
        } else if self.coeffs[0].is_zero() {
            return Err(String::from(
                "First coefficient of polynomial mustn't be zero",
            ));
        }

        let mut ret = Self {
            coeffs: vec![FsFr::zero(); output_len],
        };
        // If the input polynomial is constant, the remainder of the series is zero
        if self.coeffs.len() == 1 {
            ret.coeffs[0] = self.coeffs[0].eucl_inverse();

            return Ok(ret);
        }

        let maxd = output_len - 1;

        // Max space for multiplications is (2 * length - 1)
        // Don't need the following as its recalculated inside
        // let scale: usize = log2_pow2(next_power_of_two(2 * output_len - 1));
        // let fft_settings = FsFFTSettings::new(scale).unwrap();

        // To store intermediate results

        // Base case for d == 0
        ret.coeffs[0] = self.coeffs[0].eucl_inverse();
        let mut d: usize = 0;
        let mut mask: usize = 1 << log2_u64(maxd);
        while mask != 0 {
            d = 2 * d + (if (maxd & mask) != 0 { 1 } else { 0 });
            mask >>= 1;

            // b.c -> tmp0 (we're using out for c)
            // tmp0.length = min_u64(d + 1, b->length + output->length - 1);
            let len_temp = min_u64(d + 1, self.len() + output_len - 1);
            let mut tmp0 = self.mul(&ret, len_temp).unwrap();

            // 2 - b.c -> tmp0
            for i in 0..tmp0.len() {
                tmp0.coeffs[i] = tmp0.coeffs[i].negate();
            }
            let fr_two = Fr::from_u64(2);
            tmp0.coeffs[0] = tmp0.coeffs[0].add(&fr_two);

            // c.(2 - b.c) -> tmp1;
            let tmp1 = ret.mul(&tmp0, d + 1).unwrap();

            for i in 0..tmp1.len() {
                ret.coeffs[i] = tmp1.coeffs[i];
            }
        }

        if d + 1 != output_len {
            return Err(String::from("D + 1 must be equal to output_len"));
        }

        Ok(ret)
    }

    fn div(&mut self, divisor: &Self) -> Result<Self, String> {
        return if divisor.len() >= self.len() || divisor.len() < 128 {
            // Tunable parameter
            self.long_div(&divisor)
        } else {
            self.fast_div(&divisor)
        };
    }

    fn long_div(&mut self, divisor: &Self) -> Result<Self, String> {
        if divisor.coeffs.len() == 0 {
            return Err(String::from("Can't divide by zero"));
        } else if divisor.coeffs[divisor.coeffs.len() - 1].is_zero() {
            return Err(String::from("Highest coefficient must be non-zero"));
        }

        let out_length = self.poly_quotient_length(&divisor);
        let mut out = Self {
            coeffs: vec![FsFr::default(); out_length],
        };
        if out_length == 0 {
            return Ok(out);
        }

        let mut a_pos = self.len() - 1;
        let b_pos = divisor.len() - 1;
        let mut diff = a_pos - b_pos;

        let mut a = vec![FsFr::default(); self.len()];
        for i in 0..a.len() {
            a[i] = self.coeffs[i];
        }

        while diff > 0 {
            out.coeffs[diff] = a[a_pos].div(&divisor.coeffs[b_pos]).unwrap();

            for i in 0..(b_pos + 1) {
                let tmp = out.coeffs[diff].mul(&divisor.coeffs[i]);
                let tmp = a[diff + i].sub(&tmp);
                a[diff + i] = tmp;
            }

            diff -= 1;
            a_pos -= 1;
        }

        out.coeffs[0] = a[a_pos].div(&divisor.coeffs[b_pos]).unwrap();

        Ok(out)
    }

    fn fast_div(&mut self, divisor: &Self) -> Result<Self, String> {
        if divisor.coeffs.len() == 0 {
            return Err(String::from("Cant divide by zero"));
        } else if divisor.coeffs[divisor.coeffs.len() - 1].is_zero() {
            return Err(String::from("Highest coefficient must be non-zero"));
        }

        let m: usize = self.len() - 1;
        let n: usize = divisor.len() - 1;

        // If the divisor is larger than the dividend, the result is zero-length
        if n > m {
            return Ok(Self { coeffs: Vec::new() });
        }

        // Special case for divisor.length == 1 (it's a constant)
        if divisor.len() == 1 {
            let mut out = Self {
                coeffs: vec![FsFr::zero(); self.len()],
            };
            for i in 0..out.len() {
                out.coeffs[i] = self.coeffs[i].div(&divisor.coeffs[0]).unwrap();
            }
            return Ok(out);
        }

        let mut a_flip = self.flip().unwrap();
        let mut b_flip = divisor.flip().unwrap();

        let inv_b_flip = b_flip.inverse(m - n + 1).unwrap();
        let q_flip = a_flip.mul(&inv_b_flip, m - n + 1).unwrap();

        let out = q_flip.flip().unwrap();
        Ok(out)
    }

    fn mul_direct(&mut self, multiplier: &Self, output_len: usize) -> Result<Self, String> {
        if self.len() == 0 || multiplier.len() == 0 {
            return Ok(Self::new(0).unwrap());
        }

        let a_degree = self.len() - 1;
        let b_degree = multiplier.len() - 1;

        let mut ret = Self {
            coeffs: vec![Fr::zero(); output_len],
        };

        // Truncate the output to the length of the output polynomial
        for i in 0..(a_degree + 1) {
            let mut j = 0;
            while (j <= b_degree) && ((i + j) < output_len) {
                let tmp = self.coeffs[i].mul(&multiplier.coeffs[j]);
                let tmp = ret.coeffs[i + j].add(&tmp);
                ret.coeffs[i + j] = tmp;

                j += 1;
            }
        }

        Ok(ret)
    }

    fn _poly_norm(&self) -> Self {
        let mut ret = self.clone();

        let mut temp_len: usize = ret.coeffs.len();
        while temp_len > 0 && ret.coeffs[temp_len - 1].is_zero() {
            temp_len -= 1;
        }

        if temp_len == 0 {
            ret.coeffs = Vec::new();
        } else {
            ret.coeffs = ret.coeffs[0..temp_len].to_vec();
        }

        ret
    }

    fn poly_quotient_length(&self, divisor: &Self) -> usize {
        return if self.len() >= divisor.len() {
            self.len() - divisor.len() + 1
        } else {
            0
        };
    }

    fn pad(&self, out_length: usize) -> Self {
        let mut ret = Self {
            coeffs: vec![FsFr::zero(); out_length],
        };

        for i in 0..min_u64(self.len(), out_length) {
            ret.coeffs[i] = self.coeffs[i];
        }

        ret
    }

    fn flip(&self) -> Result<Self, String> {
        let mut ret = Self {
            coeffs: vec![FsFr::default(); self.len()],
        };
        for i in 0..self.len() {
            ret.coeffs[i] = self.coeffs[self.coeffs.len() - i - 1]
        }

        Ok(ret)
    }

    fn mul_fft(&self, multiplier: &Self, output_len: usize) -> Result<Self, String> {
        let length = next_power_of_two(self.len() + multiplier.len() - 1);

        let scale = log2_pow2(length);
        let fft_settings = FsFFTSettings::new(scale).unwrap();

        let a_pad = self.pad(length);
        let b_pad = multiplier.pad(length);
        // Convert Poly to values
        let a_fft = fft_settings.fft_fr(&a_pad.coeffs, false).unwrap();
        let b_fft = fft_settings.fft_fr(&b_pad.coeffs, false).unwrap();

        // Multiply two value ranges
        let mut ab_fft = vec![FsFr::default(); length];
        for i in 0..length {
            ab_fft[i] = a_fft[i].mul(&b_fft[i]);
        }

        // Convert value range multiplication to a resulting polynomial
        let ab = fft_settings.fft_fr(&ab_fft, true).unwrap();

        let mut ret = Self {
            coeffs: vec![FsFr::zero(); output_len],
        };
        for i in 0..min_u64(output_len, length) {
            ret.coeffs[i] = ab[i];
        }

        Ok(ret)
    }

    fn mul(&mut self, multiplier: &Self, output_len: usize) -> Result<Self, String> {
        return if self.len() < 64 || multiplier.len() < 64 || output_len < 128 {
            // Tunable parameter
            self.mul_direct(multiplier, output_len)
        } else {
            self.mul_fft(multiplier, output_len)
        };
    }
}
