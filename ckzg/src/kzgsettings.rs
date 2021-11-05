use kzg::{FFTSettings, G1, G2, KZGSettings};

use crate::common::KzgRet;
use crate::consts::{BlstFp, BlstFp2, BlstP1, BlstP2};
use crate::fftsettings::KzgFFTSettings;
use crate::finite::BlstFr;
use crate::poly::KzgPoly;

extern "C" {
    fn new_kzg_settings(ks: *mut KzgKZGSettings, secret_g1: *const BlstP1, secret_g2: *const BlstP2, length: u64, fs: *const KzgFFTSettings) -> KzgRet;
    fn free_kzg_settings(ks: *mut KzgKZGSettings);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct KzgKZGSettings {
    pub fs: *const KzgFFTSettings,
    pub secret_g1: *mut BlstP1, // G1
    pub secret_g2: *mut BlstP2, // G2
    pub length: u64,
}

impl KZGSettings<BlstFr, BlstP1, BlstP2, KzgFFTSettings, KzgPoly> for KzgKZGSettings {
    fn default() -> Self {
        Self {
            fs: &FFTSettings::default(),
            secret_g1: &mut G1::default(),
            secret_g2: &mut BlstP2 { // TODO: need something like G2::default()
                x: BlstFp2 {
                    fp: [
                        BlstFp { l: [0, 0, 0, 0, 0, 0] },
                        BlstFp { l: [0, 0, 0, 0, 0, 0] }
                    ]
                },
                y: BlstFp2 {
                    fp: [
                        BlstFp { l: [0, 0, 0, 0, 0, 0] },
                        BlstFp { l: [0, 0, 0, 0, 0, 0] }
                    ]
                },
                z: BlstFp2 {
                    fp: [
                        BlstFp { l: [0, 0, 0, 0, 0, 0] },
                        BlstFp { l: [0, 0, 0, 0, 0, 0] }
                    ]
                },
            },
            length: 0,
        }
    }

    fn new(secret_g1: &Vec<BlstP1>, secret_g2: &Vec<BlstP2>, length: usize, fs: KzgFFTSettings) -> Self {
        todo!()
    }

    fn commit_to_poly(&self, p: &KzgPoly) -> Result<BlstP1, String> {
        todo!()
    }

    fn compute_proof_single(&self, p: &KzgPoly, x: &BlstFr) -> BlstP1 {
        todo!()
    }

    fn check_proof_single(&self, com: &BlstP1, proof: &BlstP1, x: &BlstFr, value: &BlstFr) -> bool {
        todo!()
    }

    fn compute_proof_multi(&self, p: &KzgPoly, x: &BlstFr, n: usize) -> BlstP1 {
        todo!()
    }

    fn check_proof_multi(&self, com: &BlstP1, proof: &BlstP1, x: &BlstFr, values: &Vec<BlstFr>, n: usize) -> bool {
        todo!()
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> BlstFr {
        todo!()
    }

    fn destroy(&mut self) {
        unsafe {
            free_kzg_settings(self);
        }
    }
}
