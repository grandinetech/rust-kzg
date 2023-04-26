extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::{min, Ordering};

use kzg::{FFTFr, Fr, ZeroPoly};

use crate::types::fft_settings::FsFFTSettings;
use crate::types::fr::FsFr;
use crate::types::poly::FsPoly;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Pad given poly it with zeros to new length
pub fn pad_poly(mut poly: Vec<FsFr>, new_length: usize) -> Result<Vec<FsFr>, String> {
    if new_length < poly.len() {
        return Err(String::from(
            "new_length must be longer or equal to poly length",
        ));
    }

    poly.resize(new_length, FsFr::zero());

    Ok(poly)
}

#[allow(clippy::needless_range_loop)]
impl ZeroPoly<FsFr, FsPoly> for FsFFTSettings {
    /// Calculates a polynomial that evaluates to zero for roots of unity at given indices.
    /// The returned polynomial has a length of idxs.len() + 1.
    fn do_zero_poly_mul_partial(&self, idxs: &[usize], stride: usize) -> Result<FsPoly, String> {
        if idxs.is_empty() {
            return Err(String::from("idx array must not be empty"));
        }

        // Makes use of long multiplication in terms of (x - w_0)(x - w_1)..
        // Initialize poly with 1s
        let mut poly = FsPoly {
            coeffs: vec![FsFr::one(); idxs.len() + 1],
        };

        // For the first member, store -w_0 as constant term
        poly.coeffs[0] = self.expanded_roots_of_unity[idxs[0] * stride].negate();

        for i in 1..idxs.len() {
            // For member (x - w_i) take coefficient as -(w_i + w_{i-1} + ...)
            poly.coeffs[i] = self.expanded_roots_of_unity[idxs[i] * stride].negate();
            let neg_di = poly.coeffs[i];
            poly.coeffs[i] = poly.coeffs[i].add(&poly.coeffs[i - 1]);

            // Multiply all previous members by (x - w_i)
            // It equals multiplying by - w_i and adding x^(i - 1) coefficient (implied multiplication by x)
            let mut j = i - 1;
            while j > 0 {
                poly.coeffs[j] = poly.coeffs[j].mul(&neg_di);
                poly.coeffs[j] = poly.coeffs[j].add(&poly.coeffs[j - 1]);
                j -= 1;
            }

            // Multiply x^0 member by - w_i
            poly.coeffs[0] = poly.coeffs[0].mul(&neg_di);
        }

        Ok(poly)
    }

    /// Reduce partials using a specified domain size.
    /// Calculates the product of all polynomials via FFT and then applies an inverse FFT to produce a new Polynomial.
    fn reduce_partials(&self, domain_size: usize, partials: &[FsPoly]) -> Result<FsPoly, String> {
        if !domain_size.is_power_of_two() {
            return Err(String::from("Expected domain size to be a power of 2"));
        }

        // Calculate the resulting polynomial degree
        // E.g. (a * x^n + ...) (b * x^m + ...) has a degree of x^(n+m)
        let out_degree = partials
            .iter()
            .map(|partial| partial.coeffs.len() - 1)
            .sum::<usize>();

        if out_degree + 1 > domain_size {
            return Err(String::from(
                "Out degree is longer than possible polynomial size in domain",
            ));
        }

        // Pad all partial polynomials to same length, compute their FFT and multiply them together
        let mut padded_partial = pad_poly(partials[0].coeffs.clone(), domain_size)?;
        let mut eval_result = self.fft_fr(&padded_partial, false)?;

        for i in 1..(partials.len()) {
            padded_partial = pad_poly(partials[i].coeffs.clone(), domain_size)?;
            let evaluated_partial = self.fft_fr(&padded_partial, false)?;
            for j in 0..domain_size {
                eval_result[j] = eval_result[j].mul(&evaluated_partial[j]);
            }
        }

        // Apply an inverse FFT to produce a new poly. Limit its size to out_degree + 1
        let ret = FsPoly {
            coeffs: self
                .fft_fr(&eval_result, true)?
                .into_iter()
                .take(out_degree + 1)
                .collect(),
        };

        Ok(ret)
    }

    fn zero_poly_via_multiplication(
        &self,
        domain_size: usize,
        missing_idxs: &[usize],
    ) -> Result<(Vec<FsFr>, FsPoly), String> {
        let zero_eval: Vec<FsFr>;
        let mut zero_poly: FsPoly;

        if missing_idxs.is_empty() {
            zero_eval = Vec::new();
            zero_poly = FsPoly { coeffs: Vec::new() };
            return Ok((zero_eval, zero_poly));
        }

        if missing_idxs.len() >= domain_size {
            return Err(String::from("Missing idxs greater than domain size"));
        } else if domain_size > self.max_width {
            return Err(String::from(
                "Domain size greater than fft_settings.max_width",
            ));
        } else if !domain_size.is_power_of_two() {
            return Err(String::from("Domain size must be a power of 2"));
        }

        let degree_of_partial = 64; // Can be tuned & optimized (must be a power of 2)
        let missing_per_partial = degree_of_partial - 1; // Number of missing idxs needed per partial
        let domain_stride = self.max_width / domain_size;

        let mut partial_count = 1 + (missing_idxs.len() - 1) / missing_per_partial; // TODO: explain why -1 is used here

        let next_pow: usize = (partial_count * degree_of_partial).next_power_of_two();
        let domain_ceiling = min(next_pow, domain_size);
        // Calculate zero poly
        if missing_idxs.len() <= missing_per_partial {
            // When all idxs fit into a single multiplication
            zero_poly = self.do_zero_poly_mul_partial(missing_idxs, domain_stride)?;
        } else {
            // Otherwise, construct a set of partial polynomials
            // Save all constructed polynomials in a shared 'work' vector
            let mut work = Vec::with_capacity(next_pow);

            let mut partial_lens = vec![degree_of_partial; partial_count];

            #[cfg(not(feature = "parallel"))]
            {
                work.resize(work.capacity(), FsFr::zero());

                let mut missing_offset = 0;
                let mut work_offset = 0;
                let max = missing_idxs.len();

                // Insert all generated partial polynomials at degree_of_partial intervals in work vector
                for _i in 0..partial_count {
                    let end = min(missing_offset + missing_per_partial, max);

                    let mut partial = self.do_zero_poly_mul_partial(
                        &missing_idxs[missing_offset..end],
                        domain_stride,
                    )?;
                    partial.coeffs = pad_poly(partial.coeffs, degree_of_partial)?;
                    work.splice(
                        work_offset..(work_offset + degree_of_partial),
                        partial.coeffs.to_vec(),
                    );

                    missing_offset += missing_per_partial;
                    work_offset += degree_of_partial;
                }
            }

            #[cfg(feature = "parallel")]
            {
                let max = missing_idxs.len();

                // Insert all generated partial polynomials at degree_of_partial intervals in work vector
                let out_res = (0..partial_count)
                    .into_par_iter()
                    .map(|i| {
                        let missing_offset = missing_per_partial * i;
                        let end = min(missing_offset + missing_per_partial, max);

                        let partial = self.do_zero_poly_mul_partial(
                            &missing_idxs[missing_offset..end],
                            domain_stride,
                        )?;

                        pad_poly(partial.coeffs, degree_of_partial)
                    })
                    .collect::<Result<Vec<_>, String>>()?;

                out_res.into_iter().for_each(|mut item| {
                    work.append(&mut item);
                });

                work.resize(next_pow, FsFr::zero());
            }

            // Adjust last length to match its actual length
            partial_lens[partial_count - 1] =
                1 + missing_idxs.len() - (partial_count - 1) * missing_per_partial;

            // Reduce all vectors into one by reducing them w/ varying size multiplications
            let reduction_factor = 4; // Can be tuned & optimized (but must be a power of 2)
            while partial_count > 1 {
                let reduced_count = 1 + (partial_count - 1) / reduction_factor;
                let partial_size = partial_lens[0].next_power_of_two();

                // Step over polynomial space and produce larger multiplied polynomials
                for i in 0..reduced_count {
                    let start = i * reduction_factor;
                    let out_end = min((start + reduction_factor) * partial_size, domain_ceiling);
                    let reduced_len = min(out_end - start * partial_size, domain_size);
                    let partials_num = min(reduction_factor, partial_count - start);

                    // Calculate partial views from lens and offsets
                    // Also update offsets to match current iteration
                    let mut partial_vec = Vec::with_capacity(partials_num);
                    let partial_offset = start * partial_size;
                    for j in 0..partials_num {
                        let partial_offset = (start + j) * partial_size;
                        partial_vec.push(FsPoly {
                            coeffs: work[partial_offset..(partial_offset + partial_lens[i + j])]
                                .to_vec(),
                        });
                    }

                    if partials_num > 1 {
                        let mut reduced_poly = self.reduce_partials(reduced_len, &partial_vec)?;
                        // Update partial length to match its length after reduction
                        partial_lens[i] = reduced_poly.coeffs.len();
                        reduced_poly.coeffs =
                            pad_poly(reduced_poly.coeffs, partial_size * partials_num)?;
                        work.splice(
                            partial_offset..(partial_offset + reduced_poly.coeffs.len()),
                            reduced_poly.coeffs,
                        );
                    } else {
                        // Instead of keeping track of remaining polynomials, reuse i'th partial for start'th one
                        partial_lens[i] = partial_lens[start];
                    }
                }

                // Number of steps done equals the number of polynomials that we still need to reduce together
                partial_count = reduced_count;
            }

            zero_poly = FsPoly { coeffs: work };
        }

        // Pad resulting poly to expected
        match zero_poly.coeffs.len().cmp(&domain_size) {
            Ordering::Less => {
                zero_poly.coeffs = pad_poly(zero_poly.coeffs, domain_size)?;
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                zero_poly.coeffs = zero_poly.coeffs[..domain_size].to_vec();
            }
        }

        // Evaluate calculated poly
        zero_eval = self.fft_fr(&zero_poly.coeffs, false)?;

        Ok((zero_eval, zero_poly))
    }
}
