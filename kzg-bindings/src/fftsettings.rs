use kzg::{Fr, FFTSettings, FFTFr};
use crate::poly::KzgPoly;
use crate::finite::BlstFr;
use crate::common::KzgRet;

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
    //fn fft_g1(output: *mut G1, input: *const G1, inverse: bool, n: u64, fs: *const KzgFFTSettings) -> KzgRet;
    fn poly_mul(output: *mut KzgPoly, a: *const KzgPoly, b: *const KzgPoly, fs: *const KzgFFTSettings) -> KzgRet;
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
        todo!()
    }

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> BlstFr {
        unsafe {
            return *self.reverse_roots_of_unity.offset(i as isize) as BlstFr;
        }
    }

    fn get_reversed_roots_of_unity(&self) -> &[BlstFr] {
        todo!()
    }

    fn destroy(&mut self) {
        unsafe {
            free_fft_settings(self);
        }
    }

    /*
    pub fn fft_g1(&self, input: &mut Vec<G1>, inverse: bool) -> Vec<G1> {
        let output = match FFTSettings::_fft_g1(input.as_mut_ptr(), inverse, input.len() as u64, self) {
            Ok(g) => g,
            Err(e) => panic!("Error in \"FFTSettings::fft_g1\" ==> {:?}", e)
        };
        output
    }

    fn _fft_g1(input: *mut G1, inverse: bool, n: u64, fs: *const FFTSettings) -> Result<Vec<G1>, Error> {
        let mut output = vec![G1::default(); n as usize];
        unsafe {
            return match fft_g1(output.as_mut_ptr(), input, inverse, n, fs) {
                Error::KzgOk => Ok(output),
                e => Err(e)
            }
        }
    }

    // https://github.com/benjaminion/c-kzg/blob/63612c11192cea02b2cb78aa677f570041b6b763/src/fft_g1.c#L95
    pub fn make_data(n: usize) -> Vec<G1> {
        // It's not mentioned in c-kzg implementation but for safety reasons allocate (n + 1) space
        // because we'll try to access one element further its initial size in the first for loop
        let mut out_val = vec![G1_GENERATOR; n + 1];
        let out_ptr: *mut G1 = out_val.as_mut_ptr();
        // Multiples of g1_gen
        if n == 0 { return vec![G1::default(); 0]; }
        for i in 1..n as isize {
            unsafe {
                *out_ptr.offset(i + 1) = G1::add_or_dbl(*out_ptr.offset(i - 1), &G1_GENERATOR);
            }
        }
        unsafe {
            let mut out = vec![G1::default(); 0];
            let mut i: isize = 0;
            while !out_ptr.offset(i).is_null() && i < n as isize {
                out.push(*out_ptr.offset(i) as G1);
                i += 1;
            }
            return out
        }
    }

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