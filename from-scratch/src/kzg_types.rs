use crate::consts::{expand_root_of_unity, SCALE2_ROOT_OF_UNITY, SCALE_FACTOR};
use blst::{blst_fr_add, blst_fr_cneg, blst_fr_from_uint64, blst_fr_inverse, blst_fr_mul, blst_uint64_from_fr};
use kzg::{Fr, G1, G2};

pub fn fr_is_one(fr: &Fr) -> bool {
    let mut val: [u64; 4] = [0; 4];
    unsafe {
        blst_uint64_from_fr(val.as_mut_ptr(), fr);
    }
    return val[0] == 1 && val[1] == 0 && val[2] == 0 && val[3] == 0;
}

pub fn fr_is_zero(fr: &Fr) -> bool {
    let mut val: [u64; 4] = [0; 4];
    unsafe {
        blst_uint64_from_fr(val.as_mut_ptr(), fr);
    }
    return val[0] == 0 && val[1] == 0 && val[2] == 0 && val[3] == 0;
}

pub fn create_fr_u64(val: u64) -> Fr {
    let mut ret: Fr = Fr::default();
    unsafe {
        blst_fr_from_uint64(&mut ret, [val, 0, 0, 0].as_ptr());
    }

    ret
}

pub fn create_fr_zero() -> Fr {
    create_fr_u64(0)
}

pub fn create_fr_one() -> Fr {
    create_fr_u64(1)
}

pub fn fr_are_equal(a: &Fr, b: &Fr) -> bool {
    let mut val_a: [u64; 4] = [0; 4];
    let mut val_b: [u64; 4] = [0; 4];

    unsafe {
        blst_uint64_from_fr(val_a.as_mut_ptr(), a);
        blst_uint64_from_fr(val_b.as_mut_ptr(), b);
    }

    return val_a[0] == val_b[0]
        && val_a[1] == val_b[1]
        && val_a[2] == val_b[2]
        && val_a[3] == val_b[3];
}

pub fn create_fr_rand() -> Fr {
    let val: [u64; 4] = rand::random();
    let mut ret: Fr = Fr::default();
    unsafe {
        blst_fr_from_uint64(&mut ret, val.as_ptr());
    }

    ret
}

pub fn negate_fr(ret: &mut Fr, val: &Fr) {
    unsafe {
        blst_fr_cneg(ret, val, true);
    }
}

pub struct Poly {
    pub coeffs: Vec<Fr>,
}

impl Poly {
    pub fn scale(&mut self) {
        let mut scale_factor: Fr = Fr::default();
        let mut inv_factor: Fr = Fr::default();

        unsafe {
            blst_fr_from_uint64(&mut scale_factor, [SCALE_FACTOR, 0, 0, 0].as_ptr());
            blst_fr_inverse(&mut inv_factor, &scale_factor);
        }

        let mut factor_power = create_fr_one();
        for i in 0..self.coeffs.len() {
            unsafe {
                blst_fr_mul(&mut factor_power, &factor_power, &inv_factor);
                blst_fr_mul(&mut self.coeffs[i], &self.coeffs[i], &factor_power);
            }
        }
    }

    pub fn unscale(&mut self) {
        let mut scale_factor: Fr = Fr::default();

        unsafe {
            blst_fr_from_uint64(&mut scale_factor, [SCALE_FACTOR, 0, 0, 0].as_ptr());
        }

        let mut factor_power = create_fr_one();
        for i in 0..self.coeffs.len() {
            unsafe {
                blst_fr_mul(&mut factor_power, &factor_power, &scale_factor);
                blst_fr_mul(&mut self.coeffs[i], &self.coeffs[i], &factor_power);
            }
        }
    }

    pub fn eval(&mut self, x: &Fr) -> Fr {
        if self.coeffs.len() == 0 {
            return create_fr_zero();
        } else if fr_is_zero(x) {
            return self.coeffs[0].clone();
        }

        let mut ret = self.coeffs[self.coeffs.len() - 1];
        let mut i = self.coeffs.len() - 2;
        loop {
            let mut temp = Fr::default();
            unsafe {
                blst_fr_mul(&mut temp, &ret, x);
                blst_fr_add(&mut ret, &temp, &self.coeffs[i]);
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }

        return ret;
    }
}

impl Clone for Poly {
    fn clone(&self) -> Self {
        Poly { coeffs: self.coeffs.clone() }
    }
}

pub struct FFTSettings {
    pub max_width: usize,
    pub root_of_unity: Fr,
    pub expanded_roots_of_unity: Vec<Fr>,
    pub reverse_roots_of_unity: Vec<Fr>,
}

impl FFTSettings {
    pub fn from_scale(max_scale: usize) -> Result<FFTSettings, String> {
        if max_scale >= SCALE2_ROOT_OF_UNITY.len() {
            return Err(String::from(
                "Scale is expected to be within root of unity matrix row size",
            ));
        }

        let max_width: usize = 1 << max_scale;
        let mut root_of_unity: Fr = Fr::default();
        unsafe {
            blst_fr_from_uint64(&mut root_of_unity, SCALE2_ROOT_OF_UNITY[max_scale].as_ptr());
        }

        let expanded_roots_of_unity = expand_root_of_unity(&root_of_unity, max_width).unwrap();
        let mut reverse_roots_of_unity = expanded_roots_of_unity.clone();

        for i in 0..(max_width + 1) {
            reverse_roots_of_unity[i] = expanded_roots_of_unity[max_width - i];
        }

        return Ok(FFTSettings {
            max_width,
            root_of_unity,
            expanded_roots_of_unity,
            reverse_roots_of_unity,
        });
    }
}

pub struct KZGSettings {
    pub fs: FFTSettings,
    // Both secret_g1 and secret_g2 have the same number of elements
    pub secret_g1: Vec<G1>,
    pub secret_g2: Vec<G2>,
}
