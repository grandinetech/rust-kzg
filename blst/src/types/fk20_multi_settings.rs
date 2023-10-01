extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use kzg::{FK20MultiSettings, Poly, FFTG1, G1};

use crate::types::fft_settings::FsFFTSettings;
use crate::types::fr::FsFr;
use crate::types::g1::FsG1;
use crate::types::g2::FsG2;
use crate::types::kzg_settings::FsKZGSettings;
use crate::types::poly::FsPoly;
use crate::utils::reverse_bit_order;

pub struct FsFK20MultiSettings {
    pub kzg_settings: FsKZGSettings,
    pub chunk_len: usize,
    pub x_ext_fft_files: Vec<Vec<FsG1>>,
}

impl Clone for FsFK20MultiSettings {
    fn clone(&self) -> Self {
        Self {
            kzg_settings: self.kzg_settings.clone(),
            chunk_len: self.chunk_len,
            x_ext_fft_files: self.x_ext_fft_files.clone(),
        }
    }
}

impl Default for FsFK20MultiSettings {
    fn default() -> Self {
        Self {
            kzg_settings: FsKZGSettings::default(),
            chunk_len: 1,
            x_ext_fft_files: vec![],
        }
    }
}

impl FK20MultiSettings<FsFr, FsG1, FsG2, FsFFTSettings, FsPoly, FsKZGSettings>
    for FsFK20MultiSettings
{
    #[allow(clippy::many_single_char_names)]
    fn new(ks: &FsKZGSettings, n2: usize, chunk_len: usize) -> Result<Self, String> {
        if n2 > ks.fs.max_width {
            return Err(String::from(
                "n2 must be less than or equal to kzg settings max width",
            ));
        } else if !n2.is_power_of_two() {
            return Err(String::from("n2 must be a power of two"));
        } else if n2 < 2 {
            return Err(String::from("n2 must be greater than or equal to 2"));
        } else if chunk_len > n2 / 2 {
            return Err(String::from("chunk_len must be greater or equal to n2 / 2"));
        } else if !chunk_len.is_power_of_two() {
            return Err(String::from("chunk_len must be a power of two"));
        }

        let n = n2 / 2;
        let k = n / chunk_len;

        let mut ext_fft_files = Vec::with_capacity(chunk_len);
        {
            let mut x = Vec::with_capacity(k);
            for offset in 0..chunk_len {
                let mut start = 0;
                if n >= chunk_len + 1 + offset {
                    start = n - chunk_len - 1 - offset;
                }

                let mut i = 0;
                let mut j = start;

                while i + 1 < k {
                    x.push(ks.secret_g1[j]);

                    i += 1;

                    if j >= chunk_len {
                        j -= chunk_len;
                    } else {
                        j = 0;
                    }
                }
                x.push(FsG1::identity());

                let ext_fft_file = ks.fs.toeplitz_part_1(&x);
                x.clear();
                ext_fft_files.push(ext_fft_file);
            }
        }

        let ret = Self {
            kzg_settings: ks.clone(),
            chunk_len,
            x_ext_fft_files: ext_fft_files,
        };

        Ok(ret)
    }

    fn data_availability(&self, p: &FsPoly) -> Result<Vec<FsG1>, String> {
        let n = p.len();
        let n2 = n * 2;

        if n2 > self.kzg_settings.fs.max_width {
            return Err(String::from(
                "n2 must be less than or equal to kzg settings max width",
            ));
        }

        if !n2.is_power_of_two() {
            return Err(String::from("n2 must be a power of two"));
        }

        let mut ret = self.data_availability_optimized(p).unwrap();
        reverse_bit_order(&mut ret)?;

        Ok(ret)
    }

    fn data_availability_optimized(&self, p: &FsPoly) -> Result<Vec<FsG1>, String> {
        let n = p.len();
        let n2 = n * 2;

        if n2 > self.kzg_settings.fs.max_width {
            return Err(String::from(
                "n2 must be less than or equal to kzg settings max width",
            ));
        } else if !n2.is_power_of_two() {
            return Err(String::from("n2 must be a power of two"));
        }

        let n = n2 / 2;
        let k = n / self.chunk_len;
        let k2 = k * 2;

        let mut h_ext_fft = vec![FsG1::identity(); k2];

        for i in 0..self.chunk_len {
            let toeplitz_coeffs = p.toeplitz_coeffs_stride(i, self.chunk_len);
            let h_ext_fft_file = self
                .kzg_settings
                .fs
                .toeplitz_part_2(&toeplitz_coeffs, &self.x_ext_fft_files[i]);

            for j in 0..k2 {
                h_ext_fft[j] = h_ext_fft[j].add_or_dbl(&h_ext_fft_file[j]);
            }
        }

        let mut h = self.kzg_settings.fs.toeplitz_part_3(&h_ext_fft);

        h[k..k2].copy_from_slice(&vec![FsG1::identity(); k2 - k]);

        let ret = self.kzg_settings.fs.fft_g1(&h, false).unwrap();

        Ok(ret)
    }
}
