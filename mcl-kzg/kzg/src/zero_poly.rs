use std::{cmp::min};
use crate::utilities::next_pow_of_2;
use crate::kzg10::Polynomial;
use crate::data_types::{fr::*};
use crate::fk20_fft::FFTSettings;


impl FFTSettings {
    // TODO: could be optimized by using mutable slices!
    pub fn zero_poly_via_multiplication(&self, indices: &[usize], length: usize) -> (Vec<Fr>, Vec<Fr>) {
        if indices.is_empty() {
            return (vec![Fr::zero(); length], vec![Fr::zero(); length]);
        }
        
        let stride = self.max_width / length;
        let per_leaf_poly = 64;
        let per_leaf = per_leaf_poly - 1;
        //TO DO fix the optimisation, possibly using unsafe static memory block
        if indices.len() <= per_leaf {
            let mut zero_poly = vec![Fr::default(); length];
            self.make_zero_poly_mul_leaf(&mut zero_poly, indices, stride);

            let zero_eval = self.fft(&zero_poly, false);
            return (zero_eval, zero_poly);
        }

        let leaf_count = (indices.len() + per_leaf - 1) / per_leaf;
        let n = next_pow_of_2(leaf_count * per_leaf_poly);

        // TODO: rust limitation, can't have multiple mutators for same value, code fails somewhere here, as I tried to achieve same func through duplicated value management.
        let mut out = vec![Fr::default(); n];
        let mut offset = 0;
        let mut out_offset = 0;
        let mut leaves: Vec<Vec<Fr>> = vec![vec![]; leaf_count];
        let max = indices.len();
        // for i in 0..leaf_count {
        for item in leaves.iter_mut().take(leaf_count) {
            let end = min(offset + per_leaf, max);
            *item = out[out_offset..out_offset + per_leaf_poly].to_vec();
            self.make_zero_poly_mul_leaf(item, &indices[offset..end], stride);
            let mut slice_copy = item.clone();
            out.append(&mut slice_copy);
            offset += per_leaf;
            out_offset += per_leaf_poly;
        }

        out = out[n..].to_vec();
        for _ in out.len()..n {
            out.push(Fr::zero());
        }

        let reduction_factor = 4;
        let mut scratch = vec![Fr::default(); n * 3];

        while leaves.len() > 1 {
            let reduced_count = (leaves.len() + reduction_factor - 1) / reduction_factor;
            let leaf_size = next_pow_of_2(leaves[0].len());
            for i in 0..reduced_count {
                let start = i * reduction_factor;
                let mut end = start + reduction_factor;

                let out_end = min(out.len(), end * leaf_size);
                let reduced = &mut out[start * leaf_size .. out_end].to_vec();
                end = min(end, leaves.len()); 

                let leaves_slice = &mut leaves[start..end];
                if end > start + 1 {
                    *reduced = self.reduce_leaves(&mut scratch, leaves_slice, reduced.len());
                }
                leaves[i] = reduced.to_vec();
            }
            leaves = leaves[..reduced_count].to_vec();
        }
        let zero_poly = Polynomial::extend(&leaves[0], length);
        let zero_eval = self.fft(&zero_poly, false);

        (zero_eval, zero_poly)
    }


    pub fn reduce_leaves(&self, scratch: &mut [Fr], ps: &[Vec<Fr>], n: usize) -> Vec<Fr> {
        let out_degree: usize = ps.iter()
            .map(|x| {
                if x.is_empty() { 
                    return 0; 
                } 
                x.len() - 1
            }).sum();
        let (p_padded, rest) = scratch.split_at_mut(n);
        let (mul_eval_ps, p_eval) = rest.split_at_mut(n);

        for item in p_padded.iter_mut() {
        // for i in 0..p_padded.len() {
            *item = Fr::zero();
        }
        for (i, v) in ps.last().unwrap().iter().enumerate() {
            p_padded[i] = *v;
        }

        //can optimize this, one alloc instead of three
        let temp = self.inplace_fft(p_padded, false);
        mul_eval_ps[..n].clone_from_slice(&temp[..n]);

        let last_index = ps.len() - 1;
        for item in ps.iter().take(last_index) {
            let p = item;
            p_padded[..p.len()].clone_from_slice(&p[..]);
            // p_eval = inplace_fft(p_padded);
            let p_eval_result = self.inplace_fft(p_padded, false);
            p_eval[..n].clone_from_slice(&p_eval_result[..n]);

            for j in 0..n {
                mul_eval_ps[j] *= &p_eval[j];
            }
        }

        let result = self.inplace_fft(mul_eval_ps, true);
        result[..out_degree + 1].to_vec()
    }
    
    pub fn make_zero_poly_mul_leaf(&self, dest: &mut Vec<Fr>, indices: &[usize], stride: usize) {
        if (indices.len() + 1) > dest.len() {
            panic!("expected bigger dest length");
        }

        dest[indices.len()] = Fr::one();
        
        for (i, v) in indices.iter().enumerate() {
            let neg_di = self.exp_roots_of_unity[v * stride].get_neg();
            dest[i] = neg_di;
            if i > 0 {
                let temp = dest[i] + dest[i - 1];
                dest[i] = temp;
                for j in (1..i).rev() {
                    dest[j] *= &neg_di;
                    let temp = dest[j] + dest[j - 1];
                    dest[j] = temp;
                }
                dest[0] *= &neg_di;
            }
        }
    }

}