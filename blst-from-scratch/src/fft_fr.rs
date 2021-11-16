use crate::kzg_types::FsFFTSettings;
use crate::kzg_types::FsFr;
use crate::utils::is_power_of_two;
use kzg::{FFTFr, Fr};

/// Fast Fourier Transform for finite field elements. Polynomial ret is operated on in reverse order: ret_i * x ^ (len - i - 1)
pub fn fft_fr_fast(
    ret: &mut [FsFr],
    data: &[FsFr],
    stride: usize,
    roots: &[FsFr],
    roots_stride: usize,
) {
    let half: usize = ret.len() / 2;
    if half > 0 {
        // Recurse
        // Offsetting data by stride = 1 on the first iteration forces the even members to the first half
        // and the odd members to the second half
        fft_fr_fast(&mut ret[..half], data, stride * 2, roots, roots_stride * 2);
        fft_fr_fast(
            &mut ret[half..],
            &data[stride..],
            stride * 2,
            roots,
            roots_stride * 2,
        );
        for i in 0..half {
            let y_times_root = ret[i + half].mul(&roots[i * roots_stride]);
            ret[i + half] = ret[i].sub(&y_times_root);
            ret[i] = ret[i].add(&y_times_root);
        }
    } else {
        // When len = 1, return the permuted element
        ret[0] = data[0].clone();
    }
}

impl FFTFr<FsFr> for FsFFTSettings {
    /// Fast Fourier Transform for finite field elements
    fn fft_fr(&self, data: &[FsFr], inverse: bool) -> Result<Vec<FsFr>, String> {
        if data.len() > self.max_width {
            return Err(String::from(
                "Supplied list is longer than the available max width",
            ));
        } else if !is_power_of_two(data.len()) {
            return Err(String::from("A list with power-of-two length expected"));
        }

        // In case more roots are provided with fft_settings, use a larger stride
        let stride = self.max_width / data.len();
        let mut ret = vec![FsFr::default(); data.len()];

        // Inverse is same as regular, but all constants are reversed and results are divided by n
        // This is a property of the DFT matrix
        let roots = if inverse {
            &self.reverse_roots_of_unity
        } else {
            &self.expanded_roots_of_unity
        };
        fft_fr_fast(&mut ret, data, 1, roots, stride);

        if inverse {
            let mut inv_len: FsFr = FsFr::from_u64(data.len() as u64);
            inv_len = inv_len.inverse();
            for i in 0..data.len() {
                ret[i] = ret[i].mul(&inv_len);
            }
        }

        return Ok(ret);
    }
}

/// Simplified Discrete Fourier Transform, mainly used for testing
pub fn fft_fr_slow(
    ret: &mut [FsFr],
    data: &[FsFr],
    stride: usize,
    roots: &[FsFr],
    roots_stride: usize,
) {
    for i in 0..data.len() {
        // Evaluate first member at 1
        ret[i] = data[0].mul(&roots[0]);

        // Evaluate the rest of members using a step of (i * J) % data.len() over the roots
        // This distributes the roots over correct x^n members and saves on multiplication
        for j in 1..data.len() {
            let v = data[j * stride].mul(&roots[((i * j) % data.len()) * roots_stride]);
            ret[i] = ret[i].add(&v);
        }
    }
}
