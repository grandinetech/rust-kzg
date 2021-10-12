use super::{Error, Fr};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFTSettings {
    pub max_width: u64,
    pub root_of_unity: Fr,
    pub expanded_roots_of_unity: *mut Fr,
    pub reverse_roots_of_unity: *mut Fr
}

extern "C" {
    fn new_fft_settings(settings: *mut FFTSettings, max_scale: u32) -> Error;
    fn free_fft_settings(settings: *mut FFTSettings);
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
}


