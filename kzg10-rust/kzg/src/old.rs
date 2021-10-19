use std::{cmp::min, iter, ops, usize, vec};

use crate::data_types::fr::Fr;
use crate::data_types::fp::Fp;
use crate::data_types::fp2::Fp2;
use crate::data_types::g1::G1;
use crate::data_types::g1::mclBnG1_mulVec;
use crate::data_types::g2::G2;
use crate::data_types::gt::GT;
use crate::mcl_methods::*;
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
        for i in 0..chunk_len {
            x_ext_fft_files[i] = FK20Matrix::x_ext_fft_precompute(&fft_settings, &curve, n, k, chunk_len,i);
        }

        FK20Matrix {
            curve,
            x_ext_fft_files,
            fft_settings,
            chunk_len
        }
    }
    
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

        return FK20Matrix::toeplitz_part_1(&fft_settings, &x);
    }

    pub fn toeplitz_part_1(fft_settings: &FFTSettings, x: &Vec<G1>) -> Vec<G1> {
        let n = x.len();

        // extend x with zeroes
        let tail= vec![G1::zero(); n];
        let x_ext: Vec<G1> = x.iter()
            .map(|g1| g1.clone())
            .chain(tail)
            .collect();

        let x_ext_fft = FK20Matrix::fft_g1(&fft_settings, &x_ext);
        
        return x_ext_fft;
    }

    pub fn fft_g1(fft_settings: &FFTSettings, values: &Vec<G1>) -> Vec<G1> {
        // TODO: check if copy can be removed, opt?
        let vals_copy = values.clone();
        
        let root_z: Vec<Fr> = fft_settings.exp_roots_of_unity.iter()
            .take(fft_settings.max_width)
            .map(|x| x.clone())
            .collect();

        let stride = fft_settings.max_width /  values.len();
        let mut out = vec![G1::zero(); values.len()];

        FK20Matrix::_fft_g1(&fft_settings, &vals_copy, 0, 1, &root_z, stride, &mut out);

        return out;
    }

 
    pub fn fft_g1_inv(&self, values: &Vec<G1>) -> Vec<G1> {
        // TODO: check if copy can be removed, opt?
        let vals_copy = values.clone();
        
        let root_z: Vec<Fr> = self.fft_settings.exp_roots_of_unity_rev.iter()
            .take(self.fft_settings.max_width)
            .map(|x| x.clone())
            .collect();

        let stride = self.fft_settings.max_width /  values.len();
        let mut out = vec![G1::zero(); values.len()];

        FK20Matrix::_fft_g1(&self.fft_settings, &vals_copy, 0, 1, &root_z, stride, &mut out);
        
        let inv_len = Fr::from_int(values.len() as i32).get_inv();
        for i in 0..out.len() {
            let tmp = &out[i] * &inv_len;
            out[i] = tmp;
        }

        return out;
    }

    pub fn dau_using_fk20_multi(&self, polynomial: &Polynomial) -> Vec<G1> {
        let n = polynomial.order();
        //TODO: checks? -> perfmance hit tho?
        let n2 = n << 1;
        let extended_poly = polynomial.get_extended(n2);

        let mut proofs = extended_poly.fk20_multi_dao_optimized(&self);

        order_by_rev_bit_order(&mut proofs);

        return proofs;
    }

    fn _fft_g1(fft_settings: &FFTSettings, values: &Vec<G1>, value_offset: usize, value_stride: usize, roots_of_unity: &Vec<Fr>, roots_stride: usize, out: &mut [G1]) {
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

            let y_times_root = &y * &root;
            out[i] = &x + &y_times_root;
            out[i + half] = &x - &y_times_root;
        }

        return;
    }
    

    fn _fft_g1_simple(values: &Vec<G1>, value_offset: usize, value_stride: usize, roots_of_unity: &Vec<Fr>, roots_stride: usize, out: &mut [G1]) {
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

    fn toeplitz_coeffs_step_strided(&self, poly: &Vec<Fr>, offset: usize) -> Vec<Fr> {
        let stride = self.chunk_len;
        let n = poly.len();
        let k = n / stride;
        let k2 = k << 1;

        // [last] + [0]*(n+1) + [1 .. n-2]
        let mut toeplitz_coeffs = vec![Fr::zero(); k2];
        toeplitz_coeffs[0] = poly[n - 1 - offset].clone();
        
        let mut j = (stride << 1) - offset - 1;
        for i in k+2..k2 {
            toeplitz_coeffs[i] = poly[j].clone();
            j += stride;
        }

        return toeplitz_coeffs;
    }

    pub fn toeplitz_part_2(&self, coeffs: &Vec<Fr>, index: usize) -> Vec<G1> {
        let toeplitz_coeffs_fft = self.fft_settings.fft(&coeffs, false);

        let x_ext_fft = &self.x_ext_fft_files[index];

        let h_ext_fft: Vec<G1> = x_ext_fft.iter()
            .zip(toeplitz_coeffs_fft)
            .map(|(g1, coeff)| g1 * &coeff)
            .collect();

        return h_ext_fft;
    }

    // TODO: optimization, reuse h_ext_fft
    pub fn toeplitz_part_3(&self, h_ext_fft: &Vec<G1>) -> Vec<G1> {
        let out = self.fft_g1_inv(&h_ext_fft);

        // return half, can just resize the vector to be half.
        return out.iter().take(out.len() >> 1).map(|x| x.clone()).collect();
    }

    pub fn check_proof_multi(&self, commitment: &G1, proof: &G1, x: &Fr, ys: &Vec<Fr>) -> bool {
        let mut interpolation_poly = self.fft_settings.fft(&ys, true);
        let mut x_pow = Fr::one();
        for i in 0.. interpolation_poly.len() {
            interpolation_poly[i] *= &x_pow.get_inv();
            x_pow *= x;
        }

        let xn2 = &self.curve.g2_gen * &x_pow;
        let xn_minus_yn = &self.curve.g2_points[ys.len()] - &xn2;

        let mut result = G1::default();
        unsafe {
            mclBnG1_mulVec(&mut result, self.curve.g1_points.as_ptr(), interpolation_poly.as_ptr(), interpolation_poly.len())
        };

        let commit_minus_interp = commitment - &result;

        return self.curve.verify_pairing(&commit_minus_interp, &self.curve.g2_gen, &proof, &&xn_minus_yn);
    }
}

impl Polynomial {
    pub fn extend(vec: &Vec<Fr>, size: usize) -> Vec<Fr> {
        let to_pad = size - vec.len();
        let tail = iter::repeat(Fr::zero()).take(to_pad);
        let result: Vec<Fr> = vec.iter().map(|x| x.clone()).chain(tail).collect();

        return result;
    }

    pub fn get_extended(&self, size: usize) -> Polynomial { 
        return Polynomial::from_fr(Polynomial::extend(&self.coeffs, size));
    }

    pub fn fk20_multi_dao_optimized(&self, matrix: &FK20Matrix) -> Vec<G1> {
        let n = self.order() >> 1;
        let k = n / matrix.chunk_len;
        let k2 = k << 1;
        
        let mut h_ext_fft = vec![G1::zero(); k2];
        // TODO: this operates on an extended poly, but doesn't use the extended values?
        // literally just using the poly without the zero trailing tail, makes more sense to take it in as a param, or use without the tail;
        let reduced_poly: Vec<Fr> = self.coeffs.iter().map(|x| x.clone()).take(n).collect();

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
        
        return FK20Matrix::fft_g1(&matrix.fft_settings, &h);
    }
}

// DAS
impl FFTSettings {
    pub fn das_fft_extension(&self, values: &mut Vec<Fr>) {
        if (values.len() << 1) > self.max_width {
            panic!("ftt_settings max width too small!");
        }

        self._das_fft_extension(values, 1);
        
        // just dividing every value by 1/(2**depth) aka length
        // TODO: what's faster, maybe vec[x] * vec[x], ask herumi to implement?
        let inv_length = Fr::from_int(values.len() as i32).get_inv();
        for i in 0..values.len() {
            values[i] *= &inv_length;
        }
    }

    fn _das_fft_extension(&self, values: &mut [Fr], stride: usize) {
        if values.len() == 2 {
            let (x, y) = FFTSettings::_calc_add_and_sub(&values[0], &values[1]);

            let temp = &y * &self.exp_roots_of_unity[stride];
            values[0] = &x + &temp;
            values[1] = &x - &temp;
            return;
        }

        let length = values.len();
        let half = length >> 1;
        
        // let ab_half_0s = ab[..quarter];
        // let ab_half_1s = ab[quarter..];
        for i in 0..half {
            let (add, sub) = FFTSettings::_calc_add_and_sub(&values[i], &values[half + i]);
            values[half + i] = &sub * &self.exp_roots_of_unity_rev[(i << 1) * stride];
            values[i] = add;
        }

        // left
        self._das_fft_extension(&mut values[..half], stride << 1);
        // right
        self._das_fft_extension(&mut values[half..], stride << 1);

        for i in 0..half {
            let root = &self.exp_roots_of_unity[((i << 1) + 1) * stride];
            let y_times_root = &values[half + i] * root;

            let (add, sub) = FFTSettings::_calc_add_and_sub(&values[i], &y_times_root);
            values[i] = add;
            values[i + half] = sub;
        }
    }

    fn _calc_add_and_sub(a: &Fr, b: &Fr) -> (Fr, Fr) {
        return (a + b, a - b);
    }
}

// Data recovery

impl Polynomial {
    pub fn shift_in_place(&mut self) {
        self._shift_in_place(&Fr::from_int(PRIMITIVE_ROOT));
    }

    pub fn unshift_in_place(&mut self) {
        self._shift_in_place(&Fr::from_int(PRIMITIVE_ROOT).get_inv());
    }

    //TODO, use precalculated tables for factors?
    fn _shift_in_place(&mut self, factor: &Fr){
        let mut factor_to_power = Fr::one();
        for i in 0..self.order() {
            self.coeffs[i] *= &factor_to_power;
            factor_to_power *= factor;
        }
    }

    pub fn recover_from_samples(fft_settings: FFTSettings, samples: &[Option<Fr>]) -> Polynomial {
        let missing_data_indices: Vec<usize> = samples.iter()
            .enumerate()
            .filter(|(_, ex)| ex.is_none())
            .map(|(ix, _)| ix)
            .collect();

        let (zero_eval, zero_poly_coeffs) = fft_settings.zero_poly_via_multiplication(&missing_data_indices, samples.len());

        // TODO: possible optimization, remove clone()
        let poly_evals_with_zero: Vec<Fr> = samples.iter()
            .zip(zero_eval)
            .map(|(x, eval)| {
                if x.is_none() {
                    return Fr::zero();
                }
                return &x.clone().unwrap() * &eval;
            }).collect();

        // for val in poly_evals_with_zero {
        //     println!("{}", val.get_str(10));
        // }

        let poly_with_zero_coeffs = fft_settings.fft(&poly_evals_with_zero, true);
        let mut poly_with_zero = Polynomial::from_fr(poly_with_zero_coeffs);
        poly_with_zero.shift_in_place();

        let mut zero_poly = Polynomial::from_fr(zero_poly_coeffs);
        zero_poly.shift_in_place();

        let eval_shifted_poly_with_zero = fft_settings.fft(&poly_with_zero.coeffs, false);
        let eval_shifted_zero_poly = fft_settings.fft(&zero_poly.coeffs, false);
        
    
        let eval_shifted_reconstructed_poly: Vec<Fr> = eval_shifted_poly_with_zero.iter()
            .zip(eval_shifted_zero_poly)
            .map(|(a, b)| a / &b)
            .collect();

        let shifted_reconstructed_poly_coeffs = fft_settings.fft(&eval_shifted_reconstructed_poly, true);
        let mut shifted_reconstructed_poly = Polynomial::from_fr(shifted_reconstructed_poly_coeffs);
        shifted_reconstructed_poly.unshift_in_place();

        let reconstructed_data = fft_settings.fft(&shifted_reconstructed_poly.coeffs, false);
        
        return Polynomial::from_fr(reconstructed_data);
    }

    pub fn unwrap_default(values: &Vec<Option<Fr>>) -> Vec<Fr> {
        return values.iter().map(|x| {
            if x.is_none() {
                return Fr::zero()
            }
            return x.clone().unwrap();
        }).collect();
    }
}