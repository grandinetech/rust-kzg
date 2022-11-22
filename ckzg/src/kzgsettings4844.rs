use kzg::{FFTSettings, KZGSettings, G1, G2};

use crate::consts::{BlstP1, BlstP2};
use crate::fftsettings4844::KzgFFTSettings4844;
use crate::finite::BlstFr;
use crate::poly::KzgPoly;
// use crate::RUN_PARALLEL;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BlstScalar {
    pub b: [u8; 32],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct KzgKZGSettings4844 {
    pub fs: *const KzgFFTSettings4844,
    pub g1_values: *mut BlstP1, // G1
    pub g2_values: *mut BlstP2, // G2
}

extern "C" {
    // fn free_trusted_setup(s: *mut KzgKZGSettings4844);
}

impl KZGSettings<BlstFr, BlstP1, BlstP2, KzgFFTSettings4844, KzgPoly> for KzgKZGSettings4844 {
    fn default() -> Self {
    
    println!("creating kzgsettings4844");
    println!("fs created here is maybe dropped later");
        Self {
            fs: &FFTSettings::default(),
            g1_values: &mut G1::default(),
            g2_values: &mut G2::default(),
        }
    }

    // underscore was added to avoid warnings when new is unused
    fn new(
        _secret_g1: &Vec<BlstP1>,
        _secret_g2: &Vec<BlstP2>,
        _length: usize,
        _fs: &KzgFFTSettings4844,
    ) -> Result<Self, String> {
        todo!();
        // let mut settings = KzgKZGSettings4844::default();
        // unsafe {
        //     Ok(Self {
        //         fs,
        //         g1_values: secret_g1.as_ptr(),
        //         g2_values: secret_g2.as_ptr(),
        //         length: length as u64,

        //         }
        //     )
        // }
    }

    fn commit_to_poly(&self, _p: &KzgPoly) -> Result<BlstP1, String> {
        todo!();
    }

    fn compute_proof_single(&self, _p: &KzgPoly, _x: &BlstFr) -> Result<BlstP1, String> {
        todo!();
    }

    fn check_proof_single(
        &self,
        _com: &BlstP1,
        _proof: &BlstP1,
        _x: &BlstFr,
        _value: &BlstFr,
    ) -> Result<bool, String> {
        todo!();
    }

    fn compute_proof_multi(&self, _p: &KzgPoly, _x: &BlstFr, _n: usize) -> Result<BlstP1, String> {
        todo!();
    }

    fn check_proof_multi(
        &self,
        _com: &BlstP1,
        _proof: &BlstP1,
        _x: &BlstFr,
        _values: &Vec<BlstFr>,
        _n: usize,
    ) -> Result<bool, String> {
        todo!();
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> BlstFr {
        unsafe { (*self.fs).get_expanded_roots_of_unity_at(i) }
    }
}

impl Drop for KzgKZGSettings4844 {
    fn drop(&mut self) {
        println!("dropinu kzgKZGSettings4844");
        // unsafe {
        //     if self.length > 0 {
        //         free_trusted_setup(self);
        //     }
        // }
    }
}