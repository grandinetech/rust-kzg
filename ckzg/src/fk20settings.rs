use kzg::{FK20SingleSettings, G1, KZGSettings};

use crate::consts::{BlstP1, BlstP2, KzgRet};
use crate::fftsettings::KzgFFTSettings;
use crate::kzgsettings::KzgKZGSettings;
use crate::finite::BlstFr;
use crate::poly::KzgPoly;

extern "C" {
    // FK20 Single
    fn new_fk20_single_settings(fk: *mut KzgFK20SingleSettings, n2: u64, ks: *const KzgKZGSettings) -> KzgRet;
    fn da_using_fk20_single(out: *mut BlstP1, p: *const KzgPoly, fk: *const KzgFK20SingleSettings) -> KzgRet;
    fn fk20_single_da_opt(out: *mut BlstP1, p: *const KzgPoly, fk: *const KzgFK20SingleSettings) -> KzgRet;
    fn free_fk20_single_settings(fk: *mut KzgFK20SingleSettings);
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct KzgFK20SingleSettings {
    pub ks: *const KzgKZGSettings,
    pub x_ext_fft: *mut BlstP1,
    pub x_ext_fft_len: u64,
}

impl FK20SingleSettings<BlstFr, BlstP1, BlstP2, KzgFFTSettings, KzgPoly, KzgKZGSettings> for KzgFK20SingleSettings {
    fn default() -> Self {
        Self {
            ks: &KZGSettings::default(),
            x_ext_fft: &mut G1::default(),
            x_ext_fft_len: 0,
        }
    }

    fn new(n2: usize, ks: &KzgKZGSettings) -> Result<Self, String> {
        let mut settings = FK20SingleSettings::default();
        unsafe {
            return match new_fk20_single_settings(&mut settings, n2 as u64, ks) {
                KzgRet::KzgOk => Ok(settings),
                e => Err(format!("An error has occurred in FK20SingleSettings::new ==> {:?}", e))
            };
        }
    }

    fn data_availability(&self, p: &KzgPoly) -> Result<BlstP1, String> {
        let mut ret = G1::default();
        unsafe {
            return match da_using_fk20_single(&mut ret, p, self) {
                KzgRet::KzgOk => Ok(ret),
                e => Err(format!("An error has occurred in FK20SingleSettings::data_availability ==> {:?}", e))
            };
        }
    }

    fn data_availability_optimized(&self, p: &KzgPoly) -> Result<BlstP1, String> {
        let mut ret = G1::default();
        unsafe {
            return match fk20_single_da_opt(&mut ret, p, self) {
                KzgRet::KzgOk => Ok(ret),
                e => Err(format!("An error has occurred in FK20SingleSettings::data_availability_optimized ==> {:?}", e))
            };
        }
    }
}

impl Drop for KzgFK20SingleSettings {
    fn drop(&mut self) {
        unsafe {
            free_fk20_single_settings(self);
        }
    }
}
