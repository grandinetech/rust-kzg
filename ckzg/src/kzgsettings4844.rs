use kzg::{FFTSettings, Fr, G1Mul, G2Mul, KZGSettings, G1, G2};

use crate::consts::{BlstP1, BlstP2, KzgRet};
use crate::fftsettings4844::KzgFFTSettings4844;
use crate::finite::BlstFr;
use crate::poly::KzgPoly;
use crate::RUN_PARALLEL;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BlstScalar {
    pub b: [u8; 32],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct KzgKZGSettings4844 {
    pub fs: *const KzgFFTSettings4844,
    pub secret_g1: *mut BlstP1, // G1
    pub secret_g2: *mut BlstP2, // G2
}

extern "C" {
    fn free_trusted_setup(s: *mut KzgKZGSettings4844);
}

impl KZGSettings<BlstFr, BlstP1, BlstP2, KzgFFTSettings4844, KzgPoly> for KzgKZGSettings4844 {
    fn default() -> Self {
    
    println!("fs created here is maybe dropped later");
        Self {
            fs: &FFTSettings::default(),
            secret_g1: &mut G1::default(),
            secret_g2: &mut G2::default(),
        }
    }

    fn new(
        secret_g1: &Vec<BlstP1>,
        secret_g2: &Vec<BlstP2>,
        length: usize,
        fs: &KzgFFTSettings4844,
    ) -> Result<Self, String> {
        todo!();
        // let mut settings = KzgKZGSettings4844::default();
        // unsafe {
        //     Ok(Self {
        //         fs,
        //         secret_g1: secret_g1.as_ptr(),
        //         secret_g2: secret_g2.as_ptr(),
        //         length: length as u64,

        //         }
        //     )
        // }
    }

    fn commit_to_poly(&self, p: &KzgPoly) -> Result<BlstP1, String> {
        todo!();
    }

    fn compute_proof_single(&self, p: &KzgPoly, x: &BlstFr) -> Result<BlstP1, String> {
        todo!();
    }

    fn check_proof_single(
        &self,
        com: &BlstP1,
        proof: &BlstP1,
        x: &BlstFr,
        value: &BlstFr,
    ) -> Result<bool, String> {
        todo!();
    }

    fn compute_proof_multi(&self, p: &KzgPoly, x: &BlstFr, n: usize) -> Result<BlstP1, String> {
        todo!();
    }

    fn check_proof_multi(
        &self,
        com: &BlstP1,
        proof: &BlstP1,
        x: &BlstFr,
        values: &Vec<BlstFr>,
        n: usize,
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