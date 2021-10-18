use std::cmp::min;
use crate::fft_fr::fft_fr;
use crate::kzg_types::{FFTSettings, Poly};
use crate::utils::{is_power_of_two, next_power_of_two};
use kzg::IFr;
use crate::kzg_types::Fr;

/// Calculates a polynomial that evaluates to zero for roots of unity at given indices.
/// The returned polynomial has a length of idxs.len() + 1.
pub fn do_zero_poly_mul_partial(idxs: &[usize], stride: usize, fft_settings: &FFTSettings) -> Result<Poly, String> {
    if idxs.len() == 0 {
        return Err(String::from("idx array must be non-zero"));
    }

    // Makes use of long multiplication in terms of (x - w_0)(x - w_1)..
    // Initialize poly with 1s
    let mut poly = Poly { coeffs: vec![Fr::one(); idxs.len() + 1] };
    // For the first member, store -w_0 as constant term
    poly.coeffs[0] = fft_settings.expanded_roots_of_unity[idxs[0] * stride].negate();

    for i in 1..idxs.len() {
        // For member (x - w_i) take coefficient as -(w_i + w_{i-1} + ...)
        poly.coeffs[i] = fft_settings.expanded_roots_of_unity[idxs[i] * stride].negate();
        let neg_di = poly.coeffs[i].clone();
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

/// Create a copy of the given poly and pad it with zeros
pub fn pad_poly(poly: &Poly, new_length: usize) -> Result<Vec<Fr>, String> {
    if new_length < poly.coeffs.len() {
        return Err(String::from("new_length must be longer or equal to poly length"));
    }

    let mut ret = poly.coeffs.to_vec();

    for _i in poly.coeffs.len()..new_length {
        ret.push(Fr::zero())
    }

    Ok(ret)
}

/// Reduce partials using a specified domain size.
/// Calculates the product of all polynomials via FFT and then applies an inverse FFT to produce a new Polynomial.
pub fn reduce_partials(domain_size: usize, partials: &[Poly], fft_settings: &FFTSettings) -> Result<Poly, String> {
    if !is_power_of_two(domain_size) {
        return Err(String::from("Expected domain size to be a power of 2"));
    }

    // Calculate the resulting polynomial degree
    // E.g. (a * x^n + ...) (b * x^m + ...) has a degree of x^(n+m)
    let mut out_degree: usize = 0;
    for i in 0..partials.len() {
        out_degree += partials[i].coeffs.len() - 1;
    }

    if out_degree + 1 > domain_size {
        return Err(String::from("Out degree is longer than possible polynomial size in domain"));
    }

    // Pad all partial polynomials to same length, compute their FFT and multiply them together
    let mut padded_partial = pad_poly(&partials[0], domain_size)?;
    let mut eval_result = fft_fr(&padded_partial, false, fft_settings)?;

    for i in 1..(partials.len()) {
        padded_partial = pad_poly(&partials[i], domain_size)?;
        let evaluated_partial = fft_fr(&padded_partial, false, fft_settings)?;
        for j in 0..domain_size {
            eval_result[j] = eval_result[j].mul(&evaluated_partial[j]);
        }
    }

    // Apply an inverse FFT to produce a new poly. Limit its size to out_degree + 1
    let coeffs = fft_fr(&eval_result, true, fft_settings)?;
    let ret = Poly { coeffs: coeffs[..(out_degree + 1)].to_vec() };

    Ok(ret)
}

// TODO: explain how algo works
pub fn zero_poly_via_multiplication(domain_size: usize, missing_idxs: &[usize], fft_settings: &FFTSettings) -> Result<(Vec<Fr>, Poly), String> {
    let zero_eval: Vec<Fr>;
    let mut zero_poly: Poly;

    if missing_idxs.len() == 0 {
        zero_eval = Vec::new();
        zero_poly = Poly { coeffs: Vec::new() };
        return Ok((zero_eval, zero_poly));
    }

    if missing_idxs.len() >= domain_size {
        return Err(String::from("Missing idxs greater than domain size"));
    } else if domain_size > fft_settings.max_width {
        return Err(String::from("Domain size greater than fft_settings.max_width"));
    } else if !is_power_of_two(domain_size) {
        return Err(String::from("Domain size must be a power of 2"));
    }

    let degree_of_partial = 64; // Can be tuned & optimized (must be a power of 2)
    let missing_per_partial = degree_of_partial - 1; // Number of missing idxs needed per partial
    let domain_stride = fft_settings.max_width / domain_size;
    let mut partial_count = 1 + (missing_idxs.len() - 1) / missing_per_partial; // TODO: explain why -1 is used here
    let domain_ceiling = min(next_power_of_two(partial_count * degree_of_partial), domain_size);

    // Calculate zero poly
    if missing_idxs.len() <= missing_per_partial {
        // When all idxs fit into a single multiplication
        zero_poly = do_zero_poly_mul_partial(&missing_idxs, domain_stride, &fft_settings)?;
    } else {
        // Otherwise, construct a set of partial polynomials
        // Save all constructed polynomials in a shared 'work' vector
        let mut work = vec![Fr::zero(); next_power_of_two(partial_count * degree_of_partial)];

        let mut partial_lens = Vec::new();
        let mut partial_offsets = Vec::new();

        let mut missing_offset = 0;
        let mut work_offset = 0;
        let max = missing_idxs.len();

        // Insert all generated partial polynomials at degree_of_partial intervals in work vector
        for _i in 0..partial_count {
            let end = min(missing_offset + missing_per_partial, max);

            let mut partial = do_zero_poly_mul_partial(&missing_idxs[missing_offset..end], domain_stride, fft_settings)?;
            partial.coeffs = pad_poly(&partial, degree_of_partial)?;
            work.splice(work_offset..(work_offset + degree_of_partial), partial.coeffs.to_vec());
            partial_lens.push(degree_of_partial);
            partial_offsets.push(work_offset);

            missing_offset += missing_per_partial;
            work_offset += degree_of_partial;
        }

        // Adjust last length to match its actual length
        partial_lens[partial_count - 1] = 1 + missing_idxs.len() - (partial_count - 1) * missing_per_partial;

        // Reduce all vectors into one by reducing them w/ varying size multiplications
        let reduction_factor = 4; // Can be tuned & optimized (but must be a power of 2)
        while partial_count > 1 {
            let reduced_count = 1 + (partial_count - 1) / reduction_factor;
            let partial_size = next_power_of_two(partial_lens[0]);

            // Step over polynomial space and produce larger multiplied polynomials
            for i in 0..reduced_count {
                let start = i * reduction_factor;
                let out_end = min((start + reduction_factor) * partial_size, domain_ceiling);
                let reduced_len = min(out_end - start * partial_size, domain_size);
                let partials_num = min(reduction_factor, partial_count - start);

                // Calculate partial views from lens and offsets
                // Also update offsets to match current iteration
                let mut partial_vec = Vec::new();
                partial_offsets[i] = start * partial_size;
                for j in 0..(partials_num) {
                    partial_offsets[i + j] = (start + j) * partial_size;
                    partial_vec.push(Poly { coeffs: work[partial_offsets[i + j]..(partial_offsets[i + j] + partial_lens[i + j])].to_vec() });
                }

                if partials_num > 1 {
                    let mut reduced_poly = reduce_partials(reduced_len, &partial_vec, fft_settings)?;
                    // Update partial length to match its length after reduction
                    partial_lens[i] = reduced_poly.coeffs.len();
                    reduced_poly.coeffs = pad_poly(&reduced_poly, partial_size * partials_num)?;
                    work.splice((partial_offsets[i])..(partial_offsets[i] + reduced_poly.coeffs.len()),
                                reduced_poly.coeffs);
                } else {
                    // Instead of keeping track of remaining polynomials, reuse i'th partial for start'th one
                    partial_lens[i] = partial_lens[start];
                }
            }

            // Number of steps done equals the number of polynomials that we still need to reduce together
            partial_count = reduced_count;
        }

        zero_poly = Poly { coeffs: work };
    }

    // Pad resulting poly to expected length
    if zero_poly.coeffs.len() < domain_size {
        zero_poly.coeffs = pad_poly(&zero_poly, domain_size)?;
    } else if zero_poly.coeffs.len() > domain_size {
        zero_poly.coeffs = zero_poly.coeffs[..domain_size].to_vec();
    }

    // Evaluate calculated poly
    zero_eval = fft_fr(&zero_poly.coeffs, false, fft_settings)?;

    Ok((zero_eval, zero_poly))
}