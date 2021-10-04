use super::FFTSettings;
use super::KzgRet;

#[link(name = "ckzg")]
#[link(name = "blst")]
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