use blst::blst_fr as BlstFr;
use super::scale2_root_of_unity;

pub struct FFTSettings {
    pub max_width: u64,
    pub root_of_unity: BlstFr,
    pub expanded_roots_of_unity: Vec<BlstFr>,
    pub reverse_roots_of_unity: Vec<BlstFr>,
}

impl Default for FFTSettings {
    fn default() -> FFTSettings {
        FFTSettings {
            max_width: 0,
            root_of_unity: BlstFr { l: [0, 0, 0, 0] },
            expanded_roots_of_unity: Vec::new(),
            reverse_roots_of_unity: Vec::new(),
        }
    }
}

impl FFTSettings {
    pub fn from_scale(max_scale: usize) -> Result<FFTSettings, String> {
        if max_scale >= scale2_root_of_unity.len() {
            return Err(String::from("Scale is expected to be within root of unity matrix row size"));
        }
        let max_width: u64 = 1 << max_scale;

        Ok(FFTSettings {
            max_width: max_width,
            ..Default::default()
        })
    }
}