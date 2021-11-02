use kzg::{FFTSettings, Fr, Poly, G1};
use crate::consts::{expand_root_of_unity, SCALE2_ROOT_OF_UNITY, SCALE_FACTOR, G1_IDENTITY, G1_GENERATOR};
use blst::{blst_fp, blst_fp2, blst_fr, blst_fr_add, blst_fr_cneg, blst_fr_eucl_inverse,
           blst_fr_from_uint64, blst_fr_inverse, blst_fr_mul, blst_fr_sqr, blst_fr_sub, blst_p1, blst_p2,
           blst_uint64_from_fr, blst_p1_add_or_double, blst_p1_is_equal, blst_scalar, blst_scalar_from_fr,
           blst_p1_mult, blst_p1_cneg, blst_fr_from_scalar};
use crate::utils::log_2_byte;

pub struct FsFr(pub blst::blst_fr);

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
        let val: [u64; 4] = rand::random();
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

    // TODO: double-check implementation
    fn pow(&self, n: usize) -> Self {
        //fr_t tmp = *a;
        let mut tmp = self.clone();

        //*out = fr_one;
        let mut out = Self::one();
        let mut n2 = n;

        //unsafe {
            loop {
                if n2 & 1 == 1 {
                    // blst_fr_mul(&mut out.0, &out.0, &tmp.0);
                    out = out.mul(&tmp);
                }
                n2 = n2 >> 1;
                if n == 0 {
                    break;
                }
                // blst_fr_sqr(&mut tmp.0, &tmp.0);
                tmp = tmp.sqr();
            }
        //}

        out
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

    // fn get_scalar(&self) -> Scalar {
    //     let mut scalar = Scalar::default();
    //     unsafe {
    //         blst_scalar_from_fr(&mut scalar, &self.0);
    //     }
    //
    //     scalar
    // }

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

    // fn from_scalar(scalar: &Scalar) -> Self {
    //     let mut fr = blst_fr::default();
    //     unsafe {
    //         blst_fr_from_scalar(&mut fr, scalar);
    //     }
    //     let mut ret = Self::default();
    //     ret.0 = fr;
    //     ret
    // }

    fn destroy(&mut self) {}
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

impl G1<FsFr> for FsG1 {
    fn default() -> Self {
       Self(blst_p1::default())
    }

    fn rand() -> Self {
        let result = G1_GENERATOR;
        result.mul(&FsFr::rand())
    }

    fn add_or_double(&self, b: &Self) -> Self {
        let mut ret = Self::default();
        unsafe {
            blst_p1_add_or_double(&mut ret.0, &self.0, &b.0);
        }
        ret
    }

    fn equals(&self, b: &Self) -> bool {
        unsafe {
            return blst_p1_is_equal(&self.0, &b.0);
        }
    }

    fn mul(&self, b: &FsFr) -> Self {
        let mut scalar: blst_scalar = blst_scalar::default();
        unsafe {
            blst_scalar_from_fr(&mut scalar, &b.0);
        }

        // Count the number of bytes to be multiplied.
        let mut i = scalar.b.len();// std::mem::size_of::<blst_scalar>();
        while i != 0 && scalar.b[i - 1] == 0 {
            i -= 1;
        }
        let mut ret = Self::default();
        return if i == 0 {
            G1_IDENTITY
        } else if i == 1 && scalar.b[0] == 1 {
            *self
        } else {
            // Count the number of bits to be multiplied.
            unsafe {
                blst_p1_mult(&mut ret.0, &self.0, &(scalar.b[0]), 8 * i - 7 + log_2_byte(scalar.b[i - 1]));
            }
            ret
        }
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

    fn destroy(&mut self) {
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

impl FsG2 {
    pub(crate) fn from_xyz(x: blst_fp2, y: blst_fp2, z: blst_fp2) -> Self {
        FsG2(blst_p2 { x, y, z })
    }

    pub fn default() -> Self {
        Self(blst_p2::default())
    }

}

impl Clone for FsG2 {
    fn clone(&self) -> Self {
        todo!()
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

    fn inverse(&mut self, new_len: usize) -> Result<Self, String> {
        todo!()
    }

    fn div(&mut self, x: &Self) -> Result<Self, String> {
        todo!()
    }

    fn destroy(&mut self) {}
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

impl FsKZGSettings {

    pub fn default() -> Self {
        let output = Self {
            secret_g1: Vec::default(),
            secret_g2: Vec::default(),
            fs: FsFFTSettings::default()
        };
        output
    }

    pub fn new(secret_g1: &Vec<FsG1>, secret_g2: &Vec<FsG2>, length: usize, fft_settings: &FsFFTSettings) -> Self {
        let mut kzg_settings = Self::default();

        // CHECK(length >= fs->max_width);
        // assert_eq!(secret_g1.len(), secret_g2.len());
        assert!(secret_g1.len() >= fft_settings.max_width);
        assert!(secret_g2.len() >= fft_settings.max_width);
        assert!(length >= fft_settings.max_width);

        // ks->length = length;

        // Allocate space for the secrets
        // TRY(new_g1_array(&ks->secret_g1, ks->length));
        // TRY(new_g2_array(&ks->secret_g2, ks->length));

        // Populate the secrets
        for i in 0..length {
            kzg_settings.secret_g1.push(secret_g1[i]);
            kzg_settings.secret_g2.push(secret_g2[i]);

            // kzg_settings.secret_g1[i] = secret_g1[i];
            // kzg_settings.secret_g2[i] = secret_g2[i];
        }
        kzg_settings.fs = fft_settings.clone();
        kzg_settings
    }
}
