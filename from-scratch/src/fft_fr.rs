use crate::kzg_types::FFTSettings;
use crate::utils::is_power_of_two;
use blst::{blst_fr_add, blst_fr_from_uint64, blst_fr_inverse, blst_fr_mul, blst_fr_sub};
use kzg::Fr;

/// Fast Fourier Transform for finite field elements. Polynomial ret is operated on in reverse order: ret_i * x ^ (len - i - 1)
pub fn fft_fr_fast(ret: &mut [Fr], data: &[Fr], stride: usize, roots: &[Fr], roots_stride: usize) {
    let half: usize = ret.len() / 2;
    if half > 0 {
        // Recurse
        // Offsetting data by stride = 1 on the first iteration forces the even members to the first half
        // and the odd members to the second half
        fft_fr_fast(&mut ret[..half], data, stride * 2, roots, roots_stride * 2);
        fft_fr_fast(&mut ret[half..], &data[stride..], stride * 2, roots, roots_stride * 2);
        for i in 0..half {
            let mut y_times_root: Fr = Fr::default();
            unsafe {
                blst_fr_mul(&mut y_times_root, &ret[i + half], &roots[i * roots_stride]);
                blst_fr_sub(&mut ret[i + half], &ret[i], &y_times_root);
                blst_fr_add(&mut ret[i], &ret[i], &y_times_root);
            }
        }
    } else {
        // When len = 1, return the permuted element
        ret[0] = data[0].clone();
    }
}

/// Fast Fourier Transform for finite field elements
pub fn fft_fr(data: &[Fr], inverse: bool, fft_settings: &FFTSettings) -> Result<Vec<Fr>, String> {
    if data.len() > fft_settings.max_width {
        return Err(String::from("Supplied list is longer than the available max width"));
    } else if !is_power_of_two(data.len()) {
        return Err(String::from("A list with power-of-two length expected"));
    }

    // In case more roots are provided with fft_settings, use a larger stride
    let stride = fft_settings.max_width / data.len();
    let mut ret = vec![Fr::default(); data.len()];

    // Inverse is same as regular, but all constants are reversed and results are divided by n
    let roots = if inverse { &fft_settings.reverse_roots_of_unity } else { &fft_settings.expanded_roots_of_unity };
    fft_fr_fast(&mut ret, data, 1, roots, stride);

    if inverse {
        unsafe {
            let mut inv_len: Fr = Fr::default();
            blst_fr_from_uint64(&mut inv_len, [data.len() as u64, 0, 0, 0].as_ptr());
            blst_fr_inverse(&mut inv_len, &inv_len);
            for i in 0..data.len() {
                blst_fr_mul(&mut ret[i], &ret[i], &inv_len);
            }
        }
    }

    return Ok(ret);
}

/// Simplified Discrete Fourier Transform, mainly used for testing
pub fn fft_fr_slow(ret: &mut [Fr], data: &[Fr], stride: usize, roots: &[Fr], roots_stride: usize) {
    for i in 0..data.len() {
        // Evaluate first member at 1
        unsafe {
            blst_fr_mul(&mut ret[i], &data[0], &roots[0]);
        }

        // Evaluate the rest of members using a step of (i * J) % data.len() over the roots
        // This distributes the roots over correct x^n members and saves on multiplication
        for j in 1..data.len() {
            let mut v: Fr = Fr::default();
            unsafe {
                blst_fr_mul(&mut v, &data[j * stride], &roots[((i * j) % data.len()) * roots_stride]);
                blst_fr_add(&mut ret[i], &ret[i], &v);
            }
        }
    }
}