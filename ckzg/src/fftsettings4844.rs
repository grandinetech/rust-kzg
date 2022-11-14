use crate::consts::{BlstP1, KzgRet};
use crate::finite::BlstFr;
use crate::poly::KzgPoly;
use crate::utils::{log_2, next_pow_of_2};
use crate::RUN_PARALLEL;
use kzg::{FFTFr, FFTSettings, FFTSettingsPoly, Fr, Poly, ZeroPoly, DAS, FFTG1, G1};
use std::ptr::null;
use std::{cmp::min, slice};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct KzgFFTSettings4844 {
    pub max_width: u64,
    pub expanded_roots_of_unity: *mut BlstFr,
    pub reverse_roots_of_unity: *mut BlstFr,
    pub roots_of_unity: *mut BlstFr,
}

extern "C" {
    fn new_fft_settings(settings: *mut KzgFFTSettings4844, max_scale: u32) -> KzgRet;
}

impl FFTSettings<BlstFr> for KzgFFTSettings4844 {
    fn default() -> Self {
        Self {
            max_width: 0,
            roots_of_unity: &mut Fr::default(),
            expanded_roots_of_unity: &mut Fr::default(),
            reverse_roots_of_unity: &mut Fr::default(),
        }
    }

    fn new(scale: usize) -> Result<Self, String> {
        let mut settings = Box::new(KzgFFTSettings4844::default());
        unsafe {
            let v = Box::<KzgFFTSettings4844>::into_raw(settings);
            match new_fft_settings(v, scale as u32) {
                KzgRet::KzgOk => Ok(*Box::<KzgFFTSettings4844>::from_raw(v)),
                e => Err(format!(
                    "An error has occurred in FFTSettings::new ==> {:?}",
                    e
                )),
            }
        }
    }

    fn get_max_width(&self) -> usize {
        self.max_width as usize
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> BlstFr {
        unsafe { *(self.expanded_roots_of_unity.add(i )) as BlstFr }
    }

    fn get_expanded_roots_of_unity(&self) -> &[BlstFr] {
        unsafe { slice::from_raw_parts(self.expanded_roots_of_unity, self.max_width as usize) }
    }

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> BlstFr {
        unsafe { *self.reverse_roots_of_unity.add(i) as BlstFr }
    }

    fn get_reversed_roots_of_unity(&self) -> &[BlstFr] {
        unsafe { slice::from_raw_parts(self.reverse_roots_of_unity, self.max_width as usize) }
    }
}

impl Drop for KzgFFTSettings4844 {
    fn drop(&mut self) {
        println!("reikes implementuoti drop kazkada");
        // unsafe {
        //     if self.max_width > 0 && self.max_width < (1 << 32) {
        //         free_fft_settings(self);
        //     }
        // }
    }
}

