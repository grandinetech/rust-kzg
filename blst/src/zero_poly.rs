extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::{min, Ordering};

use kzg::{FFTFr, Fr, ZeroPoly, common_utils::next_pow_of_2};

use crate::types::fft_settings::FsFFTSettings;
use crate::types::fr::FsFr;
use crate::types::poly::FsPoly;

#[cfg(feature = "parallel")]
use rayon::prelude::*;
use smallvec::{smallvec, SmallVec};

// Can be tuned & optimized (must be a power of 2)
const DEGREE_OF_PARTIAL: usize = 256;
// Can be tuned & optimized (but must be a power of 2)
const REDUCTION_FACTOR: usize = 4;

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

/// Pad given poly coefficients it with zeros to new length
pub fn pad_poly_coeffs<const N: usize, T>(
    mut coeffs: SmallVec<[T; N]>,
    new_length: usize,
) -> Result<SmallVec<[T; N]>, String>
where
    T: Default + Clone,
{
    if new_length < coeffs.len() {
        return Err(String::from(
            "new_length must be longer or equal to coeffs length",
        ));
    }

    coeffs.resize(new_length, T::default());

    Ok(coeffs)
}

impl FsFFTSettings {
    fn do_zero_poly_mul_partial(
        &self,
        idxs: &[usize],
        stride: usize,
    ) -> Result<SmallVec<[FsFr; DEGREE_OF_PARTIAL]>, String> {
        if idxs.is_empty() {
            return Err(String::from("idx array must not be empty"));
        }

        // Makes use of long multiplication in terms of (x - w_0)(x - w_1)..
        let mut coeffs = SmallVec::<[FsFr; DEGREE_OF_PARTIAL]>::new();

        // For the first member, store -w_0 as constant term
        coeffs.push(self.expanded_roots_of_unity[idxs[0] * stride].negate());

        for (i, idx) in idxs.iter().copied().enumerate().skip(1) {
            // For member (x - w_i) take coefficient as -(w_i + w_{i-1} + ...)
            let neg_di = self.expanded_roots_of_unity[idx * stride].negate();
            coeffs.push(neg_di.add(&coeffs[i - 1]));

            // Multiply all previous members by (x - w_i)
            // It equals multiplying by - w_i and adding x^(i - 1) coefficient (implied multiplication by x)
            for j in (1..i).rev() {
                coeffs[j] = coeffs[j].mul(&neg_di).add(&coeffs[j - 1]);
            }

            // Multiply x^0 member by - w_i
            coeffs[0] = coeffs[0].mul(&neg_di);
        }

        coeffs.resize(idxs.len() + 1, FsFr::one());

        Ok(coeffs)
    }

    fn reduce_partials(
        &self,
        domain_size: usize,
        partial_coeffs: SmallVec<[SmallVec<[FsFr; DEGREE_OF_PARTIAL]>; REDUCTION_FACTOR]>,
    ) -> Result<SmallVec<[FsFr; DEGREE_OF_PARTIAL]>, String> {
        if !domain_size.is_power_of_two() {
            return Err(String::from("Expected domain size to be a power of 2"));
        }

        if partial_coeffs.is_empty() {
            return Err(String::from("partials must not be empty"));
        }

        // Calculate the resulting polynomial degree
        // E.g. (a * x^n + ...) (b * x^m + ...) has a degree of x^(n+m)
        let out_degree = partial_coeffs
            .iter()
            .map(|partial| {
                // TODO: Not guaranteed by function signature that this doesn't underflow
                partial.len() - 1
            })
            .sum::<usize>();

        if out_degree + 1 > domain_size {
            return Err(String::from(
                "Out degree is longer than possible polynomial size in domain",
            ));
        }

        let mut partial_coeffs = partial_coeffs.into_iter();

        // Pad all partial polynomials to same length, compute their FFT and multiply them together
        let mut padded_partial = pad_poly_coeffs(
            partial_coeffs
                .next()
                .expect("Not empty, checked above; qed"),
            domain_size,
        )?;
        let mut eval_result: SmallVec<[FsFr; DEGREE_OF_PARTIAL]> =
            smallvec![FsFr::zero(); domain_size];
        self.fft_fr_output(&padded_partial, false, &mut eval_result)?;

        for partial in partial_coeffs {
            padded_partial = pad_poly_coeffs(partial, domain_size)?;
            let mut evaluated_partial: SmallVec<[FsFr; DEGREE_OF_PARTIAL]> =
                smallvec![FsFr::zero(); domain_size];
            self.fft_fr_output(&padded_partial, false, &mut evaluated_partial)?;

            eval_result
                .iter_mut()
                .zip(evaluated_partial.iter())
                .for_each(|(eval_result, evaluated_partial)| {
                    *eval_result = eval_result.mul(evaluated_partial);
                });
        }

        let mut coeffs = smallvec![FsFr::zero(); domain_size];
        // Apply an inverse FFT to produce a new poly. Limit its size to out_degree + 1
        self.fft_fr_output(&eval_result, true, &mut coeffs)?;
        coeffs.truncate(out_degree + 1);

        Ok(coeffs)
    }
}

impl ZeroPoly<FsFr, FsPoly> for FsFFTSettings {
    fn do_zero_poly_mul_partial(&self, idxs: &[usize], stride: usize) -> Result<FsPoly, String> {
        self.do_zero_poly_mul_partial(idxs, stride)
            .map(|coeffs| FsPoly {
                coeffs: coeffs.into_vec(),
            })
    }

    fn reduce_partials(&self, domain_size: usize, partials: &[FsPoly]) -> Result<FsPoly, String> {
        self.reduce_partials(
            domain_size,
            partials
                .iter()
                .map(|partial| SmallVec::from_slice(&partial.coeffs))
                .collect(),
        )
        .map(|coeffs| FsPoly {
            coeffs: coeffs.into_vec(),
        })
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

        let missing_per_partial = DEGREE_OF_PARTIAL - 1; // Number of missing idxs needed per partial
        let domain_stride = self.max_width / domain_size;

        let mut partial_count = 1 + (missing_idxs.len() - 1) / missing_per_partial; // TODO: explain why -1 is used here

        let next_pow: usize = next_pow_of_2(partial_count * DEGREE_OF_PARTIAL);
        let domain_ceiling = min(next_pow, domain_size);
        // Calculate zero poly
        if missing_idxs.len() <= missing_per_partial {
            // When all idxs fit into a single multiplication
            zero_poly = FsPoly {
                coeffs: self
                    .do_zero_poly_mul_partial(missing_idxs, domain_stride)?
                    .into_vec(),
            };
        } else {
            // Otherwise, construct a set of partial polynomials
            // Save all constructed polynomials in a shared 'work' vector
            let mut work = vec![FsFr::zero(); next_pow];

            let mut partial_lens = vec![DEGREE_OF_PARTIAL; partial_count];

            #[cfg(not(feature = "parallel"))]
            let iter = missing_idxs
                .chunks(missing_per_partial)
                .zip(work.chunks_exact_mut(DEGREE_OF_PARTIAL));
            #[cfg(feature = "parallel")]
            let iter = missing_idxs
                .par_chunks(missing_per_partial)
                .zip(work.par_chunks_exact_mut(DEGREE_OF_PARTIAL));
            // Insert all generated partial polynomials at degree_of_partial intervals in work vector
            iter.for_each(|(missing_idxs, work)| {
                let partial_coeffs = self
                    .do_zero_poly_mul_partial(missing_idxs, domain_stride)
                    .expect("`missing_idxs` is guaranteed to not be empty; qed");

                let partial_coeffs = pad_poly_coeffs(partial_coeffs, DEGREE_OF_PARTIAL).expect(
                    "`partial.coeffs.len()` (same as `missing_idxs.len() + 1`) is \
                    guaranteed to be at most `degree_of_partial`; qed",
                );
                work[..DEGREE_OF_PARTIAL].copy_from_slice(&partial_coeffs);
            });

            // Adjust last length to match its actual length
            partial_lens[partial_count - 1] =
                1 + missing_idxs.len() - (partial_count - 1) * missing_per_partial;

            // Reduce all vectors into one by reducing them w/ varying size multiplications
            while partial_count > 1 {
                let reduced_count = 1 + (partial_count - 1) / REDUCTION_FACTOR;
                let partial_size = next_pow_of_2(partial_lens[0]);

                // Step over polynomial space and produce larger multiplied polynomials
                for i in 0..reduced_count {
                    let start = i * REDUCTION_FACTOR;
                    let out_end = min((start + REDUCTION_FACTOR) * partial_size, domain_ceiling);
                    let reduced_len = min(out_end - start * partial_size, domain_size);
                    let partials_num = min(REDUCTION_FACTOR, partial_count - start);

                    // Calculate partial views from lens and offsets
                    // Also update offsets to match current iteration
                    let partial_offset = start * partial_size;
                    let mut partial_coeffs = SmallVec::new();
                    for (partial_offset, partial_len) in (partial_offset..)
                        .step_by(partial_size)
                        .zip(partial_lens.iter().skip(i).copied())
                        .take(partials_num)
                    {
                        // We know the capacity required in `reduce_partials()` call below to avoid
                        // re-allocation
                        let mut coeffs = SmallVec::with_capacity(reduced_len);
                        coeffs.extend_from_slice(&work[partial_offset..][..partial_len]);
                        partial_coeffs.push(coeffs);
                    }

                    if partials_num > 1 {
                        let mut reduced_coeffs =
                            self.reduce_partials(reduced_len, partial_coeffs)?;
                        // Update partial length to match its length after reduction
                        partial_lens[i] = reduced_coeffs.len();
                        reduced_coeffs =
                            pad_poly_coeffs(reduced_coeffs, partial_size * partials_num)?;
                        work[partial_offset..][..reduced_coeffs.len()]
                            .copy_from_slice(&reduced_coeffs);
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
                zero_poly.coeffs.truncate(domain_size);
            }
        }

        // Evaluate calculated poly
        zero_eval = self.fft_fr(&zero_poly.coeffs, false)?;

        Ok((zero_eval, zero_poly))
    }
}
