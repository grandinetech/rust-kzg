use kzg::{FFTFr, FFTSettings, FFTSettingsPoly, Fr, Poly, PolyRecover, ZeroPoly};

use crate::consts::SCALE_FACTOR;
use crate::recovery::{scale_poly, unscale_poly};
use crate::types::fft_settings::FsFFTSettings;
use crate::types::fr::FsFr;
use crate::utils::is_power_of_two;
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
        } else if self.coeffs.is_empty() {
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
        if divisor.len() >= self.len() || divisor.len() < 128 { // Tunable parameter
            self.long_div(divisor)
        } else {
            self.fast_div(divisor)
        }
    }

    fn long_div(&mut self, divisor: &Self) -> Result<Self, String> {
        if divisor.coeffs.is_empty() {
            return Err(String::from("Can't divide by zero"));
        } else if divisor.coeffs[divisor.coeffs.len() - 1].is_zero() {
            return Err(String::from("Highest coefficient must be non-zero"));
        }

        let out_length = self.poly_quotient_length(divisor);
        let mut out: FsPoly = FsPoly {
            coeffs: vec![FsFr::default(); out_length],
        };
        if out_length == 0 {
            return Ok(out);
        }

        let mut a_pos = self.len() - 1;
        let b_pos = divisor.len() - 1;
        let mut diff = a_pos - b_pos;

        let mut a = self.coeffs.clone();

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
        if divisor.coeffs.is_empty() {
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

impl FFTSettingsPoly<FsFr, FsPoly, FsFFTSettings> for FsFFTSettings {
    fn poly_mul_fft(a: &FsPoly, b: &FsPoly, len: usize, _fs: Option<&FsFFTSettings>) -> Result<FsPoly, String> {
        b.mul_fft(a, len)
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
    fn recover_poly_from_samples(samples: &[Option<FsFr>], fs: &FsFFTSettings) -> Result<Self, String> {
        let len_samples = samples.len();

        if !is_power_of_two(len_samples) {
            return Err(String::from("Samples must have a length that is a power of two"));
        }

        let mut missing: Vec<usize> = Vec::new();

        for (i, sample) in samples.iter().enumerate() {
            if sample.is_none() {
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
        Ok(reconstr_poly)
    }
}