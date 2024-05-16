use crate::data_types::fr::Fr;
use crate::fk20_fft::FFTSettings;
use crate::kzg10::Polynomial;
use kzg::common_utils::{is_power_of_2, next_pow_of_2};
use std::cmp::min;

///  Copy all of the coefficients of polynomial @p p to @p out, padding to length @p p_len with zeros.
pub fn pad_poly(new_length: usize, poly: &Polynomial) -> Result<Vec<Fr>, String> {
    if new_length < poly.order() {
        return Err(String::from(
            "new_length must not be shorter then given poly's length",
        ));
    }

    let mut ret = poly.coeffs.to_vec();

    for _ in poly.order()..new_length {
        ret.push(Fr::zero())
    }

    Ok(ret)
}

impl FFTSettings {
    /// Calculates the minimal polynomial that evaluates to zero for powers of roots of unity at the given indices.
    /// The returned polynomial has a length of `idxs.len() + 1`.
    ///
    /// Uses straightforward long multiplication to calculate the product of `(x - r^i)` where `r` is a root of unity
    /// and the `i`s are the indices at which it must evaluate to zero.
    pub fn do_zero_poly_mul_partial(
        &self,
        indices: &[usize],
        stride: usize,
    ) -> Result<Polynomial, String> {
        if indices.is_empty() {
            return Err(String::from("Missing indices array must not be empty"));
        }

        let mut poly = Polynomial::from_fr(vec![Fr::one(); indices.len() + 1]);
        poly.coeffs[0] = self.expanded_roots_of_unity[indices[0] * stride].get_neg();

        for (i, item) in indices.iter().enumerate().skip(1) {
            poly.coeffs[i] = self.expanded_roots_of_unity[item * stride].get_neg();
            let neg_di = poly.coeffs[i];
            poly.coeffs[i] = poly.coeffs[i] + poly.coeffs[i - 1];
            for j in (1..i).rev() {
                poly.coeffs[j] = poly.coeffs[j] * neg_di + poly.coeffs[j - 1];
            }

            poly.coeffs[0] = poly.coeffs[0] * neg_di;
        }

        Ok(poly)
    }

    /// Reduce partials using a specified domain size.
    /// Calculates the product of all polynomials via FFT and then applies an inverse FFT to produce a new Polynomial.
    pub fn reduce_partials(
        &self,
        len_out: usize,
        partials: &[Polynomial],
    ) -> Result<Polynomial, String> {
        if !is_power_of_2(len_out) {
            return Err(String::from("len out must be a power of two"));
        }

        // The degree of the output polynomial is the sum of the degrees of the input polynomials.
        let mut out_degree: usize = 0;
        for item in partials {
            out_degree += item.order() - 1;
        }

        if out_degree + 1 > len_out {
            return Err(String::from("Out degree is longer than out len"));
        }

        // Do the last partial first: it is no longer than the others and the padding can remain in place for the rest.
        let mut partial_padded = pad_poly(len_out, &partials[partials.len() - 1]).unwrap();
        let mut eval_result = self.fft(&partial_padded, false).unwrap();

        for item in partials.iter().take(partials.len() - 1) {
            partial_padded = pad_poly(len_out, item).unwrap();
            let evaluated_partial = self.fft(&partial_padded, false).unwrap();

            for j in 0..len_out {
                eval_result[j] = eval_result[j] * evaluated_partial[j];
            }
        }

        let coeffs = self.fft(&eval_result, true).unwrap();
        let ret = Polynomial::from_fr(coeffs[..(out_degree + 1)].to_vec());

        Ok(ret)
    }
    /// Calculate the minimal polynomial that evaluates to zero for powers of roots of unity that correspond to missing
    /// indices.
    /// This is done simply by multiplying together `(x - r^i)` for all the `i` that are missing indices, using a
    /// combination of direct multiplication ([`Self::do_zero_poly_mul_partial()`]) and iterated multiplication via
    /// convolution (#reduce_partials).
    /// Also calculates the FFT (the "evaluation polynomial").
    pub fn zero_poly_via_multiplication(
        &self,
        length: usize,
        missing_indices: &[usize],
    ) -> Result<(Vec<Fr>, Polynomial), String> {
        let zero_eval: Vec<Fr>;
        let mut zero_poly: Polynomial;

        if missing_indices.is_empty() {
            zero_eval = vec![Fr::zero(); length];
            zero_poly = Polynomial::from_fr(vec![Fr::zero(); length]);
            return Ok((zero_eval, zero_poly));
        }

        if missing_indices.len() >= length {
            return Err(String::from("Missing indice count is bigger than length"));
        }
        if length > self.max_width {
            return Err(String::from("Length is bigger than fft_settings.max_width"));
        }
        if !is_power_of_2(length) {
            return Err(String::from("Length must be a power of 2"));
        }

        let degree_of_partial = 256; // Tunable parameter. Must be a power of two.
        let missing_per_partial = degree_of_partial - 1;
        let domain_stride = self.max_width / length;
        let mut partial_count = 1 + (missing_indices.len() - 1) / missing_per_partial;
        let domain_ceiling = min(next_pow_of_2(partial_count * degree_of_partial), length);

        if missing_indices.len() <= missing_per_partial {
            zero_poly = self
                .do_zero_poly_mul_partial(missing_indices, domain_stride)
                .unwrap();
        } else {
            // Work space for building and reducing the partials
            let mut work = vec![Fr::zero(); next_pow_of_2(partial_count * degree_of_partial)];

            // Build the partials from the missing indices
            // Just allocate pointers here since we're re-using `work` for the partial processing
            // Combining partials can be done mostly in-place, using a scratchpad.
            let mut partial_lens = vec![];
            let mut partial_offsets = vec![];
            let mut missing_offset = 0;
            let mut work_offset = 0;
            let max = missing_indices.len();

            for _ in 0..partial_count {
                let end = min(missing_offset + missing_per_partial, max);

                let mut partial = self
                    .do_zero_poly_mul_partial(&missing_indices[missing_offset..end], domain_stride)
                    .unwrap();
                partial.coeffs = pad_poly(degree_of_partial, &partial).unwrap();
                work.splice(
                    work_offset..(work_offset + degree_of_partial),
                    partial.coeffs.to_vec(),
                );
                partial_lens.push(degree_of_partial);
                partial_offsets.push(work_offset);

                missing_offset += missing_per_partial;
                work_offset += degree_of_partial;
            }
            // Adjust the length of the last partial
            partial_lens[partial_count - 1] =
                1 + missing_indices.len() - (partial_count - 1) * missing_per_partial;

            // Reduce all the partials to a single polynomial
            let reduction_factor = 4; // must be a power of 2 (for sake of the FFTs in reduce_partials)
            while partial_count > 1 {
                let reduced_count = 1 + (partial_count - 1) / reduction_factor;
                let partial_size = next_pow_of_2(partial_lens[0]);

                for i in 0..reduced_count {
                    let start = i * reduction_factor;
                    let out_end = min((start + reduction_factor) * partial_size, domain_ceiling);
                    let reduced_len = min(out_end - start * partial_size, length);
                    let partials_num = min(reduction_factor, partial_count - start);

                    let mut partial_vec = Vec::new();
                    partial_offsets[i] = start * partial_size;
                    for j in 0..(partials_num) {
                        partial_offsets[i + j] = (start + j) * partial_size;
                        partial_vec.push(Polynomial::from_fr(
                            work[partial_offsets[i + j]
                                ..(partial_offsets[i + j] + partial_lens[i + j])]
                                .to_vec(),
                        ));
                    }

                    if partials_num > 1 {
                        let mut reduced_poly =
                            self.reduce_partials(reduced_len, &partial_vec).unwrap();
                        partial_lens[i] = reduced_poly.order();
                        reduced_poly.coeffs =
                            pad_poly(partial_size * partials_num, &reduced_poly).unwrap();
                        work.splice(
                            (partial_offsets[i])..(partial_offsets[i] + reduced_poly.order()),
                            reduced_poly.coeffs,
                        );
                    } else {
                        partial_lens[i] = partial_lens[start];
                    }
                }

                partial_count = reduced_count;
            }

            zero_poly = Polynomial::from_fr(work);
        }

        zero_poly.pad_coeffs_mut(zero_poly.order(), length);
        zero_eval = self.fft(&zero_poly.coeffs, false).unwrap();

        Ok((zero_eval, zero_poly))
    }
}
