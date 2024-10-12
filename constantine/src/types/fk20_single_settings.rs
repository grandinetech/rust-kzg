extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use kzg::common_utils::reverse_bit_order;
use kzg::{FK20SingleSettings, Poly, FFTG1, G1};

use crate::types::fft_settings::CtFFTSettings;
use crate::types::fr::CtFr;
use crate::types::g1::CtG1;
use crate::types::g2::CtG2;
use crate::types::kzg_settings::CtKZGSettings;
use crate::types::poly::CtPoly;

use super::fp::CtFp;
use super::g1::CtG1Affine;

#[derive(Clone, Default)]
pub struct CtFK20SingleSettings {
    pub kzg_settings: CtKZGSettings,
    pub x_ext_fft: Vec<CtG1>,
}

impl FK20SingleSettings<CtFr, CtG1, CtG2, CtFFTSettings, CtPoly, CtKZGSettings, CtFp, CtG1Affine>
    for CtFK20SingleSettings
{
    fn new(kzg_settings: &CtKZGSettings, n2: usize) -> Result<Self, String> {
        let n = n2 / 2;

        if n2 > kzg_settings.fs.max_width {
            return Err(String::from(
                "n2 must be less than or equal to kzg settings max width",
            ));
        } else if !n2.is_power_of_two() {
            return Err(String::from("n2 must be a power of two"));
        } else if n2 < 2 {
            return Err(String::from("n2 must be greater than or equal to 2"));
        }

        let mut x = Vec::with_capacity(n);
        for i in 0..n - 1 {
            x.push(kzg_settings.g1_values_lagrange_brp[n - 2 - i]);
        }
        x.push(CtG1::identity());

        let x_ext_fft = kzg_settings.fs.toeplitz_part_1(&x);
        drop(x);
        let kzg_settings = kzg_settings.clone();

        let ret = Self {
            kzg_settings,
            x_ext_fft,
        };

        Ok(ret)
    }

    fn data_availability(&self, p: &CtPoly) -> Result<Vec<CtG1>, String> {
        let n = p.len();
        let n2 = n * 2;

        if n2 > self.kzg_settings.fs.max_width {
            return Err(String::from(
                "n2 must be less than or equal to kzg settings max width",
            ));
        } else if !n2.is_power_of_two() {
            return Err(String::from("n2 must be a power of two"));
        }

        let mut ret = self.data_availability_optimized(p).unwrap();
        reverse_bit_order(&mut ret)?;

        Ok(ret)
    }

    fn data_availability_optimized(&self, p: &CtPoly) -> Result<Vec<CtG1>, String> {
        let n = p.len();
        let n2 = n * 2;

        if n2 > self.kzg_settings.fs.max_width {
            return Err(String::from(
                "n2 must be less than or equal to kzg settings max width",
            ));
        } else if !n2.is_power_of_two() {
            return Err(String::from("n2 must be a power of two"));
        }

        let toeplitz_coeffs = p.toeplitz_coeffs_step();

        let h_ext_fft = self
            .kzg_settings
            .fs
            .toeplitz_part_2(&toeplitz_coeffs, &self.x_ext_fft);

        let h = self.kzg_settings.fs.toeplitz_part_3(&h_ext_fft);

        let ret = self.kzg_settings.fs.fft_g1(&h, false).unwrap();

        Ok(ret)
    }
}
