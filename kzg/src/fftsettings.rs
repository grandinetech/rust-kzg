use super::{Error, Fr, G1};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFTSettings {
    pub max_width: u64,
    pub root_of_unity: Fr,
    pub expanded_roots_of_unity: *mut Fr,
    pub reverse_roots_of_unity: *mut Fr
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct blst_fp {
    pub l: [u64; 6usize],
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct blst_p1 {
    pub x: blst_fp,
    pub y: blst_fp,
    pub z: blst_fp,
}

extern "C" {
    fn new_fft_settings(settings: *mut FFTSettings, max_scale: u32) -> Error;
    fn free_fft_settings(settings: *mut FFTSettings);
    fn fft_fr(output: *mut Fr, input: *mut Fr, inverse: bool, n: u64, fs: *const FFTSettings) -> Error;
    fn fft_g1(output: *mut G1, input: *mut G1, inverse: bool, n: u64, fs: *const FFTSettings) -> Error;
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
                    println!("Error in \"FFTSettings::ckzg_new_fft_settings\" ==> {:?}", e);
                    Err(e)
                }
            }
        }
    }

    pub fn free(settings: *mut FFTSettings) {
        unsafe {
            free_fft_settings(settings);
        }
    }


    pub fn fft_fr(input: *mut Fr, inverse: bool, n: u64, fs: *const FFTSettings) -> Result<Fr, Error> {
        let mut output = Fr::default();

        unsafe {
            return match fft_fr(&mut output, input, inverse, n, fs) {
                Error::KzgOk => Ok(output),
                e => {
                    println!("Error in \"FFTSettings::ckzg_new_fft_settings\" ==> {:?}", e);
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
                    println!("Error in \"FFTSettings::ckzg_new_fft_settings\" ==> {:?}", e);
                    Err(e)
                }
            }
        }
    }
}


