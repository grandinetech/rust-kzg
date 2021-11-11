use kzg::{Fr, FFTSettings, FFTSettingsPoly, Poly, FFTFr, FFTG1, G1};
use crate::utils::{log_2, next_pow_of_2};
use crate::consts::{KzgRet, BlstP1};
use crate::poly::KzgPoly;
use crate::finite::BlstFr;
use std::{cmp::min};
use std::slice;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct KzgFFTSettings {
    pub max_width: usize,
    pub root_of_unity: BlstFr,
    pub expanded_roots_of_unity: *mut BlstFr,
    pub reverse_roots_of_unity: *mut BlstFr
}

extern "C" {
    fn new_fft_settings(settings: *mut KzgFFTSettings, max_scale: u32) -> KzgRet;
    fn free_fft_settings(settings: *mut KzgFFTSettings);
    fn fft_fr(output: *mut BlstFr, input: *const BlstFr, inverse: bool, n: u64, fs: *const KzgFFTSettings) -> KzgRet;
    fn fft_g1(output: *mut BlstP1, input: *const BlstP1, inverse: bool, n: u64, fs: *const KzgFFTSettings) -> KzgRet;
    fn poly_mul_(out: *mut KzgPoly, a: *const KzgPoly, b: *const KzgPoly, fs: *mut KzgFFTSettings) -> KzgRet;
    fn fft_fr_fast(output: *mut BlstFr, input: *const BlstFr, stride: usize, roots: *const BlstFr, roots_stride: usize, n: usize);
    fn fft_fr_slow(output: *mut BlstFr, input: *const BlstFr, stride: usize, roots: *const BlstFr, roots_stride: usize, n: usize);
    fn fft_g1_fast(output: *mut BlstP1, input: *const BlstP1, stride: usize, roots: *const BlstFr, roots_stride: usize, n: usize);
    fn fft_g1_slow(output: *mut BlstP1, input: *const BlstP1, stride: usize, roots: *const BlstFr, roots_stride: usize, n: usize);
}

impl FFTSettings<BlstFr> for KzgFFTSettings {
    fn default() -> Self {
        Self {
            max_width: 0,
            root_of_unity: Fr::default(),
            expanded_roots_of_unity: &mut Fr::default(),
            reverse_roots_of_unity: &mut Fr::default()
        }
    }

    fn new(scale: usize) -> Result<Self, String> {
        let mut settings = FFTSettings::default();
        unsafe {
            return match new_fft_settings(&mut settings, scale as u32) {
                KzgRet::KzgOk => Ok(settings),
                e => Err(format!("An error has occurred in FFTSettings::new ==> {:?}", e))
            }
        }
    }

    fn get_max_width(&self) -> usize {
        self.max_width
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> BlstFr {
        unsafe {
            return *self.expanded_roots_of_unity.offset(i as isize) as BlstFr;
        }
    }

    fn get_expanded_roots_of_unity(&self) -> &[BlstFr] {
        unsafe {
            return slice::from_raw_parts(self.expanded_roots_of_unity, self.max_width);
        }
    }

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> BlstFr {
        unsafe {
            return *self.reverse_roots_of_unity.offset(i as isize) as BlstFr;
        }
    }

    fn get_reversed_roots_of_unity(&self) -> &[BlstFr] {
        unsafe {
            return slice::from_raw_parts(self.reverse_roots_of_unity, self.max_width);
        }
    }

    fn destroy(&mut self) {
        //unsafe {
        //    free_fft_settings(self);
        //}
    }
}

impl Drop for KzgFFTSettings {
    fn drop(&mut self) {
        unsafe {
            if self.max_width > 0 && self.max_width < (1 << 32) {
                free_fft_settings(self);
            }
        }
    }
}

impl FFTFr<BlstFr> for KzgFFTSettings {
    fn fft_fr(&self, data: &[BlstFr], inverse: bool) -> Result<Vec<BlstFr>, String> {
        return match _fft_fr(data.as_ptr(), inverse, data.len() as u64, self) {
            Ok(fr) => Ok(fr),
            Err(e) => Err(format!("An error has occurred in FFTFr::fft_fr ==> {:?}", e))
        };
    }
}

fn _fft_fr(input: *const BlstFr, inverse: bool, n: u64, fs: *const KzgFFTSettings) -> Result<Vec<BlstFr>, KzgRet> {
    let mut output = vec![Fr::default(); n as usize];
    unsafe {
        return match fft_fr(output.as_mut_ptr(), input, inverse, n, fs) {
            KzgRet::KzgOk => Ok(output),
            e => Err(e)
        }
    }
}

impl FFTG1<BlstP1> for KzgFFTSettings {
    fn fft_g1(&self, data: &[BlstP1], inverse: bool) -> Result<Vec<BlstP1>, String> {
        return match _fft_g1(data.as_ptr(), inverse, data.len() as u64, self) {
            Ok(g) => Ok(g),
            Err(e) => Err(format!("An error has occurred in FFTG1::fft_g1 ==> {:?}", e))
        };
    }
}

fn _fft_g1(input: *const BlstP1, inverse: bool, n: u64, fs: *const KzgFFTSettings) -> Result<Vec<BlstP1>, KzgRet> {
    let mut output = vec![G1::default(); n as usize];
    unsafe {
        return match fft_g1(output.as_mut_ptr(), input, inverse, n, fs) {
            KzgRet::KzgOk => Ok(output),
            e => Err(e)
        }
    }
}

impl FFTSettingsPoly<BlstFr, KzgPoly, KzgFFTSettings> for KzgFFTSettings {
    fn poly_mul_fft(a: &KzgPoly, b: &KzgPoly, len: usize, _fs: Option<&KzgFFTSettings>) -> Result<KzgPoly, String> {
        // Truncate a and b so as not to do excess work for the number of coefficients required
        let a_len = min(a.len(), len);
        let b_len = min(b.len(), len);
        let length = next_pow_of_2(a_len + b_len - 1);

        let mut fft = KzgFFTSettings::new(log_2(length)).unwrap();
        let mut poly = KzgPoly::new(len).unwrap();
        unsafe {
            return match poly_mul_(&mut poly, a, b, &mut fft) {
                KzgRet::KzgOk => {
                    fft.destroy();
                    Ok(poly)
                },
                e => Err(format!("An error has occurred in FFTSettingsPoly::poly_mul_fft ==> {:?}", e))
            }
        }
    }
}

pub fn make_data(n: usize) -> Vec<BlstP1> {
    let mut out_val = vec![G1::generator(); n];
    let out_ptr: *mut BlstP1 = out_val.as_mut_ptr();
    if n == 0 { return vec![G1::default(); 0]; }
    for i in 1..n as isize {
        unsafe {
            (*out_ptr.offset(i)).add_or_dbl(&*out_ptr.offset(i - 1));
        }
    }
    out_val
}

pub fn bound_fft_fr_fast(
    ret: &mut [BlstFr],
    data: &[BlstFr],
    stride: usize,
    roots: &[BlstFr],
    roots_stride: usize,
) {
    unsafe {
        fft_fr_fast(ret.as_mut_ptr(), data.as_ptr(), stride,
                    roots.as_ptr(), roots_stride, 4096);
    }
}

pub fn bound_fft_fr_slow(
    ret: &mut [BlstFr],
    data: &[BlstFr],
    stride: usize,
    roots: &[BlstFr],
    roots_stride: usize,
) {
    unsafe {
        fft_fr_slow(ret.as_mut_ptr(), data.as_ptr(), stride,
                    roots.as_ptr(), roots_stride, 4096);
    }
}

pub fn bound_fft_g1_fast(
    ret: &mut [BlstP1],
    data: &[BlstP1],
    stride: usize,
    roots: &[BlstFr],
    roots_stride: usize,
    n: usize
) {
    unsafe {
        fft_g1_fast(ret.as_mut_ptr(), data.as_ptr(), stride,
                    roots.as_ptr(), roots_stride, n);
    }
}

pub fn bound_fft_g1_slow(
    ret: &mut [BlstP1],
    data: &[BlstP1],
    stride: usize,
    roots: &[BlstFr],
    roots_stride: usize,
    n: usize
) {
    unsafe {
        fft_g1_slow(ret.as_mut_ptr(), data.as_ptr(), stride,
                    roots.as_ptr(), roots_stride, n);
    }
}
