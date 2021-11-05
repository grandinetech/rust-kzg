use kzg::{FFTSettings, Fr, G1, KZGSettings};

use crate::common::KzgRet;
use crate::consts::{BlstFp, BlstFp2, BlstP1, BlstP2};
use crate::fftsettings::KzgFFTSettings;
use crate::finite::BlstFr;
use crate::poly::KzgPoly;

extern "C" {
    fn new_kzg_settings(ks: *mut KzgKZGSettings, secret_g1: *const BlstP1, secret_g2: *const BlstP2, length: u64, fs: *const KzgFFTSettings) -> KzgRet;
    fn free_kzg_settings(ks: *mut KzgKZGSettings);
    fn commit_to_poly(out: *mut BlstP1, p: *const KzgPoly, ks: *const KzgKZGSettings) -> KzgRet;
    fn compute_proof_single(out: *mut BlstP1, p: *const KzgPoly, x0: *const BlstFr, ks: *const KzgKZGSettings) -> KzgRet;
    fn check_proof_single(out: *mut bool, commitment: *const BlstP1, proof: *const BlstP1, x: *const BlstFr, y: *const BlstFr, ks: *const KzgKZGSettings) -> KzgRet;
    fn compute_proof_multi(out: *mut BlstP1, p: *const KzgPoly, x0: *const BlstFr, n: u64, ks: *const KzgKZGSettings) -> KzgRet;
    fn check_proof_multi(out: *mut bool, commitment: *const BlstP1, proof: *const BlstP1, x: *const BlstFr, ys: *const BlstFr, n: u64, ks: *const KzgKZGSettings) -> KzgRet;
    // Fr
    fn fr_from_scalar(out: *mut BlstFr, a: *const BlstScalar);
    // G1
    fn g1_mul(out: *mut BlstP1, a: *const BlstP1, b: *const BlstFr);
    fn blst_p1_generator() -> *const BlstP1;
    // G2
    fn g2_mul(out: *mut BlstP2, a: *const BlstP2, b: *const BlstFr);
    fn blst_p2_generator() -> *const BlstP2;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BlstScalar {
    pub b: [u8; 32],
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
            secret_g2: &mut BlstP2 { // TODO: G2::default()
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

    fn new(secret_g1: &Vec<BlstP1>, secret_g2: &Vec<BlstP2>, length: usize, fs: *const KzgFFTSettings) -> Result<Self, String> {
        let mut settings = KZGSettings::default();
        unsafe {
            return match new_kzg_settings(&mut settings, secret_g1.as_ptr(), secret_g2.as_ptr(), length as u64, fs) {
                KzgRet::KzgOk => Ok(settings),
                e => Err(format!("An error has occurred in \"KZGSettings::new\" ==> {:?}", e))
            }
        }
    }

    fn commit_to_poly(&self, p: &KzgPoly) -> Result<BlstP1, String> {
        let mut ret = G1::default();
        unsafe {
            return match commit_to_poly(&mut ret, p, self) {
                KzgRet::KzgOk => Ok(ret),
                e => Err(format!("An error has occurred in \"KZGSettings::commit_to_poly\" ==> {:?}", e))
            }
        }
    }

    fn compute_proof_single(&self, p: &KzgPoly, x: &BlstFr) -> Result<BlstP1, String> {
        let mut ret = G1::default();
        unsafe {
            return match compute_proof_single(&mut ret, p, x, self) {
                KzgRet::KzgOk => Ok(ret),
                e => Err(format!("An error has occurred in \"KZGSettings::compute_proof_single\" ==> {:?}", e))
            }
        }
    }

    fn check_proof_single(&self, com: &BlstP1, proof: &BlstP1, x: &BlstFr, value: &BlstFr) -> Result<bool, String> {
        let mut ret = false;
        unsafe {
            return match check_proof_single(&mut ret, com, proof, x, value, self) {
                KzgRet::KzgOk => Ok(ret),
                e => Err(format!("An error has occurred in \"KZGSettings::check_proof_single\" ==> {:?}", e))
            }
        }
    }

    fn compute_proof_multi(&self, p: &KzgPoly, x: &BlstFr, n: usize) -> Result<BlstP1, String> {
        let mut ret = G1::default();
        unsafe {
            return match compute_proof_multi(&mut ret, p, x, n as u64, self) {
                KzgRet::KzgOk => Ok(ret),
                e => Err(format!("An error has occurred in \"KZGSettings::compute_proof_multi\" ==> {:?}", e))
            }
        }
    }

    fn check_proof_multi(&self, com: &BlstP1, proof: &BlstP1, x: &BlstFr, values: &Vec<BlstFr>, n: usize) -> Result<bool, String> {
        let mut ret = false;
        unsafe {
            return match check_proof_multi(&mut ret, com, proof, x, values.as_ptr(), n as u64, self) {
                KzgRet::KzgOk => Ok(ret),
                e => Err(format!("An error has occurred in \"KZGSettings::check_proof_multi\" ==> {:?}", e))
            }
        }
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> BlstFr {
        unsafe {
            let ffs = *self.fs as KzgFFTSettings;
            return ffs.get_expanded_roots_of_unity_at(i);
        }
    }

    fn destroy(&mut self) {
        unsafe {
            free_kzg_settings(self);
        }
    }
}

pub fn generate_trusted_setup(len: usize, secret: [u8; 32usize]) -> (Vec<BlstP1>, Vec<BlstP2>) {
    let mut blst_scalar = BlstScalar {
        b: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    };
    for i in 0..secret.len() {
        blst_scalar.b[i] = secret[i];
    }

    let mut s_pow: BlstFr = Fr::one();
    let mut s = Fr::default();
    unsafe {
        fr_from_scalar(&mut s, &blst_scalar)
    };

    let mut s1 = Vec::new();
    let mut s2 = Vec::new();

    for _i in 0..len {
        let mut g1_temp = G1::default();
        let mut g2_temp = BlstP2 { // TODO: G2::default()
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
        };
        unsafe {
            g1_mul(&mut g1_temp, blst_p1_generator(), &s_pow);
            g2_mul(&mut g2_temp, blst_p2_generator(), &s_pow);
        }
        s1.push(g1_temp);
        s2.push(g2_temp);
        s_pow = s_pow.mul(&s);
    }

    (s1, s2)
}