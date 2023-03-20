use crate::fftsettings::ZkFFTSettings;
use crate::kzg_types::ZkG1Projective;
use crate::utils::is_power_of_two;
use crate::zkfr::blsScalar;
use kzg::{Fr, FFTG1, G1};

pub fn fft_g1_slow(
    ret: &mut [ZkG1Projective],
    data: &[ZkG1Projective],
    stride: usize,
    roots: &[blsScalar],
    roots_stride: usize,
) {
    for i in 0..data.len() {
        ret[i] = data[0].mul(&roots[0]);

        for j in 1..data.len() {
            let v = data[j * stride].mul(&roots[((i * j) % data.len()) * roots_stride]);
            ret[i] = ret[i].add_or_dbl(&v);
        }
    }
}

impl FFTG1<ZkG1Projective> for ZkFFTSettings {
    fn fft_g1(
        &self,
        data: &[ZkG1Projective],
        inverse: bool,
    ) -> Result<Vec<ZkG1Projective>, String> {
        if data.len() > self.max_width {
            return Err(String::from("Given data is longer than allowed max width"));
        } else if !is_power_of_two(data.len()) {
            return Err(String::from("Given data is not power-of-two length"));
        }

        let stride = self.max_width / data.len();
        let mut ret = vec![ZkG1Projective::default(); data.len()];

        let roots = if inverse {
            &self.reverse_roots_of_unity
        } else {
            &self.expanded_roots_of_unity
        };

        fft_g1_fast(&mut ret, data, 1, roots, stride);

        if inverse {
            let mut inv_len: blsScalar = blsScalar::from_u64(data.len() as u64);
            inv_len = inv_len.inverse();
            for i in ret.iter_mut().take(data.len())
            /*0..data.len()*/
            {
                *i = i.mul(&inv_len);
            }
        }

        Ok(ret)
    }
}

pub fn fft_g1_fast(
    ret: &mut [ZkG1Projective],
    data: &[ZkG1Projective],
    stride: usize,
    roots: &[blsScalar],
    roots_stride: usize,
) {
    let split = ret.len() / 2;
    if split > 0 {
        #[cfg(feature = "parallel")]
        {
            let (lo, hi) = ret.split_at_mut(split);
            rayon::join(
                || fft_g1_fast(lo, data, stride * 2, roots, roots_stride * 2),
                || fft_g1_fast(hi, &data[stride..], stride * 2, roots, roots_stride * 2),
            );
        }

        #[cfg(not(feature = "parallel"))]
        {
            fft_g1_fast(&mut ret[..split], data, stride * 2, roots, roots_stride * 2);
            fft_g1_fast(
                &mut ret[split..],
                &data[stride..],
                stride * 2,
                roots,
                roots_stride * 2,
            );
        }
        for i in 0..split {
            let y_times_root = ret[i + split].mul(&roots[i * roots_stride]);
            ret[i + split] = ret[i].sub(&y_times_root);
            ret[i] = ret[i].add_or_dbl(&y_times_root);
        }
    } else {
        ret[0] = data[0];
    }
}
