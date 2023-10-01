extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use kzg::{FFTSettings, Fr};

use crate::consts::SCALE2_ROOT_OF_UNITY;
use crate::types::fr::FsFr;
use crate::utils::reverse_bit_order;

#[derive(Debug, Clone)]
pub struct FsFFTSettings {
    pub max_width: usize,
    pub root_of_unity: FsFr,
    pub expanded_roots_of_unity: Vec<FsFr>,
    pub reverse_roots_of_unity: Vec<FsFr>,
    pub roots_of_unity: Vec<FsFr>,
}

impl Default for FsFFTSettings {
    fn default() -> Self {
        Self::new(0).unwrap()
    }
}

impl FFTSettings<FsFr> for FsFFTSettings {
    /// Create FFTSettings with roots of unity for a selected scale. Resulting roots will have a magnitude of 2 ^ max_scale.
    fn new(scale: usize) -> Result<FsFFTSettings, String> {
        if scale >= SCALE2_ROOT_OF_UNITY.len() {
            return Err(String::from(
                "Scale is expected to be within root of unity matrix row size",
            ));
        }

        // max_width = 2 ^ max_scale
        let max_width: usize = 1 << scale;
        let root_of_unity = FsFr::from_u64_arr(&SCALE2_ROOT_OF_UNITY[scale]);

        // create max_width of roots & store them reversed as well
        let expanded_roots_of_unity = expand_root_of_unity(&root_of_unity, max_width)?;
        let mut reverse_roots_of_unity = expanded_roots_of_unity.clone();
        reverse_roots_of_unity.reverse();

        // Permute the roots of unity
        let mut roots_of_unity = expanded_roots_of_unity.clone();
        roots_of_unity.pop();
        reverse_bit_order(&mut roots_of_unity)?;

        Ok(FsFFTSettings {
            max_width,
            root_of_unity,
            expanded_roots_of_unity,
            reverse_roots_of_unity,
            roots_of_unity,
        })
    }

    fn get_max_width(&self) -> usize {
        self.max_width
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> FsFr {
        self.expanded_roots_of_unity[i]
    }

    fn get_expanded_roots_of_unity(&self) -> &[FsFr] {
        &self.expanded_roots_of_unity
    }

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> FsFr {
        self.reverse_roots_of_unity[i]
    }

    fn get_reversed_roots_of_unity(&self) -> &[FsFr] {
        &self.reverse_roots_of_unity
    }

    fn get_roots_of_unity_at(&self, i: usize) -> FsFr {
        self.roots_of_unity[i]
    }

    fn get_roots_of_unity(&self) -> &[FsFr] {
        &self.roots_of_unity
    }
}

/// Multiply a given root of unity by itself until it results in a 1 and result all multiplication values in a vector
pub fn expand_root_of_unity(root: &FsFr, width: usize) -> Result<Vec<FsFr>, String> {
    let mut generated_powers = vec![FsFr::one(), *root];

    while !(generated_powers.last().unwrap().is_one()) {
        if generated_powers.len() > width {
            return Err(String::from("Root of unity multiplied for too long"));
        }

        generated_powers.push(generated_powers.last().unwrap().mul(root));
    }

    if generated_powers.len() != width + 1 {
        return Err(String::from("Root of unity has invalid scale"));
    }

    Ok(generated_powers)
}
