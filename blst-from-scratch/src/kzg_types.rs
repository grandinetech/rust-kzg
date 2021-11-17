use crate::consts::{expand_root_of_unity, G1_GENERATOR, G1_NEGATIVE_GENERATOR, G1_IDENTITY, G2_GENERATOR, G2_NEGATIVE_GENERATOR, SCALE2_ROOT_OF_UNITY, SCALE_FACTOR};
use crate::utils::{is_power_of_two, log_2_byte};
use blst::{blst_fp, blst_fp2, blst_fr, blst_fr_add, blst_fr_cneg, blst_fr_eucl_inverse, blst_fr_from_uint64, blst_fr_inverse, blst_fr_mul, blst_fr_sqr, blst_fr_sub, blst_p1, blst_p2, blst_uint64_from_fr, blst_fr_from_scalar, blst_scalar_from_fr, blst_p1_add_or_double, blst_p1_cneg, blst_p1_mult, blst_p1_is_equal, blst_scalar, blst_p2_mult, blst_p2_cneg, blst_p2_add_or_double, blst_p2_is_equal, blst_p2_double, blst_p1_is_inf, blst_p1_double};
use kzg::{FFTSettings, Fr, Poly, G1, G1Mul, FFTFr, G2, G2Mul, FK20SingleSettings, FK20MultiSettings, KZGSettings, FFTG1, PolyRecover, ZeroPoly};
use crate::bytes::reverse_bit_order;
use crate::kzg_proofs::{g1_linear_combination, pairings_verify};
use crate::utils::{log2_pow2, log2_u64, min_u64, next_power_of_two};
use crate::recovery::{scale_poly, unscale_poly};

pub struct FsFr(pub blst::blst_fr);

impl Fr for FsFr {
    fn default() -> Self {
        Self(blst_fr::default())
    }

    fn null() -> Self {
        Self::from_u64_arr(&[u64::MAX, u64::MAX, u64::MAX, u64::MAX])
    }

    fn zero() -> Self {
        Self::from_u64(0)
    }

    fn one() -> Self {
        Self::from_u64(1)
    }

    fn rand() -> Self {
        let val: [u64; 4] =
            [
                rand::random(),
                rand::random(),
                rand::random(),
                rand::random(),
            ];
        let mut ret = Self::default();
        unsafe {
            blst_fr_from_uint64(&mut ret.0, val.as_ptr());
        }

        ret
    }

    fn from_u64_arr(u: &[u64; 4]) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_from_uint64(&mut ret.0, u.as_ptr());
        }

        ret
    }

    fn from_u64(val: u64) -> Self {
        Self::from_u64_arr(&[val, 0, 0, 0])
    }

    fn to_u64_arr(&self) -> [u64; 4] {
        let mut val: [u64; 4] = [0; 4];
        unsafe {
            blst_uint64_from_fr(val.as_mut_ptr(), &self.0);
        }

        val
    }

    fn is_one(&self) -> bool {
        let mut val: [u64; 4] = [0; 4];
        unsafe {
            blst_uint64_from_fr(val.as_mut_ptr(), &self.0);
        }
        return val[0] == 1 && val[1] == 0 && val[2] == 0 && val[3] == 0;
    }

    fn is_zero(&self) -> bool {
        let mut val: [u64; 4] = [0; 4];
        unsafe {
            blst_uint64_from_fr(val.as_mut_ptr(), &self.0);
        }
        return val[0] == 0 && val[1] == 0 && val[2] == 0 && val[3] == 0;
    }

    fn is_null(&self) -> bool {
        let null = Self::null();
        return null.equals(self);
    }

    fn sqr(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_sqr(&mut ret.0, &self.0);
        }

        ret
    }

    fn mul(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_mul(&mut ret.0, &self.0, &b.0);
        }

        ret
    }

    fn div(&self, b: &Self) -> Result<Self, String> {
        let tmp = b.eucl_inverse();
        let out = self.mul(&tmp);

        Ok(out)
    }

    fn add(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_add(&mut ret.0, &self.0, &b.0);
        }

        ret
    }

    fn sub(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_sub(&mut ret.0, &self.0, &b.0);
        }

        ret
    }

    fn eucl_inverse(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_eucl_inverse(&mut ret.0, &self.0);
        }

        return ret;
    }

    fn negate(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_cneg(&mut ret.0, &self.0, true);
        }

        ret
    }

    fn inverse(&self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_fr_inverse(&mut ret.0, &self.0);
        }

        ret
    }

    fn pow(&self, n: usize) -> Self {
        let mut out = Self::one();

        let mut temp = self.clone();
        let mut n = n;
        loop {
            if (n & 1) == 1 {
                out = out.mul(&temp);
            }
            n = n >> 1;
            if n == 0 {
                break;
            }

            temp = temp.sqr();
        }

        out
    }

    fn equals(&self, b: &Self) -> bool {
        let mut val_a: [u64; 4] = [0; 4];
        let mut val_b: [u64; 4] = [0; 4];

        unsafe {
            blst_uint64_from_fr(val_a.as_mut_ptr(), &self.0);
            blst_uint64_from_fr(val_b.as_mut_ptr(), &b.0);
        }

        return val_a[0] == val_b[0]
            && val_a[1] == val_b[1]
            && val_a[2] == val_b[2]
            && val_a[3] == val_b[3];
    }
}

impl FsFr {
    pub fn to_scalar(&self) -> [u8; 32usize] {
        let mut scalar = blst_scalar::default();
        unsafe {
            blst_scalar_from_fr(&mut scalar, &self.0);
        }

        scalar.b
    }

    pub fn from_scalar(scalar: [u8; 32usize]) -> Self {
        let mut bls_scalar = blst_scalar::default();
        bls_scalar.b = scalar;
        let mut fr = blst_fr::default();
        unsafe {
            blst_fr_from_scalar(&mut fr, &bls_scalar);
        }
        let mut ret = Self::default();
        ret.0 = fr;
        ret
    }
}

impl Clone for FsFr {
    fn clone(&self) -> Self {
        FsFr(self.0.clone())
    }
}

impl Copy for FsFr {}

pub struct FsG1(pub blst::blst_p1);

impl FsG1 {
    pub(crate) const fn from_xyz(x: blst_fp, y: blst_fp, z: blst_fp) -> Self {
        FsG1(blst_p1 { x, y, z })
    }
}

impl G1 for FsG1 {
    fn default() -> Self {
        Self(blst_p1::default())
    }

    fn identity() -> Self {
        G1_IDENTITY
    }

    fn generator() -> Self {
        G1_GENERATOR
    }

    fn negative_generator() -> Self {
        G1_NEGATIVE_GENERATOR
    }

    fn rand() -> Self {
        let result: FsG1 = G1_GENERATOR;
        result.mul(&FsFr::rand())
    }

    fn add_or_dbl(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_p1_add_or_double(&mut ret.0, &self.0, &b.0);
        }
        ret
    }

    fn is_inf(&self) -> bool {
        unsafe {
            return blst_p1_is_inf(&self.0);
        }
    }

    fn dbl(&self) -> Self {
        let mut result = blst_p1::default();
        unsafe {
            blst_p1_double(&mut result, &self.0);
        }
        Self(result)
    }

    fn sub(&self, b: &Self) -> Self {
        let mut b_negative: FsG1 = *b;
        let mut ret = Self::default();
        unsafe {
            blst_p1_cneg(&mut b_negative.0, true);
            blst_p1_add_or_double(&mut ret.0, &self.0, &b_negative.0);
            ret
        }
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe {
            return blst_p1_is_equal(&self.0, &b.0);
        }
    }

    fn div(&self, _b: &Self) -> Result<Self, String> {
        todo!()
    }
}

impl Clone for FsG1 {
    fn clone(&self) -> Self {
        FsG1(self.0.clone())
    }
}

impl Copy for FsG1 {}

pub struct FsG2(pub blst::blst_p2);

impl G2Mul<FsFr> for FsG2 {
    fn mul(&self, b: &FsFr) -> Self {
        let mut result = blst_p2::default();
        let mut scalar = blst_scalar::default();
        unsafe {
            blst_scalar_from_fr(&mut scalar, &b.0);
            blst_p2_mult(&mut result, &self.0, scalar.b.as_ptr(), 8 * std::mem::size_of::<blst_scalar>());
        }
        Self(result)
    }
}

impl G2 for FsG2 {
    fn default() -> Self {
        Self(blst_p2::default())
    }

    fn generator() -> Self {
        G2_GENERATOR
    }

    fn negative_generator() -> Self {
        G2_NEGATIVE_GENERATOR
    }

    fn add_or_dbl(&mut self, b: &Self) -> Self {
        let mut result = blst_p2::default();
        unsafe {
            blst_p2_add_or_double(&mut result, &self.0, &b.0);
        }
        Self(result)
    }

    fn dbl(&self) -> Self {
        let mut result = blst_p2::default();
        unsafe {
            blst_p2_double(&mut result, &self.0);
        }
        Self(result)
    }

    fn sub(&self, b: &Self) -> Self {
        let mut bneg: blst_p2 = b.0;
        let mut result = blst_p2::default();
        unsafe {
            blst_p2_cneg(&mut bneg, true);
            blst_p2_add_or_double(&mut result, &self.0, &bneg);
        }
        Self(result)
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe {
            return blst_p2_is_equal(&self.0, &b.0);
        }
    }
}

impl G1Mul<FsFr> for FsG1 {
    fn mul(&self, b: &FsFr) -> Self {
        let mut scalar: blst_scalar = blst_scalar::default();
        unsafe {
            blst_scalar_from_fr(&mut scalar, &b.0);
        }

        // Count the number of bytes to be multiplied.
        let mut i = scalar.b.len();
        while i != 0 && scalar.b[i - 1] == 0 {
            i -= 1;
        }

        let mut result = Self::default();
        if i == 0 {
            return G1_IDENTITY;
        } else if i == 1 && scalar.b[0] == 1 {
            return *self;
        } else {
            // Count the number of bits to be multiplied.
            unsafe {
                blst_p1_mult(
                    &mut result.0,
                    &self.0,
                    &(scalar.b[0]),
                    8 * i - 7 + log_2_byte(scalar.b[i - 1]),
                );
            }
        }
        result
    }
}

impl FsG2 {
    pub(crate) fn _from_xyz(x: blst_fp2, y: blst_fp2, z: blst_fp2) -> Self {
        FsG2(blst_p2 { x, y, z })
    }

    pub fn default() -> Self {
        Self(blst_p2::default())
    }
}

impl Clone for FsG2 {
    fn clone(&self) -> Self {
        FsG2(self.0.clone())
    }
}

impl Copy for FsG2 {}

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
        if self.coeffs.len() == 0 {
            return FsFr::zero();
        } else if x.is_zero() {
            return self.coeffs[0].clone();
        }

        let mut ret = self.coeffs[self.coeffs.len() - 1].clone();
        let mut i = self.coeffs.len() - 2;
        loop {
            let temp = ret.mul(&x);
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
            return Err(String::from("First coefficient of polynomial mustn't be zero"));
        }

        let mut ret = FsPoly { coeffs: vec![FsFr::zero(); output_len] };
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

    fn div(&self, divisor: &Self) -> Result<Self, String> {
        return if divisor.len() >= self.len() || divisor.len() < 128 { // Tunable parameter
            self.div_long(&divisor)
        } else {
            self.div_fast(&divisor)
        };
    }

    fn div_long(&self, divisor: &Self) -> Result<Self, String> {
        if divisor.coeffs.len() == 0 {
            return Err(String::from("Can't divide by zero"));
        } else if divisor.coeffs[divisor.coeffs.len() - 1].is_zero() {
            return Err(String::from("Highest coefficient must be non-zero"));
        }

        let out_length = self.poly_quotient_length(&divisor);
        let mut out: FsPoly = FsPoly { coeffs: vec![FsFr::default(); out_length] };
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

    fn div_fast(&self, divisor: &Self) -> Result<Self, String> {
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
            let mut out = FsPoly { coeffs: vec![FsFr::zero(); self.len()] };
            for i in 0..out.len() {
                out.coeffs[i] = self.coeffs[i].div(&divisor.coeffs[0]).unwrap();
            }
            return Ok(out);
        }

        let a_flip = self.flip().unwrap();
        let mut b_flip = divisor.flip().unwrap();

        let inv_b_flip = b_flip.inverse(m - n + 1).unwrap();
        let q_flip = a_flip.mul(&inv_b_flip, m - n + 1).unwrap();

        let out = q_flip.flip().unwrap();
        Ok(out)
    }

    fn mul(&self, multiplier: &Self, output_len: usize) -> Result<Self, String> {
        return if self.len() < 64 || multiplier.len() < 64 || output_len < 128 { // Tunable parameter
            self.mul_direct(multiplier, output_len)
        } else {
            self.mul_fft(multiplier, output_len)
        };
    }

    fn mul_direct(&self, multiplier: &Self, output_len: usize) -> Result<Self, String> {
        if self.len() == 0 || multiplier.len() == 0 {
            return Ok(FsPoly::new(0).unwrap());
        }

        let a_degree = self.len() - 1;
        let b_degree = multiplier.len() - 1;

        let mut ret = FsPoly { coeffs: vec![Fr::zero(); output_len] };

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

        let mut ret = FsPoly { coeffs: vec![FsFr::zero(); output_len] };
        for i in 0..min_u64(output_len, length) {
            ret.coeffs[i] = ab[i];
        }

        Ok(ret)
    }

    fn flip(&self) -> Result<FsPoly, String> {
        let mut ret = FsPoly { coeffs: vec![FsFr::default(); self.len()] };
        for i in 0..self.len() {
            ret.coeffs[i] = self.coeffs[self.coeffs.len() - i - 1]
        }

        Ok(ret)
    }

    fn pad(&self, out_length: usize) -> Self {
        let mut ret = Self { coeffs: vec![FsFr::zero(); out_length] };

        for i in 0..min_u64(self.len(), out_length) {
            ret.coeffs[i] = self.coeffs[i];
        }

        ret
    }
}

impl FsPoly {
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
        return if self.len() >= divisor.len() { self.len() - divisor.len() + 1 } else { 0 };
    }
}

impl Clone for FsPoly {
    fn clone(&self) -> Self {
        FsPoly {
            coeffs: self.coeffs.clone(),
        }
    }
}

pub struct FsFFTSettings {
    pub max_width: usize,
    pub root_of_unity: FsFr,
    pub expanded_roots_of_unity: Vec<FsFr>,
    pub reverse_roots_of_unity: Vec<FsFr>,
}

impl FFTSettings<FsFr> for FsFFTSettings {
    fn default() -> Self {
        Self::new(0).unwrap()
    }

    /// Create FFTSettings with roots of unity for a selected scale. Resulting roots will have a magnitude of 2 ^ max_scale.
    fn new(scale: usize) -> Result<FsFFTSettings, String> {
        if scale >= SCALE2_ROOT_OF_UNITY.len() {
            return Err(String::from(
                "Scale is expected to be within root of unity matrix row size",
            ));
        }

        // max_width = 2 ^ max_scale
        let max_width: usize = 1 << scale;
        let root_of_unity = FsFr::from_u64_arr(&SCALE2_ROOT_OF_UNITY[scale]);

        // create max_width of roots & store them reversed as well
        let expanded_roots_of_unity = expand_root_of_unity(&root_of_unity, max_width).unwrap();
        let mut reverse_roots_of_unity = expanded_roots_of_unity.clone();
        reverse_roots_of_unity.reverse();

        Ok(FsFFTSettings {
            max_width,
            root_of_unity,
            expanded_roots_of_unity,
            reverse_roots_of_unity,
        })
    }

    fn get_max_width(&self) -> usize {
        self.max_width
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> FsFr {
        self.expanded_roots_of_unity[i]
    }

    fn get_expanded_roots_of_unity(&self) -> &[FsFr] {
        &self.expanded_roots_of_unity
    }

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> FsFr {
        self.reverse_roots_of_unity[i]
    }

    fn get_reversed_roots_of_unity(&self) -> &[FsFr] {
        &self.reverse_roots_of_unity
    }
}

impl Clone for FsFFTSettings {
    fn clone(&self) -> Self {
        let mut output = FsFFTSettings::new(0).unwrap();
        output.max_width = self.max_width;
        output.root_of_unity = self.root_of_unity.clone();
        output.expanded_roots_of_unity = self.expanded_roots_of_unity.clone();
        output.reverse_roots_of_unity = self.reverse_roots_of_unity.clone();
        output
    }
}

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
        let output = Self {
            secret_g1: Vec::new(),
            secret_g2: Vec::new(),
            fs: FsFFTSettings::default(),
        };
        output
    }

    fn new(
        secret_g1: &Vec<FsG1>,
        secret_g2: &Vec<FsG2>,
        length: usize,
        fft_settings: &FsFFTSettings,
    ) -> Result<Self, String> {
        let mut kzg_settings = Self::default();

        if secret_g1.len() < fft_settings.max_width {
            return Err(String::from("secret_g1 must have a length equal to or greater than fft_settings roots"));
        } else if secret_g2.len() < fft_settings.max_width {
            return Err(String::from("secret_g2 must have a length equal to or greater than fft_settings roots"));
        } else if length < fft_settings.max_width {
            return Err(String::from("length must be equal to or greater than number of fft_settings roots"));
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
        g1_linear_combination(
            &mut out,
            &self.secret_g1,
            &poly.coeffs,
            poly.coeffs.len(),
        );

        Ok(out)
    }

    fn compute_proof_single(&self, p: &FsPoly, x: &FsFr) -> Result<FsG1, String> {
        self.compute_proof_multi(p, x, 1)
    }

    fn check_proof_single(&self, com: &FsG1, proof: &FsG1, x: &FsFr, y: &FsFr) -> Result<bool, String> {
        let x_g2: FsG2 = G2_GENERATOR.mul(x);
        let s_minus_x: FsG2 = self.secret_g2[1].sub(&x_g2);
        let y_g1 = G1_GENERATOR.mul(y);
        let commitment_minus_y: FsG1 = g1_sub(com, &y_g1);

        Ok(pairings_verify(&commitment_minus_y, &G2_GENERATOR, proof, &s_minus_x))
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

        // Calculate q = p / (x^n - x0^n)
        let q = p.div(&divisor).unwrap();

        let ret = self.commit_to_poly(&q).unwrap();

        Ok(ret)
    }

    fn check_proof_multi(&self, com: &FsG1, proof: &FsG1, x: &FsFr, ys: &Vec<FsFr>, n: usize) -> Result<bool, String> {
        if !is_power_of_two(n) {
            return Err(String::from("n is not a power of two"));
        }

        // Interpolate at a coset.
        let mut interp = FsPoly { coeffs: self.fs.fft_fr(ys, true)? };

        let inv_x = x.inverse(); // Not euclidean?
        let mut inv_x_pow = inv_x.clone();
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


pub struct FsFK20SingleSettings {
    pub kzg_settings: FsKZGSettings,
    pub x_ext_fft: Vec<FsG1>,
}

impl Clone for FsFK20SingleSettings {
    fn clone(&self) -> Self {
        Self {
            kzg_settings: self.kzg_settings.clone(),
            x_ext_fft: self.x_ext_fft.clone(),
        }
    }
}

impl FK20SingleSettings<FsFr, FsG1, FsG2, FsFFTSettings, FsPoly, FsKZGSettings> for FsFK20SingleSettings {
    fn default() -> Self {
        Self {
            kzg_settings: FsKZGSettings::default(),
            x_ext_fft: vec![],
        }
    }

    fn new(kzg_settings: &FsKZGSettings, n2: usize) -> Result<Self, String> {
        let n = n2 / 2;

        if n2 > kzg_settings.fs.max_width {
            return Err(String::from("n2 must be less than or equal to kzg settings max width"));
        } else if !is_power_of_two(n2) {
            return Err(String::from("n2 must be a power of two"));
        } else if n2 < 2 {
            return Err(String::from("n2 must be greater than or equal to 2"));
        }

        let mut x = Vec::new();
        for i in 0..n - 1 {
            x.push(kzg_settings.secret_g1[n - 2 - i]);
        }
        x.push(FsG1::identity());

        let x_ext_fft = kzg_settings.fs.toeplitz_part_1(&x);
        let kzg_settings = kzg_settings.clone();

        let ret = Self {
            kzg_settings,
            x_ext_fft,
        };

        Ok(ret)
    }

    fn data_availability(&self, p: &FsPoly) -> Result<Vec<FsG1>, String> {
        let n = p.len();
        let n2 = n * 2;

        if n2 > self.kzg_settings.fs.max_width {
            return Err(String::from("n2 must be less than or equal to kzg settings max width"));
        } else if !is_power_of_two(n2) {
            return Err(String::from("n2 must be a power of two"));
        }

        let mut ret = self.data_availability_optimized(p).unwrap();
        reverse_bit_order(&mut ret);

        Ok(ret)
    }

    fn data_availability_optimized(&self, p: &FsPoly) -> Result<Vec<FsG1>, String> {
        let n = p.len();
        let n2 = n * 2;

        if n2 > self.kzg_settings.fs.max_width {
            return Err(String::from("n2 must be less than or equal to kzg settings max width"));
        } else if !is_power_of_two(n2) {
            return Err(String::from("n2 must be a power of two"));
        }

        let toeplitz_coeffs = p.toeplitz_coeffs_step();

        let h_ext_fft = self.kzg_settings.fs.toeplitz_part_2(&toeplitz_coeffs, &self.x_ext_fft);

        let h = self.kzg_settings.fs.toeplitz_part_3(&h_ext_fft);

        let ret = self.kzg_settings.fs.fft_g1(&h, false).unwrap();

        Ok(ret)
    }
}

pub struct FsFK20MultiSettings {
    pub kzg_settings: FsKZGSettings,
    pub chunk_len: usize,
    pub x_ext_fft_files: Vec<Vec<FsG1>>,
}

impl Clone for FsFK20MultiSettings {
    fn clone(&self) -> Self {
        Self {
            kzg_settings: self.kzg_settings.clone(),
            chunk_len: self.chunk_len,
            x_ext_fft_files: self.x_ext_fft_files.clone(),
        }
    }
}

impl FK20MultiSettings<FsFr, FsG1, FsG2, FsFFTSettings, FsPoly, FsKZGSettings> for FsFK20MultiSettings {
    fn default() -> Self {
        Self {
            kzg_settings: FsKZGSettings::default(),
            chunk_len: 1,
            x_ext_fft_files: vec![],
        }
    }

    fn new(ks: &FsKZGSettings, n2: usize, chunk_len: usize) -> Result<Self, String> {
        if n2 > ks.fs.max_width {
            return Err(String::from("n2 must be less than or equal to kzg settings max width"));
        } else if !is_power_of_two(n2) {
            return Err(String::from("n2 must be a power of two"));
        } else if n2 < 2 {
            return Err(String::from("n2 must be greater than or equal to 2"));
        } else if chunk_len > n2 / 2 {
            return Err(String::from("chunk_len must be greater or equal to n2 / 2"));
        } else if !is_power_of_two(chunk_len) {
            return Err(String::from("chunk_len must be a power of two"));
        }

        let n = n2 / 2;
        let k = n / chunk_len;

        let mut ext_fft_files = Vec::new();

        for offset in 0..chunk_len {
            let mut x = Vec::new();

            let mut start = 0;
            if n >= chunk_len + 1 + offset {
                start = n - chunk_len - 1 - offset;
            }

            let mut i = 0;
            let mut j = start;

            while i + 1 < k {
                x.push(ks.secret_g1[j as usize]);

                i += 1;

                if j >= chunk_len {
                    j -= chunk_len;
                } else {
                    j = 0;
                }
            }
            x.push(FsG1::identity());

            let ext_fft_file = ks.fs.toeplitz_part_1(&x);
            ext_fft_files.push(ext_fft_file);
        }

        let ret = Self {
            kzg_settings: ks.clone(),
            chunk_len,
            x_ext_fft_files: ext_fft_files,
        };

        Ok(ret)
    }

    fn data_availability(&self, p: &FsPoly) -> Result<Vec<FsG1>, String> {
        let n = p.len();
        let n2 = n * 2;

        if n2 > self.kzg_settings.fs.max_width {
            return Err(String::from("n2 must be less than or equal to kzg settings max width"));
        } else if !is_power_of_two(n2) {
            return Err(String::from("n2 must be a power of two"));
        }

        let mut ret = self.data_availability_optimized(p).unwrap();
        reverse_bit_order(&mut ret);

        Ok(ret)
    }

    fn data_availability_optimized(&self, p: &FsPoly) -> Result<Vec<FsG1>, String> {
        let n = p.len();
        let n2 = n * 2;

        if n2 > self.kzg_settings.fs.max_width {
            return Err(String::from("n2 must be less than or equal to kzg settings max width"));
        } else if !is_power_of_two(n2) {
            return Err(String::from("n2 must be a power of two"));
        }

        let n = n2 / 2;
        let k = n / self.chunk_len;
        let k2 = k * 2;

        let mut h_ext_fft = vec![FsG1::identity(); k2];

        for i in 0..self.chunk_len {
            let toeplitz_coeffs = p.toeplitz_coeffs_stride(i, self.chunk_len);
            let h_ext_fft_file = self.kzg_settings.fs.toeplitz_part_2(&toeplitz_coeffs, &self.x_ext_fft_files[i]);

            for j in 0..k2 {
                h_ext_fft[j] = h_ext_fft[j].add_or_dbl(&h_ext_fft_file[j]);
            }
        }

        let mut h = self.kzg_settings.fs.toeplitz_part_3(&h_ext_fft);

        for i in k..k2 {
            h[i] = FsG1::identity();
        }

        let ret = self.kzg_settings.fs.fft_g1(&h, false).unwrap();

        Ok(ret)
    }
}

impl PolyRecover<FsFr, FsPoly, FsFFTSettings> for FsPoly {
    fn recover_poly_from_samples(samples: &[Option<FsFr>], fs: FsFFTSettings) -> Self {

        let len_samples = samples.len();
        assert!(is_power_of_two(len_samples));

        let mut missing: Vec<usize> = Vec::new();
        for i in 0..len_samples {
            if samples[i].is_none() {
                missing.push(i);
            }
        }

        // Calculate `Z_r,I`
        let (zero_eval, mut zero_poly) = fs.zero_poly_via_multiplication(len_samples, &missing).unwrap();

        for i in 0..len_samples {
            assert_eq!(samples[i].is_none(), zero_eval[i].is_zero());
        }

        let mut poly_evaluations_with_zero = FsPoly::default();

        // Construct E * Z_r,I: the loop makes the evaluation polynomial
        for i in 0..len_samples {
            if samples[i].is_none() {
                poly_evaluations_with_zero.coeffs.push(FsFr::zero());
            } else {
                poly_evaluations_with_zero.coeffs.push(samples[i].unwrap().mul(&zero_eval[i]));
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
        let eval_scaled_poly_with_zero: Vec<FsFr> = fs.fft_fr(&scaled_poly_with_zero, false).unwrap();
        let eval_scaled_zero_poly: Vec<FsFr> = fs.fft_fr(&scaled_zero_poly, false).unwrap();

        let mut eval_scaled_reconstructed_poly = FsPoly::default();
        eval_scaled_reconstructed_poly.coeffs = eval_scaled_poly_with_zero.clone();
        for i in 0..len_samples {
            eval_scaled_reconstructed_poly.coeffs[i] = eval_scaled_poly_with_zero[i].div(&eval_scaled_zero_poly[i]).unwrap();
        }

        // The result of the division is D(k * x):
        let mut scaled_reconstructed_poly: Vec<FsFr> = fs.fft_fr(&eval_scaled_reconstructed_poly.coeffs, true).unwrap();

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