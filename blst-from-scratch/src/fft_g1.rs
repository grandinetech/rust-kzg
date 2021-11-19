use crate::kzg_types::{FsFFTSettings, FsFr, FsG1};
use crate::utils::is_power_of_two;
use kzg::{Fr, FFTG1, G1, G1Mul};

pub fn fft_g1_fast(
    ret: &mut [FsG1],
    data: &[FsG1],
    stride: usize,
    roots: &[FsFr],
    roots_stride: usize,
) {
    let half = ret.len() / 2;
    if half > 0 {
        fft_g1_fast(&mut ret[..half], data, stride * 2, roots, roots_stride * 2);
        fft_g1_fast(
            &mut ret[half..],
            &data[stride..],
            stride * 2,
            roots,
            roots_stride * 2,
        );
        for i in 0..half {
            let y_times_root = ret[i + half].mul(&roots[i * roots_stride]);
            ret[i + half] = ret[i].sub(&y_times_root);
            ret[i] = ret[i].add_or_dbl(&y_times_root);
        }
    } else {
        ret[0] = data[0].clone();
    }
}

// Used for testing
pub fn fft_g1_slow(
    ret: &mut [FsG1],
    data: &[FsG1],
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
            ret[i] = ret[i].add_or_dbl(&v);
        }
    }
}

impl FFTG1<FsG1> for FsFFTSettings {
    fn fft_g1(&self, data: &[FsG1], inverse: bool) -> Result<Vec<FsG1>, String> {
        if data.len() > self.max_width {
            return Err(String::from(
                "Supplied list is longer than the available max width",
            ));
        } else if !is_power_of_two(data.len()) {
            return Err(String::from("A list with power-of-two length expected"));
        }

        let stride = self.max_width / data.len();
        let mut ret = vec![FsG1::default(); data.len()];

        let roots = if inverse {
            &self.reverse_roots_of_unity
        } else {
            &self.expanded_roots_of_unity
        };
        fft_g1_fast(&mut ret, data, 1, roots, stride);
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
