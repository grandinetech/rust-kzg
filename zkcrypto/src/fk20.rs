use crate::fftsettings::ZkFFTSettings;
use crate::kzg_proofs::KZGSettings;
use crate::kzg_types::{ZkG1Projective, ZkG2Projective};
use crate::poly::ZPoly;
use crate::utils::*;
use crate::zkfr::blsScalar;
use kzg::{FFTFr, FK20MultiSettings, FK20SingleSettings, Poly, FFTG1, G1};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct ZkFK20SingleSettings {
    pub kzg_settings: KZGSettings,
    pub x_ext_fft: Vec<ZkG1Projective>,
}

impl
    FK20SingleSettings<blsScalar, ZkG1Projective, ZkG2Projective, ZkFFTSettings, ZPoly, KZGSettings>
    for ZkFK20SingleSettings
{
    fn new(kzg_settings: &KZGSettings, n: usize) -> Result<Self, String> {
        let n2 = n / 2;

        if n > kzg_settings.fs.max_width {
            return Err(String::from(
                "n must be less than or equal to kzg settings max width",
            ));
        } else if !is_power_of_two(n) {
            return Err(String::from("n must be a power of two"));
        } else if n < 2 {
            return Err(String::from("n must be greater than or equal to 2"));
        }

        let mut x = Vec::new();
        for i in 0..n2 - 1 {
            x.push(kzg_settings.secret_g1[n2 - 2 - i]);
        }
        x.push(ZkG1Projective::identity());

        let x_ext_fft = toeplitz_part_1(&x, &kzg_settings.fs);
        let kzg_settings = kzg_settings.clone();

        let out = Self {
            kzg_settings,
            x_ext_fft,
        };

        Ok(out)
    }

    fn data_availability(&self, p: &ZPoly) -> Result<Vec<ZkG1Projective>, String> {
        let n = p.len();
        let n2 = n * 2;

        if n2 > self.kzg_settings.fs.max_width {
            return Err(String::from(
                "n2 must be less than or equal to kzg settings max width",
            ));
        } else if !is_power_of_two(n2) {
            return Err(String::from("n2 must be a power of two"));
        }

        let mut out = self.data_availability_optimized(p).unwrap();
        reverse_bit_order(&mut out); // reverse bit order

        Ok(out)
    }

    fn data_availability_optimized(&self, p: &ZPoly) -> Result<Vec<ZkG1Projective>, String> {
        let n = p.len();
        let n2 = n * 2;

        if n2 > self.kzg_settings.fs.max_width {
            return Err(String::from(
                "n2 must be less than or equal to kzg settings max width",
            ));
        } else if !is_power_of_two(n2) {
            return Err(String::from("n2 must be a power of two"));
        }

        let toeplitz_coeffs = toeplitz_coeffs_step(p);
        let h_ext_fft = toeplitz_part_2(&toeplitz_coeffs, &self.x_ext_fft, &self.kzg_settings.fs);
        let h = toeplitz_part_3(&h_ext_fft, &self.kzg_settings.fs);

        let out = self.kzg_settings.fs.fft_g1(&h, false).unwrap();
        Ok(out)
    }
}

#[derive(Debug, Clone)]
pub struct ZkFK20MultiSettings {
    pub kzg_settings: KZGSettings,
    pub chunk_len: usize,
    pub x_ext_fft_files: Vec<Vec<ZkG1Projective>>,
}

impl Default for ZkFK20MultiSettings {
    fn default() -> Self {
        Self {
            kzg_settings: KZGSettings::default(),
            chunk_len: 1,
            x_ext_fft_files: vec![],
        }
    }
}

impl FK20MultiSettings<blsScalar, ZkG1Projective, ZkG2Projective, ZkFFTSettings, ZPoly, KZGSettings>
    for ZkFK20MultiSettings
{
    #[allow(clippy::many_single_char_names)]
    fn new(ks: &KZGSettings, n: usize, chunk_len: usize) -> Result<Self, String> {
        if n > ks.fs.max_width {
            return Err(String::from(
                "n must be less than or equal to kzg settings max width",
            ));
        } else if !is_power_of_two(n) {
            return Err(String::from("n must be a power of two"));
        } else if n < 2 {
            return Err(String::from("n must be greater than or equal to 2"));
        } else if chunk_len > n / 2 {
            return Err(String::from("chunk_len must be greater or equal to n / 2"));
        } else if !is_power_of_two(chunk_len) {
            return Err(String::from("chunk_len must be a power of two"));
        }

        let n2 = n / 2;
        let k = n2 / chunk_len;

        let mut ext_fft_files = Vec::new();

        for offset in 0..chunk_len {
            let mut x = Vec::new();

            let mut start = 0;
            if n2 >= chunk_len + 1 + offset {
                start = n2 - chunk_len - 1 - offset;
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
            x.push(ZkG1Projective::identity());

            let ext_fft_file = toeplitz_part_1(&x, &ks.fs);
            ext_fft_files.push(ext_fft_file);
        }

        let out = Self {
            kzg_settings: ks.clone(),
            chunk_len,
            x_ext_fft_files: ext_fft_files,
        };

        Ok(out)
    }

    fn data_availability(&self, p: &ZPoly) -> Result<Vec<ZkG1Projective>, String> {
        let n = p.len();
        let n2 = n * 2;

        if n2 > self.kzg_settings.fs.max_width {
            return Err(String::from(
                "n2 must be less than or equal to kzg settings max width",
            ));
        } else if !is_power_of_two(n2) {
            return Err(String::from("n2 must be a power of two"));
        }

        let mut out = self.data_availability_optimized(p).unwrap();
        reverse_bit_order(&mut out);

        Ok(out)
    }

    fn data_availability_optimized(&self, p: &ZPoly) -> Result<Vec<ZkG1Projective>, String> {
        let n = p.len();
        let n2 = n * 2;

        if n2 > self.kzg_settings.fs.max_width {
            return Err(String::from(
                "n2 must be less than or equal to kzg settings max width",
            ));
        } else if !is_power_of_two(n2) {
            return Err(String::from("n2 must be a power of two"));
        }

        let n = n2 / 2;
        let k = n / self.chunk_len;
        let k2 = k * 2;

        let mut h_ext_fft = vec![ZkG1Projective::identity(); k2];

        for i in 0..self.chunk_len {
            let toeplitz_coeffs = toeplitz_coeffs_stride(i, self.chunk_len, p);
            let h_ext_fft_file = toeplitz_part_2(
                &toeplitz_coeffs,
                &self.x_ext_fft_files[i],
                &self.kzg_settings.fs,
            );

            for j in 0..k2 {
                h_ext_fft[j] = h_ext_fft[j].add_or_dbl(&h_ext_fft_file[j]);
            }
        }

        let mut h = toeplitz_part_3(&h_ext_fft, &self.kzg_settings.fs);

        // for i in k..k2 {
        // h[i] = ZkG1Projective::identity();
        // }
        h[k..k2].copy_from_slice(&vec![ZkG1Projective::identity(); k2 - k]);

        let out = self.kzg_settings.fs.fft_g1(&h, false).unwrap();

        Ok(out)
    }
}

pub fn toeplitz_part_1(x: &[ZkG1Projective], fft_set: &ZkFFTSettings) -> Vec<ZkG1Projective> {
    let n2 = x.len() * 2;
    let mut x_ext = Vec::new();

    for &x_i in x[..x.len()].iter() {
        x_ext.push(x_i);
    }

    for _i in x.len()..n2 {
        x_ext.push(ZkG1Projective::identity());
    }

    let out = fft_set.fft_g1(&x_ext, false);
    out.unwrap()
}

pub fn toeplitz_part_2(
    toeplitz: &ZPoly,
    x_ext_fft: &[ZkG1Projective],
    fft_set: &ZkFFTSettings,
) -> Vec<ZkG1Projective> {
    let fft_coeffs = fft_set.fft_fr(&toeplitz.coeffs, false).unwrap();

    #[cfg(feature = "parallel")]
    {
        let out: Vec<_> = (0..toeplitz.len())
            .into_par_iter()
            .map(|i| x_ext_fft[i].mul(&fft_coeffs[i]))
            .collect();
        out
    }

    #[cfg(not(feature = "parallel"))]
    {
        let mut out = Vec::new();
        for i in 0..toeplitz.len() {
            out.push(x_ext_fft[i].mul(&fft_coeffs[i]));
        }
        out
    }
}

pub fn toeplitz_part_3(
    h_ext_fft: &[ZkG1Projective],
    fft_set: &ZkFFTSettings,
) -> Vec<ZkG1Projective> {
    // let n2 = h_ext_fft.len();
    let n = h_ext_fft.len() / 2;
    let mut out = fft_set.fft_g1(h_ext_fft, true).unwrap();

    // for i in n..h_ext_fft.len() {
    // out[i] = ZkG1Projective::identity();
    // }
    // out[n..h_ext_fft.len()].copy_from_slice(&[ZkG1Projective::identity()]);

    out[n..h_ext_fft.len()].copy_from_slice(&vec![ZkG1Projective::identity(); h_ext_fft.len() - n]);

    out
}

pub fn toeplitz_coeffs_stride(offset: usize, stride: usize, poly: &ZPoly) -> ZPoly {
    let n = poly.len();
    assert!(stride > 0);

    let k = n / stride;
    let k2 = k * 2;

    let mut out = ZPoly { coeffs: Vec::new() };

    out.coeffs.push(poly.coeffs[n - 1 - offset]);
    for _i in 1..min_u64(k + 2, k2).unwrap() {
        // is this good?
        out.coeffs.push(blsScalar::zero());
    }

    let mut j = 2 * stride - offset - 1;
    for _i in (k + 2)..k2 {
        // is this good?
        out.coeffs.push(poly.coeffs[j]);
        j += stride;
    }

    out
}

pub fn toeplitz_coeffs_step(poly: &ZPoly) -> ZPoly {
    toeplitz_coeffs_stride(0, 1, poly)
}

pub fn reverse_bit_order<T>(values: &mut [T])
where
    T: Clone,
{
    let unused_bit_len = values.len().leading_zeros() + 1;
    for i in 0..values.len() - 1 {
        let r = i.reverse_bits() >> unused_bit_len;
        if r > i {
            let tmp = values[r].clone();
            values[r] = values[i].clone();
            values[i] = tmp;
        }
    }
}
