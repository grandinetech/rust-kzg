use crate::fftsettings::ZkFFTSettings;
use crate::poly::{pad, ZPoly};
use crate::utils::is_power_of_two;
use crate::zkfr::blsScalar;
use kzg::{FFTFr, ZeroPoly};
use std::cmp::min;
use std::ops::Neg;

// pub(crate) fn pad_poly(poly: &ZPoly, new_length: usize) -> Result<Vec<blsScalar>, String> {
// if new_length < poly.coeffs.len() {
// return Err(String::from("Wanted length less than current"));
// }

// let mut out = poly.coeffs.to_vec();

// for _i in poly.coeffs.len()..new_length {
// out.push(blsScalar::zero())
// }

// Ok(out)
// }

impl ZeroPoly<blsScalar, ZPoly> for ZkFFTSettings {
    #[allow(clippy::needless_range_loop)]
    fn do_zero_poly_mul_partial(&self, indices: &[usize], stride: usize) -> Result<ZPoly, String> {
        if indices.is_empty() {
            //  == 0
            return Err(String::from("index array length mustnt be zero"));
        }
        let mut poly = ZPoly {
            coeffs: vec![blsScalar::one(); indices.len() + 1],
        };
        poly.coeffs[0] = (self.expanded_roots_of_unity[indices[0] * stride]).neg();

        for i in 1..indices.len() {
            let neg_di = (self.expanded_roots_of_unity[indices[i] * stride]).neg();
            poly.coeffs[i] = neg_di;

            poly.coeffs[i] = poly.coeffs[i].add(&poly.coeffs[i - 1]);

            let mut j = i - 1;
            while j > 0 {
                poly.coeffs[j] = poly.coeffs[j].mul(&neg_di);
                poly.coeffs[j] = poly.coeffs[j].add(&poly.coeffs[j - 1]);
                j -= 1;
            }

            poly.coeffs[0] = poly.coeffs[0].mul(&neg_di);
        }

        Ok(poly)
    }
    #[allow(clippy::needless_range_loop)]
    fn reduce_partials(&self, length: usize, partials: &[ZPoly]) -> Result<ZPoly, String> {
        let mut out_degree: usize = 0;
        for partial in partials {
            // 0..partials.len()
            out_degree += partial.coeffs.len() - 1;
        }

        if !is_power_of_two(length) {
            return Err(String::from("Expected length to be power of two"));
        }

        if out_degree + 1 > length {
            return Err(String::from(
                "Expected out_degree to be within possible polynomial size",
            ));
        }

        let mut p_padded = pad(&partials[0], length);
        let mut mul_eval_ps = self.fft_fr(&p_padded, false).unwrap();

        for i in 1..(partials.len()) {
            p_padded = pad(&partials[i], length);

            let p_eval = self.fft_fr(&p_padded, false).unwrap();
            for j in 0..length {
                mul_eval_ps[j] = mul_eval_ps[j].mul(&p_eval[j]);
            }
        }

        let coeffs = self.fft_fr(&mul_eval_ps, true).unwrap();

        let out = ZPoly {
            coeffs: coeffs[..(out_degree + 1)].to_vec(),
        };

        Ok(out)
    }
    #[allow(clippy::comparison_chain)]
    fn zero_poly_via_multiplication(
        &self,
        length: usize,
        missing_indices: &[usize],
    ) -> Result<(Vec<blsScalar>, ZPoly), String> {
        let zero_eval: Vec<blsScalar>;
        let mut zero_poly: ZPoly;

        if missing_indices.is_empty() {
            // .len() == 0
            zero_eval = Vec::new();
            zero_poly = ZPoly { coeffs: Vec::new() };
            return Ok((zero_eval, zero_poly));
        }

        if missing_indices.len() >= length {
            return Err(String::from("Missing indexes are greater than domain size"));
        } else if length > self.max_width {
            return Err(String::from(
                "Domain size is greater than fft_settings.max_width",
            ));
        } else if !length.is_power_of_two() {
            return Err(String::from("Domain size must be a power of 2"));
        }

        let degree_of_partial = 256;
        let missing_per_partial = degree_of_partial - 1;
        let domain_stride = self.max_width / length;
        let mut partial_count =
            (missing_per_partial + missing_indices.len() - 1) / missing_per_partial;
        let domain_ceiling = min(
            (partial_count * degree_of_partial).next_power_of_two(),
            length,
        );

        if missing_indices.len() <= missing_per_partial {
            zero_poly = self
                .do_zero_poly_mul_partial(missing_indices, domain_stride)
                .unwrap();
        } else {
            let mut work =
                vec![blsScalar::zero(); (partial_count * degree_of_partial).next_power_of_two()];

            let mut partial_lens = Vec::new();
            let mut partial_offsets = Vec::new();

            let mut offset = 0;
            let mut out_offset = 0;
            let max = missing_indices.len();

            for _i in 0..partial_count {
                let end = min(offset + missing_per_partial, max);

                let mut partial =
                    self.do_zero_poly_mul_partial(&missing_indices[offset..end], domain_stride)?;
                partial.coeffs = pad(&partial, degree_of_partial);
                work.splice(
                    out_offset..(out_offset + degree_of_partial),
                    partial.coeffs.to_vec(),
                );
                partial_lens.push(degree_of_partial);
                partial_offsets.push(out_offset);

                offset += missing_per_partial;
                out_offset += degree_of_partial;
            }

            partial_lens[partial_count - 1] =
                1 + missing_indices.len() - (partial_count - 1) * missing_per_partial;

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
                    partial_offsets[i] = start * partial_size;
                    for j in 0..(partials_num) {
                        partial_offsets[i + j] = (start + j) * partial_size;
                        partial_vec.push(ZPoly {
                            coeffs: work[partial_offsets[i + j]
                                ..(partial_offsets[i + j] + partial_lens[i + j])]
                                .to_vec(),
                        });
                    }

                    if partials_num > 1 {
                        let mut reduced_poly = self.reduce_partials(reduced_len, &partial_vec)?;
                        partial_lens[i] = reduced_poly.coeffs.len();
                        reduced_poly.coeffs = pad(&reduced_poly, partial_size * partials_num);
                        work.splice(
                            (partial_offsets[i])..(partial_offsets[i] + reduced_poly.coeffs.len()),
                            reduced_poly.coeffs,
                        );
                    } else {
                        partial_lens[i] = partial_lens[start];
                    }
                }

                partial_count = reduced_count;
            }

            zero_poly = ZPoly { coeffs: work };
        }

        if zero_poly.coeffs.len() < length {
            zero_poly.coeffs = pad(&zero_poly, length);
        } else if zero_poly.coeffs.len() > length {
            zero_poly.coeffs = zero_poly.coeffs[..length].to_vec();
        }

        zero_eval = self.fft_fr(&zero_poly.coeffs, false)?;

        Ok((zero_eval, zero_poly))
    }
}
