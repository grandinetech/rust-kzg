use crate::kzg_proofs::FFTSettings;
use crate::kzg_types::ArkFr as BlstFr;
use kzg::{FFTFr, Fr as FFr};

impl FFTFr<BlstFr> for FFTSettings {
    fn fft_fr(&self, data: &[BlstFr], inverse: bool) -> Result<Vec<BlstFr>, String> {
        if data.len() > self.max_width {
            return Err(String::from("data length is longer than allowed max width"));
        }
        if !data.len().is_power_of_two() {
            return Err(String::from("data length is not power of 2"));
        }

        let stride = self.max_width / data.len();
        let mut ret = vec![BlstFr::default(); data.len()];

        let roots = if inverse {
            &self.reverse_roots_of_unity
        } else {
            &self.expanded_roots_of_unity
        };

        fft_fr_fast(&mut ret, data, 1, roots, stride);

        if inverse {
            let inv_fr_len = BlstFr::from_u64(data.len() as u64).inverse();
            ret[..data.len()]
                .iter_mut()
                .for_each(|f| *f = BlstFr::mul(f, &inv_fr_len));
        }

        Ok(ret)
    }
}

pub fn fft_fr_fast(
    ret: &mut [BlstFr],
    data: &[BlstFr],
    stride: usize,
    roots: &[BlstFr],
    roots_stride: usize,
) {
    let half: usize = ret.len() / 2;
    if half > 0 {
        #[cfg(not(feature = "parallel"))]
        {
            fft_fr_fast(&mut ret[..half], data, stride * 2, roots, roots_stride * 2);
            fft_fr_fast(
                &mut ret[half..],
                &data[stride..],
                stride * 2,
                roots,
                roots_stride * 2,
            );
        }

        #[cfg(feature = "parallel")]
        {
            if half > 256 {
                let (lo, hi) = ret.split_at_mut(half);
                rayon::join(
                    || fft_fr_fast(lo, data, stride * 2, roots, roots_stride * 2),
                    || fft_fr_fast(hi, &data[stride..], stride * 2, roots, roots_stride * 2),
                );
            } else {
                fft_fr_fast(&mut ret[..half], data, stride * 2, roots, roots_stride * 2);
                fft_fr_fast(
                    &mut ret[half..],
                    &data[stride..],
                    stride * 2,
                    roots,
                    roots_stride * 2,
                );
            }
        }

        for i in 0..half {
            let y_times_root = ret[i + half].mul(&roots[i * roots_stride]);
            ret[i + half] = ret[i].sub(&y_times_root);
            ret[i] = ret[i].add(&y_times_root);
        }
    } else {
        ret[0] = data[0];
    }
}

pub fn fft_fr_slow(
    ret: &mut [BlstFr],
    data: &[BlstFr],
    stride: usize,
    roots: &[BlstFr],
    roots_stride: usize,
) {
    let mut v;
    let mut jv;
    let mut r;

    for i in 0..data.len() {
        ret[i] = data[0].mul(&roots[0]);
        for j in 1..data.len() {
            jv = data[j * stride];
            r = roots[((i * j) % data.len()) * roots_stride];
            v = jv.mul(&r);
            ret[i] = ret[i].add(&v);
        }
    }
}
