use crate::consts::{expand_root_of_unity, SCALE2_ROOT_OF_UNITY, SCALE_FACTOR};
use blst::{
    blst_fp, blst_fp2, blst_fr, blst_fr_add, blst_fr_cneg, blst_fr_eucl_inverse,
    blst_fr_from_uint64, blst_fr_inverse, blst_fr_mul, blst_fr_sqr, blst_fr_sub, blst_p1, blst_p2,
    blst_uint64_from_fr,
};
use kzg::{FFTSettings, Fr, Poly, G1, FFTFr};
use crate::utils::{log2_pow2, log2_u64, min_u64, next_power_of_two};

pub struct FsFr(blst::blst_fr);

impl Fr for FsFr {
    fn default() -> Self {
        Self(blst_fr::default())
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
        todo!("double check implementation");
        //fr_t tmp = *a;
        let mut tmp = self.clone();

        //*out = fr_one;
        let mut out = Self::one();
        let mut n2 = n;

        unsafe {
            loop {
                if n2 & 1 == 1 {
                    blst_fr_mul(&mut out.0, &out.0, &tmp.0);
                }
                n2 = n2 >> 1;
                if n == 0 {
                    break;
                }
                blst_fr_sqr(&mut out.0, &tmp.0);
            }
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

    fn div(&self, b: &Self) -> Result<Self, String> {
        let tmp = b.eucl_inverse();
        let out = self.mul(&tmp);

        Ok(out)
    }

    fn destroy(&mut self) {}
}

impl Clone for FsFr {
    fn clone(&self) -> Self {
        FsFr(self.0.clone())
    }
}

impl Copy for FsFr {}

pub struct FsG1(blst::blst_p1);

impl FsG1 {
    pub(crate) fn from_xyz(x: blst_fp, y: blst_fp, z: blst_fp) -> Self {
        FsG1(blst_p1 { x, y, z })
    }
}

impl G1 for FsG1 {
    fn default() -> Self {
        todo!()
    }

    fn rand() -> Self {
        todo!()
    }

    fn add_or_double(&mut self, b: &Self) -> Self {
        todo!()
    }

    fn equals(&self, b: &Self) -> bool {
        todo!()
    }

    fn destroy(&mut self) {
        todo!()
    }
}

impl Clone for FsG1 {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl Copy for FsG1 {}

pub struct FsG2(blst::blst_p2);

impl FsG2 {
    pub(crate) fn from_xyz(x: blst_fp2, y: blst_fp2, z: blst_fp2) -> Self {
        FsG2(blst_p2 { x, y, z })
    }
}

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
    fn inverse(&self, output_len: usize) -> Result<Self, String> {
        todo!();
        if self.coeffs.len() == 0 {
            return Err(String::from("Can't inverse a zero-length poly"));
        } else if self.coeffs[0].is_zero() {
            return Err(String::from("First coefficient of polynomial mustn't be zero"));
        }

        let mut ret = FsPoly { coeffs: vec![Fr::default(); output_len] };
        // If the input polynomial is constant, the remainder of the series is zero
        if self.coeffs.len() == 1 {
            ret.coeffs[0] = self.coeffs[0].eucl_inverse();

            for i in 1..output_len {
                ret.coeffs[i] = Fr::zero();
            }

            return Ok(ret);
        }

        let maxd = output_len - 1;

        // Max space for multiplications is (2 * length - 1)
        // let scale: usize = log2_pow2(next_power_of_two(2 * output_len - 1));
        // let fft_settings = FsFFTSettings::new(scale).unwrap();

        // To store intermediate results
        let mut tmp0 = FsPoly { coeffs: vec![Fr::default(); output_len] };
        let mut tmp1 = FsPoly { coeffs: vec![Fr::default(); output_len] };

        // Base case for d == 0
        ret.coeffs[0] = self.coeffs[0].eucl_inverse();
        let mut d: usize = 0;
        let mut mask: usize = 1 << log2_u64(maxd);
        while mask != 0 {
            d = 2 * d + (if (maxd & mask) != 0 { 1 } else { 0 });
            mask = mask >> 1;

            // b.c -> tmp0 (we're using out for c)
            // tmp0.length = min_u64(d + 1, b->length + output->length - 1);
            let len_temp = min_u64(d + 1, self.coeffs.len() + output_len - 1);
            tmp0 = self.mul(&ret, len_temp).unwrap();

            // 2 - b.c -> tmp0
            for i in 0..tmp0.coeffs.len() {
                tmp0.coeffs[i] = tmp0.coeffs[i].negate();
            }
            let fr_two = Fr::from_u64(2);
            tmp0.coeffs[0] = tmp0.coeffs[0].add(&fr_two);

            // c.(2 - b.c) -> tmp1;
            tmp1 = ret.mul(&tmp0, d + 1).unwrap();

            // output->length = tmp1.length;
            for i in 0..tmp1.coeffs.len() {
                ret.coeffs.push(tmp1.coeffs[i]);
            }
        }

        if d + 1 != output_len {
            return Err(String::from(""));
        }

        Ok(ret)
    }

    fn div(&self, multiplier: &Self) -> Result<Self, String> {
        todo!();
        return if self.coeffs.len() >= multiplier.coeffs.len() || self.coeffs.len() < 128 { // Tunable paramter
            self.div_long(&multiplier)
        } else {
            self.div_fast(&multiplier)
        };
    }

    fn div_long(&self, divisor: &Self) -> Result<Self, String> {
        todo!();
        if divisor.coeffs.len() == 0 {
            return Err(String::from("Cant divide by zero"));
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
        for i in 0..self.len() {
            a.push(self.coeffs[i]);
        }

        while diff > 0 {
            out.coeffs[diff] = a[a_pos].div(&divisor.coeffs[b_pos]).unwrap();

            for i in 0..(b_pos + 1) {
                let tmp = out.coeffs[diff].mul(&divisor.coeffs[i]);
                a[diff + i] = a[diff + i].sub(&tmp);
            }

            diff -= 1;
            a_pos -= 1;
        }

        out.coeffs[0] = a[a_pos].div(&self.coeffs[b_pos]).unwrap();

        Ok(out)
    }

    fn div_fast(&self, divisor: &Self) -> Result<Self, String> {
        todo!();
        if divisor.coeffs.len() == 0 {
            return Err(String::from("Cant divide by zero"));
        } else if divisor.coeffs[divisor.coeffs.len() - 1].is_zero() {
            return Err(String::from("Highest coefficient must be non-zero"));
        }

        let m: usize = self.coeffs.len() - 1;
        let n: usize = divisor.coeffs.len() - 1;

        // If the divisor is larger than the dividend, the result is zero-length
        if n > m {
            return Ok(FsPoly { coeffs: Vec::default() });
        }

        // Special case for divisor.length == 1 (it's a constant)
        if divisor.len() == 1 {
            let mut out = FsPoly { coeffs: Vec::default() };
            for i in 0..divisor.len() {
                out.coeffs.push(self.coeffs[i].div(&divisor.coeffs[0])?);
            }
            return Ok(out);
        }

        let mut a_flip = self.flip().unwrap();
        let mut b_flip = divisor.flip().unwrap();

        let mut inv_b_flip = b_flip.inverse(m - n + 1).unwrap();
        let mut q_flip = a_flip.mul(&inv_b_flip, m - n + 1).unwrap();

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
        // Truncate a and b so as not to do excess work for the number of coefficients required.
        let a_len = min_u64(self.len(), output_len);
        let b_len = min_u64(multiplier.len(), output_len);
        let length = next_power_of_two(a_len + b_len - 1);

        let scale: usize = log2_pow2(length);
        let fft_settings = FsFFTSettings::new(scale).unwrap();

        let a_pad = self.pad(length);
        let b_pad = multiplier.pad(length);
        let a_fft = fft_settings.fft_fr(&a_pad, false).unwrap();
        let b_fft = fft_settings.fft_fr(&b_pad, false).unwrap();

        let mut ab_fft = vec![FsFr::default(); length];
        for i in 0..length {
            ab_fft[i] = a_fft[i].mul(&b_fft[i]);
        }
        let ab = fft_settings.fft_fr(&ab_fft, true).unwrap();

        let mut ret = FsPoly { coeffs: vec![FsFr::zero(); output_len] };
        for i in 0..min_u64(output_len, length) {
            ret.coeffs[i] = ab[i];
        }

        Ok(ret)
    }

    fn flip(&self) -> Result<FsPoly, String> {
        todo!();
        let mut output = FsPoly { coeffs: Vec::new() };
        for i in 0..self.coeffs.len() {
            output.coeffs.push(self.coeffs[self.coeffs.len() - i - 1]);
        }

        Ok(output)
    }

    fn destroy(&mut self) {}
}

impl FsPoly {
    fn poly_norm(&self) -> Self {
        let mut ret = self.clone();

        let mut temp_len: usize = ret.coeffs.len();
        while temp_len > 0 && ret.coeffs[temp_len - 1].is_zero() {
            temp_len -= 1;
        }

        if temp_len == 0 {
            ret.coeffs = Vec::default();
        } else {
            ret.coeffs = ret.coeffs[0..temp_len].to_vec();
        }

        ret
    }

    fn poly_quotient_length(&self, divisor: &Self) -> usize {
        if self.len() >= divisor.len() {
            return self.len() - divisor.len() + 1;
        }

        0
    }

    fn pad(&self, out_length: usize) -> Vec<FsFr> {
        let mut ret = vec![FsFr::zero(); out_length];

        for i in 0..min_u64(self.len(), out_length) {
            ret[i] = self.coeffs[i];
        }

        ret
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
        todo!()
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

    fn destroy(&mut self) {}
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
