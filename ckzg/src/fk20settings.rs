use kzg::{FK20MultiSettings, FK20SingleSettings, KZGSettings, Poly, G1};

use crate::consts::{BlstP1, BlstP2, KzgRet};
use crate::fftsettings::KzgFFTSettings;
use crate::kzgsettings::KzgKZGSettings;
use crate::finite::BlstFr;
use crate::poly::KzgPoly;
use crate::RUN_PARALLEL;

extern "C" {
    // FK20 Single
    fn new_fk20_single_settings(fk: *mut KzgFK20SingleSettings, n2: u64, ks: *const KzgKZGSettings) -> KzgRet;
    fn da_using_fk20_single(out: *mut BlstP1, p: *const KzgPoly, fk: *const KzgFK20SingleSettings, run_parallel: bool) -> KzgRet;
    fn fk20_single_da_opt(out: *mut BlstP1, p: *const KzgPoly, fk: *const KzgFK20SingleSettings, run_parallel: bool) -> KzgRet;
    fn free_fk20_single_settings(fk: *mut KzgFK20SingleSettings);
    // FK20 Multi
    fn new_fk20_multi_settings(fk: *mut KzgFK20MultiSettings, n2: u64, chunk_len: u64, ks: *const KzgKZGSettings) -> KzgRet;
    fn da_using_fk20_multi(out: *mut BlstP1, p: *const KzgPoly, fk: *const KzgFK20MultiSettings, run_parallel: bool) -> KzgRet;
    fn fk20_multi_da_opt(out: *mut BlstP1, p: *const KzgPoly, fk: *const KzgFK20MultiSettings, run_parallel: bool) -> KzgRet;
    fn free_fk20_multi_settings(fk: *mut KzgFK20MultiSettings);
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct KzgFK20SingleSettings {
    pub ks: *const KzgKZGSettings,
    pub x_ext_fft: *mut BlstP1,
    pub x_ext_fft_len: u64,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct KzgFK20MultiSettings {
    pub ks: *const KzgKZGSettings,
    pub chunk_len: u64,
    pub x_ext_fft_files: *mut *mut BlstP1,
    pub length: u64,
}

impl FK20SingleSettings<BlstFr, BlstP1, BlstP2, KzgFFTSettings, KzgPoly, KzgKZGSettings> for KzgFK20SingleSettings {
    fn default() -> Self {
        Self {
            ks: &KZGSettings::default(),
            x_ext_fft: &mut G1::default(),
            x_ext_fft_len: 0,
        }
    }

    fn new(ks: &KzgKZGSettings, n2: usize) -> Result<Self, String> {
        let mut settings = FK20SingleSettings::default();
        unsafe {
            match new_fk20_single_settings(&mut settings, n2 as u64, ks) {
                KzgRet::KzgOk => Ok(settings),
                e => Err(format!("An error has occurred in FK20SingleSettings::new ==> {:?}", e))
            }
        }
    }

    fn data_availability(&self, p: &KzgPoly) -> Result<Vec<BlstP1>, String> {
        let mut ret = vec![G1::default(); 2 * p.len() as usize];
        unsafe {
            match da_using_fk20_single(ret.as_mut_ptr(), p, self, RUN_PARALLEL) {
                KzgRet::KzgOk => Ok(ret),
                e => Err(format!("An error has occurred in FK20SingleSettings::data_availability ==> {:?}", e))
            }
        }
    }

    fn data_availability_optimized(&self, p: &KzgPoly) -> Result<Vec<BlstP1>, String> {
        let mut ret = vec![G1::default(); 2 * p.len() as usize];
        unsafe {
            match fk20_single_da_opt(ret.as_mut_ptr(), p, self, RUN_PARALLEL) {
                KzgRet::KzgOk => Ok(ret),
                e => Err(format!("An error has occurred in FK20SingleSettings::data_availability_optimized ==> {:?}", e))
            }
        }
    }
}

impl Drop for KzgFK20SingleSettings {
    fn drop(&mut self) {
        unsafe {
            if self.x_ext_fft_len > 0 {
                free_fk20_single_settings(self);
            }
        }
    }
}

impl FK20MultiSettings<BlstFr, BlstP1, BlstP2, KzgFFTSettings, KzgPoly, KzgKZGSettings> for KzgFK20MultiSettings {
    fn default() -> Self {
        Self {
            ks: &KZGSettings::default(),
            chunk_len: 0,
            x_ext_fft_files: std::ptr::null_mut(),
            length: 0
        }
    }

    fn new(ks: &KzgKZGSettings, n2: usize, chunk_len: usize) -> Result<Self, String> {
        let mut settings = FK20MultiSettings::default();
        unsafe {
            match new_fk20_multi_settings(&mut settings, n2 as u64, chunk_len as u64, ks) {
                KzgRet::KzgOk => Ok(settings),
                e => Err(format!("An error has occurred in FK20MultiSettings::new ==> {:?}", e))
            }
        }
    }

    fn data_availability(&self, p: &KzgPoly) -> Result<Vec<BlstP1>, String> {
        let mut ret = vec![G1::default(); 2 * p.len() as usize];
        unsafe {
            match da_using_fk20_multi(ret.as_mut_ptr(), p, self, RUN_PARALLEL) {
                KzgRet::KzgOk => Ok(ret),
                e => Err(format!("An error has occurred in FK20MultiSettings::data_availability ==> {:?}", e))
            }
        }
    }

    fn data_availability_optimized(&self, p: &KzgPoly) -> Result<Vec<BlstP1>, String> {
        let mut ret = vec![G1::default(); 2 * p.len() as usize];
        unsafe {
            match fk20_multi_da_opt(ret.as_mut_ptr(), p, self, RUN_PARALLEL) {
                KzgRet::KzgOk => Ok(ret),
                e => Err(format!("An error has occurred in FK20MultiSettings::data_availability_optimized ==> {:?}", e))
            }
        }
    }
}

impl Drop for KzgFK20MultiSettings {
    fn drop(&mut self) {
        unsafe {
            if self.length > 0 || self.chunk_len > 0 {
                free_fk20_multi_settings(self);
            }
        }
    }
}
