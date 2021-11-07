use crate::fftsettings::{ZkFFTSettings, new_fft_settings};
use crate::utils::is_power_of_two;
use crate::zkfr::blsScalar;
use kzg::{Fr, FFTFr};

pub fn fft_fr_slow(ret: &mut [blsScalar], data: &[blsScalar], stride: usize, roots: &[blsScalar], roots_stride: usize) {
    for i in 0..data.len() {
        ret[i] = data[0].mul(&roots[0]);

        for j in 1..data.len() {
            let value = data[j * stride].mul(&roots[((i * j) % data.len()) * roots_stride]);

            ret[i] = ret[i].add(&value);
        }
    }
}

pub fn fft_fr(data: &[blsScalar], inverse: bool, fft_settings: &ZkFFTSettings) -> Result<Vec<blsScalar>, String> {
    if data.len() > fft_settings.max_width {
        return Err(String::from("Given data is longer than allowed max width"));
    } else if !is_power_of_two(data.len()) {
        return Err(String::from("Given data is not power-of-two"));
    }

    let stride = fft_settings.max_width / data.len();
    let mut ret = vec![Fr::default(); data.len()];

    let roots = if inverse { &fft_settings.reverse_roots_of_unity } else { &fft_settings.expanded_roots_of_unity };
    fft_fr_fast(&mut ret, data, 1, roots, stride);

    if inverse {
        let mut inv_len: blsScalar = blsScalar::from(data.len() as u64);
        inv_len = inv_len.inverse();
        for i in 0..data.len() {
            ret[i] = ret[i].mul(&inv_len);
        }
    }

    return Ok(ret);
}

pub fn fft_fr_fast(ret: &mut [blsScalar], data: &[blsScalar], stride: usize, roots: &[blsScalar], roots_stride: usize) {
    let split: usize = ret.len() / 2;

    if split > 0 {
        fft_fr_fast(
            &mut ret[..split],
            data,
            stride * 2,
            roots,
            roots_stride * 2
        );
        fft_fr_fast(
            &mut ret[split..],
            &data[stride..],
            stride * 2,
            roots,
            roots_stride * 2
        );

        for i in 0..split {
            let y_times_root = ret[i + split].mul(&roots[i * roots_stride]);
            ret[i + split] = ret[i].sub(&y_times_root);
            ret[i] = ret[i].add(&y_times_root);
        }
    } else {
        ret[0] = data[0].clone();
    }
}
