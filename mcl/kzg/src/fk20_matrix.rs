use std::iter;
use crate::data_types::g1::G1;
use crate::data_types::fr::Fr;
use crate::utilities::*;
use crate::kzg10::*;
use crate::fk20_fft::*;
use crate::kzg_settings::KZGSettings;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

impl FFTSettings {
    pub fn toeplitz_part_1(&self, x: &[G1]) -> Result<Vec<G1>, String> {
        let n = x.len();

        // extend x with zeroes
        let tail= vec![G1::zero(); n];
        let x_ext: Vec<G1> = x.iter().cloned()
            .chain(tail)
            .collect();

        self.fft_g1(&x_ext)
    }

    pub fn toeplitz_part_2(&self, coeffs: &[Fr], x_ext_fft: &[G1]) -> Result<Vec<G1>, String> {
        let toeplitz_coeffs_fft = self.fft(coeffs, false).unwrap();

        #[cfg(feature = "parallel")]
        {
            let ret: Vec<_> = (0..coeffs.len()).into_par_iter().map(|i| {
                x_ext_fft[i] * &toeplitz_coeffs_fft[i]
            }).collect();
            Ok(ret)
        }

        #[cfg(not(feature = "parallel"))]
        {
            let mut ret = Vec::new();
            for i in 0..coeffs.len() {
                ret.push(x_ext_fft[i] * &toeplitz_coeffs_fft[i]);
            }
            Ok(ret)
        }
    }

    pub fn toeplitz_part_3(&self, h_ext_fft: &[G1]) -> Result<Vec<G1>, String> {
        let n2 = h_ext_fft.len();
        let n = n2 / 2;

        let mut ret = self.fft_g1_inv(h_ext_fft).unwrap();

        for item in ret.iter_mut().take(n2).skip(n) {
            *item = G1::G1_IDENTITY;
        }

        Ok(ret)
    }
}

#[derive(Debug, Clone, Default)]
pub struct FK20SingleMatrix {
    pub x_ext_fft: Vec<G1>,
    pub kzg_settings: KZGSettings
}

impl FK20SingleMatrix {
    pub fn new(kzg_settings: &KZGSettings, n2: usize) -> Result<Self, String> {
        let n = n2 >> 1; // div by 2

        if !is_power_of_2(n2) {
            return Err(String::from("n2 must be a power of two"));
        }
        if n2 < 2 {
            return Err(String::from("n2 must be greater than or equal to 2"));
        }
        if n2 > kzg_settings.fft_settings.max_width {
            return Err(String::from("n2 must be less than or equal to fft settings max width"));
        }

        let mut x = Vec::new();
        for i in 0..n - 1 {
            x.push(kzg_settings.curve.g1_points[n - 2 - i]);
        }
        x.push(G1::G1_IDENTITY);

        let x_ext_fft = kzg_settings.fft_settings.toeplitz_part_1(&x).unwrap();
        let kzg_settings = kzg_settings.clone();

        Ok(Self {
            kzg_settings,
            x_ext_fft
        })
    }

    pub fn dau_using_fk20_single(&self, polynomial: &Polynomial) -> Result<Vec<G1>, String> {
        let n = polynomial.order();
        let n2 = n << 1;

        if !is_power_of_2(n2) {
            return Err(String::from("n2 must be a power of two"));
        }
        if n2 > self.kzg_settings.fft_settings.max_width {
            return Err(String::from("n2 must be less than or equal to fft settings max width"));
        }

        let mut proofs = self.fk20_single_dao_optimized(polynomial).unwrap();
        order_by_rev_bit_order(&mut proofs);

        Ok(proofs)
    }

    pub fn fk20_single_dao_optimized(&self, polynomial: &Polynomial) -> Result<Vec<G1>, String> {
        let n = polynomial.order();
        let n2 = n * 2;

        if !is_power_of_2(n2) {
            return Err(String::from("n2 must be a power of two"));
        }
        if n2 > self.kzg_settings.fft_settings.max_width {
            return Err(String::from("n2 must be less than or equal to fft settings max width"));
        }

        let toeplitz_coeffs = polynomial.toeplitz_coeffs_step_strided(0, 1);
        let h_ext_fft = self.kzg_settings.fft_settings.toeplitz_part_2(&toeplitz_coeffs, &self.x_ext_fft).unwrap();
        let h = self.kzg_settings.fft_settings.toeplitz_part_3(&h_ext_fft).unwrap();

        self.kzg_settings.fft_settings.fft_g1(&h)
    }
}

#[derive(Debug, Clone)]
pub struct FK20Matrix {
    pub x_ext_fft_files: Vec<Vec<G1>>,
    pub chunk_len: usize,
    pub kzg_settings: KZGSettings
}

impl Default for FK20Matrix {
    fn default() -> Self {
        Self {
            kzg_settings: KZGSettings::default(),
            chunk_len: 1,
            x_ext_fft_files: vec![],
        }
    }
}

impl FK20Matrix {
    pub fn new(kzg_settings: &KZGSettings, n2: usize, chunk_len: usize) -> Result<Self, String> {
        let n = n2 >> 1; // div by 2
        let k = n / chunk_len;

        if !is_power_of_2(n2) {
            return Err(String::from("n2 must be a power of two"));
        }
        if !is_power_of_2(chunk_len) {
            return Err(String::from("chunk_len must be a power of two"));
        }
        if n2 < 2 {
            return Err(String::from("n2 must be greater than or equal to 2"));
        }
        if n2 > kzg_settings.fft_settings.max_width {
            return Err(String::from("n2 must be less than or equal to kzg settings max width"));
        }
        if n2 > kzg_settings.fft_settings.max_width {
            return Err(String::from("n2 must be less than or equal to fft settings max width"));
        }
        if chunk_len > n2 / 2 {
            return Err(String::from("chunk_len must be greater or equal to n2 / 2"));
        }

        let mut x_ext_fft_files: Vec<Vec<G1>> = vec![vec![]; chunk_len];
        for (i, item) in x_ext_fft_files.iter_mut().enumerate().take(chunk_len) {
            *item = FK20Matrix::x_ext_fft_precompute(&kzg_settings.fft_settings, &kzg_settings.curve, n, k,chunk_len, i).unwrap();
        }

        Ok(FK20Matrix {
            x_ext_fft_files,
            chunk_len,
            kzg_settings: kzg_settings.clone(),
        })
    }

    #[allow(clippy::many_single_char_names)]
    fn x_ext_fft_precompute(fft_settings: &FFTSettings, curve: &Curve, n: usize, k: usize, chunk_len: usize, offset: usize) -> Result<Vec<G1>, String> {
        let mut x: Vec<G1> = vec![G1::default(); k];

        let mut start = 0;
        let temp = chunk_len + 1 + offset;
        if n >= temp {
            start = n - temp;
        }

        let mut i = 0;
        let mut j = start + chunk_len;

        while i + 1 < k {
            // hack to remove overflow checking,
            // could just move this to the bottom and define j as start, but then need to check for overflows
            // basically last j -= chunk_len overflows, but it's not used to access the array, as the i + 1 < k is false
            j -= chunk_len;
            x[i] = curve.g1_points[j];
            i += 1;
        }

        x[k - 1] = G1::zero();

        fft_settings.toeplitz_part_1(&x)
    }

    pub fn dau_using_fk20_multi(&self, polynomial: &Polynomial)  -> Result<Vec<G1>, String> {
        let n = polynomial.order();
        let n2 = n << 1;

        if !is_power_of_2(n2) {
            return Err(String::from("n2 must be a power of two"));
        }
        if n2 > self.kzg_settings.fft_settings.max_width {
            return Err(String::from("n2 must be less than or equal to fft settings max width"));
        }

        let extended_poly = polynomial.get_extended(n2);
        let mut proofs = self.fk20_multi_dao_optimized(&extended_poly).unwrap();
        order_by_rev_bit_order(&mut proofs);

        Ok(proofs)
    }

    pub fn fk20_multi_dao_optimized(&self, polynomial: &Polynomial) -> Result<Vec<G1>, String> {
        let n = polynomial.order() >> 1;
        let k = n / self.chunk_len;
        let k2 = k << 1;

        let n2 = n << 1;
        if !is_power_of_2(n2) {
            return Err(String::from("n2 must be a power of two"));
        }
        if n2 > self.kzg_settings.fft_settings.max_width {
            return Err(String::from("n2 must be less than or equal to fft settings max width"));
        }

        let mut h_ext_fft = vec![G1::zero(); k2];
        // TODO: this operates on an extended poly, but doesn't use the extended values?
        // literally just using the poly without the zero trailing tail, makes more sense to take it in as a param, or use without the tail;
        let reduced_poly = Polynomial::from_fr(polynomial.coeffs.iter().copied().take(n).collect());

        for i in 0..self.chunk_len {
            let toeplitz_coeffs = reduced_poly.toeplitz_coeffs_step_strided(i, self.chunk_len);
            let h_ext_fft_file = self.kzg_settings.fft_settings.toeplitz_part_2(&toeplitz_coeffs, &self.x_ext_fft_files[i]).unwrap();

            for j in 0..k2 {
                let tmp = &h_ext_fft[j] + &h_ext_fft_file[j];
                h_ext_fft[j] = tmp;
            }
        }

        let tail = iter::repeat(G1::zero()).take(k);
        let h: Vec<G1> = self.kzg_settings.fft_settings.toeplitz_part_3(&h_ext_fft)
            .unwrap()
            .into_iter()
            .take(k)
            .chain(tail)
            .collect();

        self.kzg_settings.fft_settings.fft_g1(&h)
    }
}

impl Polynomial {
    pub fn extend(vec: &[Fr], size: usize) -> Vec<Fr> {
        if size < vec.len() {
            return vec.to_owned();
        }
        let to_pad = size - vec.len();
        let tail = iter::repeat(Fr::zero()).take(to_pad);
        let result: Vec<Fr> = vec.iter().copied().chain(tail).collect();

        result
    }

    pub fn get_extended(&self, size: usize) -> Polynomial {
        Polynomial::from_fr(Polynomial::extend(&self.coeffs, size))
    }

    fn toeplitz_coeffs_step_strided(&self, offset: usize, stride: usize) -> Vec<Fr> {
        let n = self.order();
        let k = n / stride;
        let k2 = k << 1;

        // [last] + [0]*(n+1) + [1 .. n-2]
        let mut toeplitz_coeffs = vec![Fr::zero(); k2];
        toeplitz_coeffs[0] = self.coeffs[n - 1 - offset];

        let mut j = (stride << 1) - offset - 1;
        for item in toeplitz_coeffs.iter_mut().take(k2).skip(k+2) {
            *item = self.coeffs[j];
            j += stride;
        }

        toeplitz_coeffs
    }
}
