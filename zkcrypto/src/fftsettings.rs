use crate::consts::*;
use crate::fft_fr::*;
use crate::poly::*;
use crate::zkfr::blsScalar;
use std::cmp::Ordering;

use kzg::common_utils::is_power_of_two;
use kzg::common_utils::reverse_bit_order;
use kzg::{FFTFr, FFTSettings, FFTSettingsPoly, Fr};

#[derive(Debug, Clone, Default)]
pub struct ZkFFTSettings {
    pub max_width: usize,
    pub root_of_unity: blsScalar,
    pub expanded_roots_of_unity: Vec<blsScalar>,
    pub reverse_roots_of_unity: Vec<blsScalar>,
    pub roots_of_unity: Vec<blsScalar>,
}

impl ZkFFTSettings {
    pub fn das_fft_extension_stride(&self, vals: &mut [blsScalar], stride: usize) {
        match vals.len().cmp(&2) {
            Ordering::Less => {}
            Ordering::Equal => {
                let x = vals[0].add(&vals[1]);
                let y = vals[0].sub(&vals[1]);
                let tmp = y.mul(&self.expanded_roots_of_unity[stride]);

                vals[0] = x.add(&tmp);
                vals[1] = x.sub(&tmp);
            }
            _ => {
                let half = vals.len();
                let half_halved = half / 2;

                for i in 0..half_halved {
                    let tmp1 = vals[i].add(&vals[half_halved + i]);
                    let tmp2 = vals[i].sub(&vals[half_halved + i]);
                    vals[half_halved + i] = tmp2.mul(&self.reverse_roots_of_unity[i * 2 * stride]);
                    vals[i] = tmp1;
                }

                #[cfg(feature = "parallel")]
                {
                    if vals.len() > 32 {
                        let (lo, hi) = vals.split_at_mut(half_halved);
                        rayon::join(
                            || self.das_fft_extension_stride(hi, stride * 2),
                            || self.das_fft_extension_stride(lo, stride * 2),
                        );
                    } else {
                        self.das_fft_extension_stride(&mut vals[..half_halved], stride * 2);
                        self.das_fft_extension_stride(&mut vals[half_halved..], stride * 2);
                    }
                }

                #[cfg(not(feature = "parallel"))]
                {
                    self.das_fft_extension_stride(&mut vals[..half_halved], stride * 2);
                    self.das_fft_extension_stride(&mut vals[half_halved..], stride * 2);
                }

                for i in 0..half_halved {
                    let x = vals[i];
                    let y = vals[half_halved + i];
                    let y_times_root = y.mul(&self.expanded_roots_of_unity[(1 + 2 * i) * stride]);
                    vals[i] = x.add(&y_times_root);
                    vals[half_halved + i] = x.sub(&y_times_root);
                }
            }
        }
    }
}

impl FFTSettingsPoly<blsScalar, ZPoly, ZkFFTSettings> for ZkFFTSettings {
    fn poly_mul_fft(
        a: &ZPoly,
        b: &ZPoly,
        len: usize,
        _fs: Option<&ZkFFTSettings>,
    ) -> Result<ZPoly, String> {
        poly_mul_fft(len, a, b)
    }
}

impl FFTFr<blsScalar> for ZkFFTSettings {
    fn fft_fr(&self, data: &[blsScalar], inverse: bool) -> Result<Vec<blsScalar>, String> {
        if data.len() > self.max_width {
            return Err(String::from(
                "The supplied list is longer than the available max width",
            ));
        } else if !is_power_of_two(data.len()) {
            return Err(String::from("A list with power-of-two length is expected"));
        }

        // In case more roots are provided with fft_settings, use a larger stride
        let stride = self.max_width / data.len();
        let mut ret = vec![blsScalar::default(); data.len()];

        // Inverse is same as regular, but all constants are reversed and results are divided by n
        // This is a property of the DFT matrix
        let roots = if inverse {
            &self.reverse_roots_of_unity
        } else {
            &self.expanded_roots_of_unity
        };
        fft_fr_fast(&mut ret, data, 1, roots, stride);

        if inverse {
            let mut inv_len: blsScalar = blsScalar::from_u64(data.len() as u64);
            inv_len = inv_len.inverse();
            for i in ret.iter_mut().take(data.len()) {
                *i = i.mul(&inv_len);
            }
        }

        Ok(ret)
    }
}

impl FFTSettings<blsScalar> for ZkFFTSettings {
    fn new(scale: usize) -> Result<Self, String> {
        if scale >= SCALE2_ROOT_OF_UNITY.len() {
            return Err(String::from(
                "Scale is expected to be within root of unity matrix row size",
            ));
        }

        // max_width = 2 ^ max_scale
        let max_width: usize = 1 << scale;
        let root_of_unity = blsScalar::from_u64_arr(&SCALE2_ROOT_OF_UNITY[scale]);

        // create max_width of roots & store them reversed as well
        let expanded_roots_of_unity = expand_root_of_unity(&root_of_unity, max_width).unwrap();
        let mut reverse_roots_of_unity = expanded_roots_of_unity.clone();
        reverse_roots_of_unity.reverse();

        // Permute the roots of unity
        let mut roots_of_unity = expanded_roots_of_unity.clone();
        reverse_bit_order(&mut roots_of_unity)?;

        Ok(Self {
            max_width,
            root_of_unity,
            expanded_roots_of_unity,
            reverse_roots_of_unity,
            roots_of_unity,
        })
    }

    fn get_max_width(&self) -> usize {
        self.max_width
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> blsScalar {
        self.expanded_roots_of_unity[i]
    }

    fn get_expanded_roots_of_unity(&self) -> &[blsScalar] {
        &self.expanded_roots_of_unity
    }

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> blsScalar {
        self.reverse_roots_of_unity[i]
    }

    fn get_reversed_roots_of_unity(&self) -> &[blsScalar] {
        &self.reverse_roots_of_unity
    }

    fn get_roots_of_unity_at(&self, i: usize) -> blsScalar {
        self.roots_of_unity[i]
    }

    fn get_roots_of_unity(&self) -> &[blsScalar] {
        &self.roots_of_unity
    }
}

pub fn new_fft_settings(_max_scale: usize) -> ZkFFTSettings {
    ZkFFTSettings::default()
}
