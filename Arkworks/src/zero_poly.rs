use super::kzg_proofs::FFTSettings;
use super::utils::{
    blst_fr_into_pc_fr, blst_poly_into_pc_poly, pc_fr_into_blst_fr, pc_poly_into_blst_poly,
    PolyData,
};
use crate::kzg_types::FsFr as BlstFr;
use kzg::{FFTFr, Fr as FrTrait, ZeroPoly};
use std::cmp::{min, Ordering};
use std::ops::Neg;

pub(crate) fn pad_poly(poly: &PolyData, new_length: usize) -> Result<Vec<BlstFr>, String> {
    if new_length <= poly.coeffs.len() {
        return Ok(poly.coeffs.clone());
    }

    let mut out = poly.coeffs.to_vec();

    for _i in poly.coeffs.len()..new_length {
        out.push(BlstFr::zero())
    }

    Ok(out)
}

impl ZeroPoly<BlstFr, PolyData> for FFTSettings {
    fn do_zero_poly_mul_partial(
        &self,
        indices: &[usize],
        len_indices: usize,
        stride: usize,
    ) -> Result<PolyData, String> {
        if indices.is_empty(){
            return Err(String::from("idx array must be non-zero"));
        }
        let blstpoly = PolyData {
            coeffs: vec![BlstFr::one(); indices.len() + 1],
        };
        let mut poly = blst_poly_into_pc_poly(&blstpoly).unwrap();
        poly.coeffs[0] =
            blst_fr_into_pc_fr(&self.expanded_roots_of_unity[indices[0] * stride]).neg();

        for (i, indice) in indices.iter().enumerate().take(len_indices).skip(1) {
            let neg_di =
                blst_fr_into_pc_fr(&self.expanded_roots_of_unity[indice * stride]).neg();

            poly.coeffs[i] = neg_di + poly.coeffs[i - 1];

            let mut j = i - 1;
            while j > 0 {
                let temp = poly.coeffs[j] * neg_di;
                poly.coeffs[j] = temp + poly.coeffs[j - 1];
                j -= 1;
            }

            poly.coeffs[0] *= neg_di;
        }

        Ok(pc_poly_into_blst_poly(poly).unwrap())
    }

    fn reduce_partials(&self, len_out: usize, partials: &[PolyData]) -> Result<PolyData, String> {
        let mut out_degree: usize = 0;
        for partial in partials  {
            out_degree += partial.coeffs.len() - 1;
        }

        if out_degree + 1 > len_out {
            return Err(String::from("Expected domain size to be a power of 2"));
        }

        let mut p_partial = pad_poly(&partials[0], len_out).unwrap();
        let mut mul_eval_ps = self.fft_fr(&p_partial, false).unwrap();

        for partial in partials.iter().skip(1) {
            p_partial = pad_poly(partial, len_out)?;

            let p_eval = self.fft_fr(&p_partial, false).unwrap();
            for j in 0..len_out {
                mul_eval_ps[j] = pc_fr_into_blst_fr(
                    blst_fr_into_pc_fr(&mul_eval_ps[j]) * blst_fr_into_pc_fr(&p_eval[j]),
                );
            }
        }

        let coeffs = self.fft_fr(&mul_eval_ps, true)?;

        let out = PolyData {
            coeffs: coeffs[..(out_degree + 1)].to_vec(),
        };

        Ok(out)
    }

    // Partially modified from team 'From-scratch'
    fn zero_poly_via_multiplication(
        &self,
        length: usize,
        missing_indices: &[usize],
        len_indices: usize
    ) -> Result<(Vec<BlstFr>, PolyData), String> {
        let zero_eval: Vec<BlstFr>;
        let mut zero_poly: PolyData;

        if len_indices == 0 {
            zero_eval = Vec::new();
            zero_poly = PolyData { coeffs: Vec::new() };
            return Ok((zero_eval, zero_poly));
        }

        if len_indices >= length {
            return Err(String::from("Missing idxs greater than domain size"));
        } else if length > self.max_width as usize {
            return Err(String::from(
                "Domain size greater than fft_settings.max_width",
            ));
        } else if !length.is_power_of_two() {
            return Err(String::from("Domain size must be a power of 2"));
        }

        let degree_of_partial = 64;
        let missing_per_partial = degree_of_partial - 1;
        let domain_stride = self.max_width as usize / length;
        let mut partial_count =
            (missing_per_partial + len_indices - 1) / missing_per_partial;
        let domain_ceiling = min(
            (partial_count * degree_of_partial).next_power_of_two(),
            length,
        );

        if len_indices <= missing_per_partial {
            zero_poly = self.do_zero_poly_mul_partial(missing_indices, len_indices, domain_stride)?;
        } else {
            let mut work =
                vec![BlstFr::zero(); (partial_count * degree_of_partial).next_power_of_two()];

            let mut partial_lens = Vec::new();

            let mut offset = 0;
            let mut out_offset = 0;
            let max = len_indices;

            for _i in 0..partial_count {
                let end = min(offset + missing_per_partial, max);

                let mut partial =
                    self.do_zero_poly_mul_partial(&missing_indices[offset..end], end - offset, domain_stride)?;
                partial.coeffs = pad_poly(&partial, degree_of_partial)?;
                work.splice(
                    out_offset..(out_offset + degree_of_partial),
                    partial.coeffs.to_vec(),
                );
                partial_lens.push(degree_of_partial);

                offset += missing_per_partial;
                out_offset += degree_of_partial;
            }

            partial_lens[partial_count - 1] =
                1 + len_indices - (partial_count - 1) * missing_per_partial;

            let reduction_factor = 4;
            while partial_count > 1 {
                let reduced_count = 1 + (partial_count - 1) / reduction_factor;
                let partial_size = (partial_lens[0]).next_power_of_two();

                for i in 0..reduced_count {
                    let start = i * reduction_factor;
                    let out_end = min((start + reduction_factor) * partial_size, domain_ceiling);
                    let reduced_len = min(out_end - start * partial_size, length);
                    let partials_num = min(reduction_factor, partial_count - start);

                    let mut partial_vec = Vec::new();
                    for j in 0..partials_num {
                        let k = (start + j) * partial_size;
                        partial_vec.push(PolyData {
                            coeffs: work[k..(k + partial_lens[i + j])].to_vec()});
                    }

                    if partials_num > 1 {
                        let mut reduced_poly = self.reduce_partials(reduced_len, &partial_vec)?;
                        partial_lens[i] = reduced_poly.coeffs.len();
                        reduced_poly.coeffs = pad_poly(&reduced_poly, partial_size * partials_num)?;
                        work.splice(
                            (start * partial_size)..(start * partial_size + reduced_poly.coeffs.len()),
                            reduced_poly.coeffs,
                        );
                    } else {
                        partial_lens[i] = partial_lens[start];
                    }
                }

                partial_count = reduced_count;
            }

            zero_poly = PolyData { coeffs: work };
        }

        match zero_poly.coeffs.len().cmp(&length){
            Ordering::Less => zero_poly.coeffs = pad_poly(&zero_poly, length)?,
            Ordering::Greater => zero_poly.coeffs = zero_poly.coeffs[..length].to_vec(),
            Ordering::Equal => {}
        }

        zero_eval = self.fft_fr(&zero_poly.coeffs, false)?;

        Ok((zero_eval, zero_poly))
    }
}
