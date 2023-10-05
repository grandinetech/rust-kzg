use crate::consts::G1_IDENTITY;
use crate::kzg_proofs::{FFTSettings, KZGSettings};
use crate::kzg_types::{ArkG1, ArkG2, FsFr as BlstFr};
use crate::utils::PolyData;
use kzg::common_utils::reverse_bit_order;
use kzg::{FFTFr, FK20MultiSettings, FK20SingleSettings, Fr, G1Mul, Poly, FFTG1, G1};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct KzgFK20SingleSettings {
    pub ks: KZGSettings,
    pub x_ext_fft: Vec<ArkG1>,
    pub x_ext_fft_len: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct KzgFK20MultiSettings {
    pub ks: KZGSettings,
    pub chunk_len: usize,
    pub x_ext_fft_files: Vec<Vec<ArkG1>>,
    pub length: usize,
}

impl FK20SingleSettings<BlstFr, ArkG1, ArkG2, FFTSettings, PolyData, KZGSettings>
    for KzgFK20SingleSettings
{
    fn new(ks: &KZGSettings, n2: usize) -> Result<Self, String> {
        let n = n2 / 2;

        if n2 > ks.fs.max_width {
            return Err(String::from(
                "n2 must be equal or less than kzg settings max width",
            ));
        }
        if !n2.is_power_of_two() {
            return Err(String::from("n2 must be power of 2"));
        }
        if n2 < 2 {
            return Err(String::from("n2 must be equal or greater than 2"));
        }

        let mut x = Vec::new();
        for i in 0..(n - 1) {
            x.push(ks.secret_g1[n - 2 - i])
        }
        x.push(G1_IDENTITY);

        let new_ks = KZGSettings {
            fs: ks.fs.clone(),
            ..KZGSettings::default()
        };

        Ok(KzgFK20SingleSettings {
            ks: new_ks,
            x_ext_fft: toeplitz_part_1(&x, &ks.fs).unwrap(),
            x_ext_fft_len: n2,
        })
    }

    fn data_availability(&self, p: &PolyData) -> Result<Vec<ArkG1>, String> {
        let n = p.len();
        let n2 = n * 2;

        if n2 > self.ks.fs.max_width {
            return Err(String::from(
                "n2 must be equal or less than kzg settings max width",
            ));
        }
        if !n.is_power_of_two() {
            return Err(String::from("n2 must be power of 2"));
        }

        let mut out = fk20_single_da_opt(p, self).unwrap();
        reverse_bit_order(&mut out)?;
        Ok(out)
    }

    fn data_availability_optimized(&self, p: &PolyData) -> Result<Vec<ArkG1>, String> {
        fk20_single_da_opt(p, self)
    }
}

impl FK20MultiSettings<BlstFr, ArkG1, ArkG2, FFTSettings, PolyData, KZGSettings>
    for KzgFK20MultiSettings
{
    fn new(ks: &KZGSettings, n2: usize, chunk_len: usize) -> Result<Self, String> {
        if n2 > ks.fs.max_width {
            return Err(String::from(
                "n2 must be equal or less than kzg settings max width",
            ));
        }
        if !n2.is_power_of_two() {
            return Err(String::from("n2 must be power of 2"));
        }
        if n2 < 2 {
            return Err(String::from("n2 must be equal or greater than 2"));
        }
        if chunk_len > n2 / 2 {
            return Err(String::from("chunk_len must be equal or less than n2/2"));
        }
        if !chunk_len.is_power_of_two() {
            return Err(String::from("chunk_len must be power of 2"));
        }
        if chunk_len == 0 {
            return Err(String::from("chunk_len must be greater than 0"));
        }

        let n = n2 / 2;
        let k = n / chunk_len;

        let mut x_ext_fft_files = Vec::new();

        for offset in 0..chunk_len {
            let mut x = vec![ArkG1::default(); k];
            let start = if n >= chunk_len + 1 + offset {
                n - chunk_len - 1 - offset
            } else {
                0
            };
            let mut j = start;
            for i in x.iter_mut().take(k - 1) {
                i.0 = ks.secret_g1[j].0;
                if j >= chunk_len {
                    j -= chunk_len;
                } else {
                    j = 0;
                }
            }
            x[k - 1] = G1_IDENTITY;
            x_ext_fft_files.push(toeplitz_part_1(&x, &ks.fs).unwrap());
        }

        let new_ks = KZGSettings {
            fs: ks.fs.clone(),
            ..KZGSettings::default()
        };

        Ok(KzgFK20MultiSettings {
            ks: new_ks,
            x_ext_fft_files,
            chunk_len,
            length: n, //unsure if this is right
        })
    }

    fn data_availability(&self, p: &PolyData) -> Result<Vec<ArkG1>, String> {
        let n = p.len();
        let n2 = n * 2;

        if n2 > self.ks.fs.max_width {
            return Err(String::from(
                "n2 must be equal or less than kzg settings max width",
            ));
        }
        if !n.is_power_of_two() {
            return Err(String::from("n2 must be power of 2"));
        }

        let mut out = fk20_multi_da_opt(p, self).unwrap();
        reverse_bit_order(&mut out)?;
        Ok(out)
    }

    fn data_availability_optimized(&self, p: &PolyData) -> Result<Vec<ArkG1>, String> {
        fk20_multi_da_opt(p, self)
    }
}

fn fk20_single_da_opt(p: &PolyData, fk: &KzgFK20SingleSettings) -> Result<Vec<ArkG1>, String> {
    let n = p.len();
    let n2 = n * 2;

    if n2 > fk.ks.fs.max_width {
        return Err(String::from(
            "n2 must be equal or less than kzg settings max width",
        ));
    }
    if !n.is_power_of_two() {
        return Err(String::from("n2 must be power of 2"));
    }

    let outlen = 2 * p.len();
    let toeplitz_coeffs = toeplitz_coeffs_step(p, outlen).unwrap();
    let h_ext_fft = toeplitz_part_2(&toeplitz_coeffs, &fk.x_ext_fft, &fk.ks.fs).unwrap();
    let h = toeplitz_part_3(&h_ext_fft, &fk.ks.fs).unwrap();

    fk.ks.fs.fft_g1(&h, false)
}

fn fk20_multi_da_opt(p: &PolyData, fk: &KzgFK20MultiSettings) -> Result<Vec<ArkG1>, String> {
    let n = p.len();
    let n2 = n * 2;

    if n2 > fk.ks.fs.max_width {
        return Err(String::from(
            "n2 must be equal or less than kzg settings max width",
        ));
    }
    if !n.is_power_of_two() {
        return Err(String::from("n2 must be power of 2"));
    }

    let n = n2 / 2;
    let k = n / fk.chunk_len;
    let k2 = k * 2;

    let mut h_ext_fft = Vec::new();
    for _i in 0..k2 {
        h_ext_fft.push(G1_IDENTITY);
    }

    let mut toeplitz_coeffs = PolyData::new(n2 / fk.chunk_len).unwrap();
    for i in 0..fk.chunk_len {
        toeplitz_coeffs =
            toeplitz_coeffs_stride(p, i, fk.chunk_len, toeplitz_coeffs.len()).unwrap();
        let h_ext_fft_file =
            toeplitz_part_2(&toeplitz_coeffs, &fk.x_ext_fft_files[i], &fk.ks.fs).unwrap();
        for j in 0..k2 {
            h_ext_fft[j] = h_ext_fft[j].add_or_dbl(&h_ext_fft_file[j]);
        }
    }

    // Calculate `h`
    let mut h = toeplitz_part_3(&h_ext_fft, &fk.ks.fs).unwrap();

    // Overwrite the second half of `h` with zero
    for i in h.iter_mut().take(k2).skip(k) {
        i.0 = G1_IDENTITY.0;
    }

    fk.ks.fs.fft_g1(&h, false)
}

fn toeplitz_coeffs_step(p: &PolyData, outlen: usize) -> Result<PolyData, String> {
    toeplitz_coeffs_stride(p, 0, 1, outlen)
}

fn toeplitz_coeffs_stride(
    poly: &PolyData,
    offset: usize,
    stride: usize,
    outlen: usize,
) -> Result<PolyData, String> {
    let n = poly.len();

    if stride == 0 {
        return Err(String::from("stride must be greater than 0"));
    }

    let k = n / stride;
    let k2 = k * 2;

    if outlen < k2 {
        return Err(String::from("outlen must be equal or greater than k2"));
    }

    let mut out = PolyData::new(outlen).unwrap();
    out.set_coeff_at(0, &poly.coeffs[n - 1 - offset]);
    let mut i = 1;
    while i <= (k + 1) && i < k2 {
        out.set_coeff_at(i, &BlstFr::zero());
        i += 1;
    }
    let mut j = 2 * stride - offset - 1;
    for i in (k + 2)..k2 {
        out.set_coeff_at(i, &poly.coeffs[j]);
        j += stride;
    }
    Ok(out)
}

fn toeplitz_part_1(x: &[ArkG1], fs: &FFTSettings) -> Result<Vec<ArkG1>, String> {
    let n = x.len();
    let n2 = n * 2;

    let mut x_ext = Vec::new();
    for i in x.iter().take(n) {
        x_ext.push(*i);
    }
    for _i in n..n2 {
        x_ext.push(G1_IDENTITY);
    }
    fs.fft_g1(&x_ext, false)
}

fn toeplitz_part_2(
    toeplitz_coeffs: &PolyData,
    x_ext_fft: &[ArkG1],
    fs: &FFTSettings,
) -> Result<Vec<ArkG1>, String> {
    let toeplitz_coeffs_fft = fs.fft_fr(&toeplitz_coeffs.coeffs, false).unwrap();

    #[cfg(feature = "parallel")]
    {
        let out: Vec<_> = (0..toeplitz_coeffs.len())
            .into_par_iter()
            .map(|i| x_ext_fft[i].mul(&toeplitz_coeffs_fft[i]))
            .collect();
        Ok(out)
    }

    #[cfg(not(feature = "parallel"))]
    {
        let mut out = Vec::new();
        for i in 0..toeplitz_coeffs.len() {
            out.push(x_ext_fft[i].mul(&toeplitz_coeffs_fft[i]));
        }
        Ok(out)
    }
}

fn toeplitz_part_3(h_ext_fft: &[ArkG1], fs: &FFTSettings) -> Result<Vec<ArkG1>, String> {
    let n = h_ext_fft.len() / 2;
    let mut out = fs.fft_g1(h_ext_fft, true).unwrap();

    // Zero the second half of h
    for i in out.iter_mut().take(h_ext_fft.len()).skip(n) {
        i.0 = G1_IDENTITY.0;
    }
    Ok(out)
}
