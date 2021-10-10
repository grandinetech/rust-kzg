use kzg::{Fr, G1, G2};
use blst::{blst_uint64_from_fr, blst_fr_from_uint64, blst_fr_from_scalar, blst_scalar};
use crate::consts::{SCALE2_ROOT_OF_UNITY, expand_root_of_unity};

pub fn fr_is_one(fr: &Fr) -> bool {
    let mut val: [u64; 4] = [0, 0, 0, 0];
    unsafe {
        blst_uint64_from_fr(val.as_mut_ptr(), fr);
    }
    return val[0] == 1 && val[1] == 0 && val[2] == 0 && val[3] == 0;
}

pub fn create_fr_one() -> Fr {
    let mut ret: Fr = Fr::default();
    unsafe {
        blst_fr_from_uint64(&mut ret, [1, 0, 0, 0].as_ptr());
    }

    ret
}

pub struct Poly {
    pub coeffs: Vec<Fr>,
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
            return Err(String::from("Scale is expected to be within root of unity matrix row size"));
        }

        let max_width: usize = 1 << max_scale;
        let mut root_of_unity: Fr = Fr::default();
        unsafe {
            blst_fr_from_uint64(&mut root_of_unity, SCALE2_ROOT_OF_UNITY[max_scale].as_ptr());
        }

        let expanded_roots_of_unity = expand_root_of_unity(&root_of_unity, max_width).unwrap();
        let mut reverse_roots_of_unity = expanded_roots_of_unity.clone();

        for i in 0..max_width {
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