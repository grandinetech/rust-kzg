use super::{Error, Fr, G1, Poly};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFTSettings {
    pub max_width: usize,
    pub root_of_unity: Fr,
    pub expanded_roots_of_unity: *mut Fr,
    pub reverse_roots_of_unity: *mut Fr
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct BlstFp {
    pub l: [u64; 6]
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct BlstFp2 {
    pub fp: [BlstFp; 2]
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct BlstP2 {
    pub x: BlstFp2,
    pub y: BlstFp2,
    pub z: BlstFp2
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct BlstP1 {
    pub x: BlstFp,
    pub y: BlstFp,
    pub z: BlstFp
}

extern "C" {
    fn new_fft_settings(settings: *mut FFTSettings, max_scale: u32) -> Error;
    fn free_fft_settings(settings: *mut FFTSettings);
    fn fft_fr(output: *mut Fr, input: *const Fr, inverse: bool, n: u64, fs: *const FFTSettings) -> Error;
    fn fft_g1(output: *mut G1, input: *mut G1, inverse: bool, n: u64, fs: *const FFTSettings) -> Error;
    fn poly_mul(output: *mut Poly, a: *const Poly, b: *const Poly, fs: *const FFTSettings) -> Error;
}

impl FFTSettings {
    pub fn default() -> Self {
        Self {
            max_width: 16,
            root_of_unity: Fr::default(),
            expanded_roots_of_unity: &mut Fr::default(),
            reverse_roots_of_unity: &mut Fr::default()
        }
    }

    pub fn new(max_scale: u32) -> Result<Self, Error> {
        let mut settings = FFTSettings::default();
        unsafe {
            return match new_fft_settings(&mut settings, max_scale) {
                Error::KzgOk => Ok(settings),
                e => {
                    println!("Error in \"FFTSettings::new\" ==> {:?}", e);
                    Err(e)
                }
            }
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            free_fft_settings(self);
        }
    }

    pub fn fft(&self, input: &mut Vec<Fr>, inverse: bool) -> Vec<Fr> {
        let output = match FFTSettings::fft_fr(input.as_mut_ptr(), inverse, input.len() as u64, self) {
            Ok(fr) => fr,
            Err(e) => panic!("Error in \"FFTSettings::fft\" ==> {:?}", e)
        };
        output
    }

    fn fft_fr(input: *const Fr, inverse: bool, n: u64, fs: *const FFTSettings) -> Result<Vec<Fr>, Error> {
        let mut output = vec![Fr::default(); n as usize];
        unsafe {
            return match fft_fr(output.as_mut_ptr(), input, inverse, n, fs) {
                Error::KzgOk => Ok(output),
                e => {
                    println!("Error in \"FFTSettings::fft_fr\" ==> {:?}", e);
                    Err(e)
                }
            }
        }
    }

    pub fn fft_g1(input: *mut G1, inverse: bool, n: u64, fs: *const FFTSettings) -> Result<G1, Error> {
        let mut output = G1::default();
        unsafe {
            return match fft_g1(&mut output, input, inverse, n, fs) {
                Error::KzgOk => Ok(output),
                e => {
                    println!("Error in \"FFTSettings::fft_g1\" ==> {:?}", e);
                    Err(e)
                }
            }
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
}
