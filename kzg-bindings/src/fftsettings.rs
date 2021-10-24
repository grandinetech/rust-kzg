use kzg::{Fr, FFTSettings, FFTFr, FFTG1, G1};
use crate::consts::{BlstP1, G1_GENERATOR};
use crate::poly::KzgPoly;
use crate::finite::BlstFr;
use crate::common::KzgRet;
use std::slice;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
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
    fn poly_mul(output: *mut KzgPoly, a: *const KzgPoly, b: *const KzgPoly, fs: *const KzgFFTSettings) -> KzgRet;
    //fn fft_ft_slow(output: *mut BlstFr, input: *mut BlstFr, stride: u64, roots: *const BlstFr, roots_stride: u64, n: u64);
    //fn fft_ft_fast(output: *mut BlstFr, input: *mut BlstFr, stride: u64, roots: *const BlstFr, roots_stride: u64, n: u64);
}

impl FFTSettings<BlstFr> for KzgFFTSettings {
    fn default() -> Self {
        Self {
            max_width: 16,
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
                e => Err(format!("An error has occurred in \"FFTSettings::new\" ==> {:?}", e))
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
        unsafe {
            free_fft_settings(self);
        }
    }

    /*
    pub fn poly_mul(a: *const Poly, b: *const Poly, fs: *const FFTSettings) -> Result<Poly, Error> {
        let mut output = Poly::default();
        unsafe {
            return match poly_mul(&mut output, a, b, fs) {
                Error::KzgOk => Ok(output),
                e => {
                    println!("Error in \"FFTSettings::poly_mul\" ==> {:?}", e);
                    Err(e)
                }
            }
        }
    }

    // Used only for benchmarks
    pub fn bench_fft_fr(scale: u64) {
        let mut fs = match FFTSettings::new(scale as u32) {
            Ok(s) => s,
            Err(_) => FFTSettings::default()
        };
        let mut data = vec![Fr::default(); fs.max_width];
        for i in 0..fs.max_width {
            data[i] = Fr::rand();
        }
        fs.fft_fr(&mut data, false);
        fs.destroy();
    }

    // Used only for benchmarks
    pub fn bench_fft_g1(scale: u64) {
        let mut fs = match FFTSettings::new(scale as u32) {
            Ok(s) => s,
            Err(_) => FFTSettings::default()
        };
        let mut data = vec![G1::default(); fs.max_width];
        for i in 0..fs.max_width {
            data[i] = G1::rand();
        }
        fs.fft_g1(&mut data, false);
        fs.destroy();
    }*/
}

impl FFTFr<BlstFr> for KzgFFTSettings {
    fn fft_fr(&self, data: &mut [BlstFr], inverse: bool) -> Result<Vec<BlstFr>, String> {
        return match _fft_fr(data.as_mut_ptr(), inverse, data.len() as u64, self) {
            Ok(fr) => Ok(fr),
            Err(e) => Err(format!("An error has occurred in \"FFTFr::fft_fr\" ==> {:?}", e))
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
    fn fft_g1(&self, data: &mut [BlstP1], inverse: bool) -> Result<Vec<BlstP1>, String> {
        return match _fft_g1(data.as_mut_ptr(), inverse, data.len() as u64, self) {
            Ok(g) => Ok(g),
            Err(e) => Err(format!("An error has occurred in \"FFTG1::fft_g1\" ==> {:?}", e))
        };
    }
}

fn _fft_g1(input: *mut BlstP1, inverse: bool, n: u64, fs: *const KzgFFTSettings) -> Result<Vec<BlstP1>, KzgRet> {
    let mut output = vec![G1::default(); n as usize];
    unsafe {
        return match fft_g1(output.as_mut_ptr(), input, inverse, n, fs) {
            KzgRet::KzgOk => Ok(output),
            e => Err(e)
        }
    }
}

pub fn make_data(n: usize) -> Vec<BlstP1> {
    let mut out_val = vec![G1_GENERATOR; n];
    let out_ptr: *mut BlstP1 = out_val.as_mut_ptr();
    // Multiples of g1_gen
    if n == 0 { return vec![G1::default(); 0]; }
    for i in 1..n as isize {
        unsafe {
            (*out_ptr.offset(i)).add_or_double(&*out_ptr.offset(i - 1));
        }
    }
    out_val
}
