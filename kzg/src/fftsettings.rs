use crate::finite::BlstFr;
use crate::common::KzgRet;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFTSettings {
    pub max_width: u64,
    pub root_of_unity: *mut BlstFr,
    pub expanded_roots_of_unity: *mut BlstFr,
    pub reverse_roots_of_unity: *mut BlstFr
}

extern "C" {
    pub fn new_fft_settings(settings: *mut FFTSettings, max_scale: u32) -> KzgRet;
    pub fn free_fft_settings(settings: *mut FFTSettings);
}

pub fn ckzg_new_fft_settings(settings: *mut FFTSettings, max_scale: u32) -> KzgRet {
    let result;

    unsafe {
        result = new_fft_settings(settings, max_scale);
    }

    result
}

pub fn ckzg_free_fft_settings(settings: *mut FFTSettings) {
    unsafe {
        free_fft_settings(settings);
    }
}