use std::iter;
use crate::data_types::g1::mclBnG1_mulVec;
use crate::data_types::g1::G1;
use crate::data_types::fr::Fr;
use crate::utilities::*;
use crate::kzg10::*;
use crate::fk20_fft::*;

// KZG Settings + FK20 Settings + FFTSettings?
pub struct FK20Matrix {
    pub curve: Curve,
    pub x_ext_fft_files: Vec<Vec<G1>>,
    pub fft_settings: FFTSettings,
    pub chunk_len: usize,
}

impl FK20Matrix {
    
    pub fn new(curve: Curve, n2: usize, chunk_len: usize, fft_max_scale: u8) -> FK20Matrix {
        let n = n2 >> 1; // div by 2
        let k = n / chunk_len;
        let fft_settings = FFTSettings::new(fft_max_scale);
        if n2 > fft_settings.max_width {
            panic!("extended size is larger than fft settings supoort");
        }
        // TODO: more panic checks
        
        let mut x_ext_fft_files: Vec<Vec<G1>> = vec![vec![]; chunk_len];
        for (i, item) in x_ext_fft_files.iter_mut().enumerate().take(chunk_len) {
            *item = FK20Matrix::x_ext_fft_precompute(&fft_settings, &curve, n, k, chunk_len, i);
        }

        FK20Matrix {
            curve,
            x_ext_fft_files,
            fft_settings,
            chunk_len
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn x_ext_fft_precompute(fft_settings: &FFTSettings, curve: &Curve, n: usize, k: usize, chunk_len: usize, offset: usize) -> Vec<G1> {
        let mut x: Vec<G1> = vec![G1::default(); k];
        let start = n - chunk_len - offset - 1;

        let mut i = 0;
        let mut j = start + chunk_len;

        while i + 1 < k {
            // hack to remove overflow checking, 
            // could just move this to the bottom and define j as start, but then need to check for overflows
            // basically last j -= chunk_len overflows, but it's not used to access the array, as the i + 1 < k is false
            j -= chunk_len;
            x[i] = curve.g1_points[j].clone();
            i += 1;
        }
        
        x[k - 1] = G1::zero();

        FK20Matrix::toeplitz_part_1(fft_settings, &x)
    }

    pub fn toeplitz_part_1(fft_settings: &FFTSettings, x: &[G1]) -> Vec<G1> {
        let n = x.len();

        // extend x with zeroes
        let tail= vec![G1::zero(); n];
        let x_ext: Vec<G1> = x.iter().cloned()
            .chain(tail)
            .collect();

        
        
        FK20Matrix::fft_g1(fft_settings, &x_ext)
    }

    pub fn fft_g1(fft_settings: &FFTSettings, values: &[G1]) -> Vec<G1> {
        // TODO: check if copy can be removed, opt?
        // let vals_copy = values.clone();
        
        let root_z: Vec<Fr> = fft_settings.exp_roots_of_unity.iter()
            .take(fft_settings.max_width).copied()
            .collect();

        let stride = fft_settings.max_width /  values.len();
        let mut out = vec![G1::zero(); values.len()];

        FK20Matrix::_fft_g1(fft_settings, values, 0, 1, &root_z, stride, &mut out);

        out
    }

    //possible remove self, merge this methods with the ones in fft_settings
    pub fn fft_g1_inv(&self, values: &[G1]) -> Vec<G1> {
        // TODO: check if copy can be removed, opt?
        // let vals_copy = values.clone();
        
        let root_z: Vec<Fr> = self.fft_settings.exp_roots_of_unity_rev.iter()
            .take(self.fft_settings.max_width).copied()
            .collect();

        let stride = self.fft_settings.max_width /  values.len();
        let mut out = vec![G1::zero(); values.len()];

        FK20Matrix::_fft_g1(&self.fft_settings, values, 0, 1, &root_z, stride, &mut out);
        
        let inv_len = Fr::from_int(values.len() as i32).get_inv();
        for item in out.iter_mut() {
            *item = &*item * &inv_len;
        }

        out
    }

    pub fn dau_using_fk20_multi(&self, polynomial: &Polynomial) -> Vec<G1> {
        let n = polynomial.order();
        //TODO: checks? -> perfmance hit tho?
        let n2 = n << 1;
        let extended_poly = polynomial.get_extended(n2);

        let mut proofs = extended_poly.fk20_multi_dao_optimized(self);

        order_by_rev_bit_order(&mut proofs);

        proofs
    }

    fn _fft_g1(fft_settings: &FFTSettings, values: &[G1], value_offset: usize, value_stride: usize, roots_of_unity: &[Fr], roots_stride: usize, out: &mut [G1]) {
        //TODO: fine tune for opt, maybe resolve number dinamically based on experiments
        if out.len() <= 4 {
            return FK20Matrix::_fft_g1_simple(values, value_offset, value_stride, roots_of_unity, roots_stride, out);
        }

        let half = out.len() >> 1;

        // left
        FK20Matrix::_fft_g1(fft_settings, values, value_offset, value_stride << 1, roots_of_unity, roots_stride << 1, &mut out[..half]);
        // right
        FK20Matrix::_fft_g1(fft_settings, values, value_offset + value_stride, value_stride << 1, roots_of_unity, roots_stride << 1, &mut out[half..]);

        for i in 0..half {
            let x = out[i].clone();
            let y = out[i + half].clone();
            let root = &roots_of_unity[i * roots_stride];

            let y_times_root = &y * root;
            out[i] = &x + &y_times_root;
            out[i + half] = &x - &y_times_root;
        }

        
    }
    

    fn _fft_g1_simple(values: &[G1], value_offset: usize, value_stride: usize, roots_of_unity: &[Fr], roots_stride: usize, out: &mut [G1]) {
        let l = out.len();
        for i in 0..l {
            // TODO: check this logic with a working brain, there could be a simpler way to write this;
            let mut v = &values[value_offset] * &roots_of_unity[0];
            let mut last = v.clone();
            for j in 1..l {
                v = &values[value_offset + j * value_stride] * &roots_of_unity[((i * j) % l) * roots_stride];
                let temp = last.clone();
                last = &temp + &v;
            }
            out[i] = last;
        }
    }

    fn toeplitz_coeffs_step_strided(&self, poly: &[Fr], offset: usize) -> Vec<Fr> {
        let stride = self.chunk_len;
        let n = poly.len();
        let k = n / stride;
        let k2 = k << 1;

        // [last] + [0]*(n+1) + [1 .. n-2]
        let mut toeplitz_coeffs = vec![Fr::zero(); k2];
        toeplitz_coeffs[0] = poly[n - 1 - offset];
        
        let mut j = (stride << 1) - offset - 1;
        for item in toeplitz_coeffs.iter_mut().take(k2).skip(k+2) {
            *item = poly[j];
            j += stride;
        }

        toeplitz_coeffs
    }

    pub fn toeplitz_part_2(&self, coeffs: &[Fr], index: usize) -> Vec<G1> {
        let toeplitz_coeffs_fft = self.fft_settings.fft(coeffs, false);

        let x_ext_fft = &self.x_ext_fft_files[index];

        let h_ext_fft: Vec<G1> = x_ext_fft.iter()
            .zip(toeplitz_coeffs_fft)
            .map(|(g1, coeff)| g1 * &coeff)
            .collect();

        h_ext_fft
    }

    // TODO: optimization, reuse h_ext_fft
    pub fn toeplitz_part_3(&self, h_ext_fft: &[G1]) -> Vec<G1> {
        let out = self.fft_g1_inv(h_ext_fft);

        // return half, can just resize the vector to be half.
        return out.iter().take(out.len() >> 1).cloned().collect();
    }

    pub fn check_proof_multi(&self, commitment: &G1, proof: &G1, x: &Fr, ys: &[Fr]) -> bool {
        let mut interpolation_poly = self.fft_settings.fft(ys, true);
        let mut x_pow = Fr::one();
        for item in interpolation_poly.iter_mut() {
        // for i in 0.. interpolation_poly.len() {
            *item *= &x_pow.get_inv();
            x_pow *= x;
        }

        let xn2 = &self.curve.g2_gen * &x_pow;
        let xn_minus_yn = &self.curve.g2_points[ys.len()] - &xn2;

        let mut result = G1::default();
        unsafe {
            mclBnG1_mulVec(&mut result, self.curve.g1_points.as_ptr(), interpolation_poly.as_ptr(), interpolation_poly.len())
        };

        let commit_minus_interp = commitment - &result;

        Curve::verify_pairing(&commit_minus_interp, &self.curve.g2_gen, proof, &xn_minus_yn)
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

    pub fn fk20_multi_dao_optimized(&self, matrix: &FK20Matrix) -> Vec<G1> {
        let n = self.order() >> 1;
        let k = n / matrix.chunk_len;
        let k2 = k << 1;
        
        let mut h_ext_fft = vec![G1::zero(); k2];
        // TODO: this operates on an extended poly, but doesn't use the extended values?
        // literally just using the poly without the zero trailing tail, makes more sense to take it in as a param, or use without the tail;
        let reduced_poly: Vec<Fr> = self.coeffs.iter().copied().take(n).collect();

        for i in 0..matrix.chunk_len {
            let toeplitz_coeffs = matrix.toeplitz_coeffs_step_strided(&reduced_poly, i);
            let h_ext_fft_file = matrix.toeplitz_part_2(&toeplitz_coeffs, i);

            for j in 0..k2 {
                let tmp = &h_ext_fft[j] + &h_ext_fft_file[j];
                h_ext_fft[j] = tmp;
            }
        }
        
        let tail = iter::repeat(G1::zero()).take(k);
        let h: Vec<G1> = matrix.toeplitz_part_3(&h_ext_fft)
            .into_iter()
            .take(k)
            .chain(tail)
            .collect();
        
        FK20Matrix::fft_g1(&matrix.fft_settings, &h)
    }
}