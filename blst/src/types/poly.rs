extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use kzg::{FFTFr, FFTSettings, FFTSettingsPoly, Fr, Poly};

use crate::consts::SCALE_FACTOR;
use crate::types::fft_settings::FsFFTSettings;
use crate::types::fr::FsFr;
use crate::utils::{log2_pow2, log2_u64};

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct FsPoly {
    pub coeffs: Vec<FsFr>,
}

impl Poly<FsFr> for FsPoly {
    fn new(size: usize) -> Self {
        Self {
            coeffs: vec![FsFr::default(); size],
        }
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
            d = 2 * d + usize::from((maxd & mask) != 0);
            mask >>= 1;

            // b.c -> tmp0 (we're using out for c)
            // tmp0.length = min_u64(d + 1, b->length + output->length - 1);
            let len_temp = (d + 1).min(self.len() + output_len - 1);
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
        if divisor.len() >= self.len() || divisor.len() < 128 {
            // Tunable parameter
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
        if out_length == 0 {
            return Ok(FsPoly { coeffs: vec![] });
        }

        // Special case for divisor.len() == 2
        if divisor.len() == 2 {
            let divisor_0 = divisor.coeffs[0];
            let divisor_1 = divisor.coeffs[1];

            let mut out_coeffs = Vec::from(&self.coeffs[1..]);
            for i in (1..out_length).rev() {
                out_coeffs[i] = out_coeffs[i].div(&divisor_1).unwrap();

                let tmp = out_coeffs[i].mul(&divisor_0);
                out_coeffs[i - 1] = out_coeffs[i - 1].sub(&tmp);
            }

            out_coeffs[0] = out_coeffs[0].div(&divisor_1).unwrap();

            Ok(FsPoly { coeffs: out_coeffs })
        } else {
            let mut out: FsPoly = FsPoly {
                coeffs: vec![FsFr::default(); out_length],
            };

            let mut a_pos = self.len() - 1;
            let b_pos = divisor.len() - 1;
            let mut diff = a_pos - b_pos;

            let mut a = self.coeffs.clone();

            while diff > 0 {
                out.coeffs[diff] = a[a_pos].div(&divisor.coeffs[b_pos]).unwrap();

                for i in 0..(b_pos + 1) {
                    let tmp = out.coeffs[diff].mul(&divisor.coeffs[i]);
                    a[diff + i] = a[diff + i].sub(&tmp);
                }

                diff -= 1;
                a_pos -= 1;
            }

            out.coeffs[0] = a[a_pos].div(&divisor.coeffs[b_pos]).unwrap();
            Ok(out)
        }
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
            return Ok(FsPoly::new(0));
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
    fn poly_mul_fft(
        a: &FsPoly,
        b: &FsPoly,
        len: usize,
        _fs: Option<&FsFFTSettings>,
    ) -> Result<FsPoly, String> {
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

        for i in 0..self.len().min(out_length) {
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
        let length = (self.len() + multiplier.len() - 1).next_power_of_two();

        let scale = log2_pow2(length);
        let fft_settings = FsFFTSettings::new(scale).unwrap();

        let a_pad = self.pad(length);
        let b_pad = multiplier.pad(length);

        let a_fft: Vec<FsFr>;
        let b_fft: Vec<FsFr>;

        #[cfg(feature = "parallel")]
        {
            if length > 1024 {
                let mut a_fft_temp = vec![];
                let mut b_fft_temp = vec![];

                rayon::join(
                    || a_fft_temp = fft_settings.fft_fr(&a_pad.coeffs, false).unwrap(),
                    || b_fft_temp = fft_settings.fft_fr(&b_pad.coeffs, false).unwrap(),
                );

                a_fft = a_fft_temp;
                b_fft = b_fft_temp;
            } else {
                a_fft = fft_settings.fft_fr(&a_pad.coeffs, false).unwrap();
                b_fft = fft_settings.fft_fr(&b_pad.coeffs, false).unwrap();
            }
        }

        #[cfg(not(feature = "parallel"))]
        {
            // Convert Poly to values
            a_fft = fft_settings.fft_fr(&a_pad.coeffs, false).unwrap();
            b_fft = fft_settings.fft_fr(&b_pad.coeffs, false).unwrap();
        }

        // Multiply two value ranges
        let mut ab_fft = a_fft;
        ab_fft.iter_mut().zip(b_fft).for_each(|(a, b)| {
            *a = a.mul(&b);
        });

        // Convert value range multiplication to a resulting polynomial
        let ab = fft_settings.fft_fr(&ab_fft, true).unwrap();
        drop(ab_fft);

        let mut ret = FsPoly {
            coeffs: vec![FsFr::zero(); output_len],
        };

        let range = ..output_len.min(length);
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
