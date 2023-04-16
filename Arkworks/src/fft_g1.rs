use crate::kzg_proofs::FFTSettings;
use crate::kzg_types::{ArkG1, FsFr as BlstFr};
use crate::utils::{
    blst_fr_into_pc_fr, blst_p1_into_pc_g1projective, pc_g1projective_into_blst_p1,
};
use ark_bls12_381::Fr as ArkFr;
use ark_ec::msm::VariableBaseMSM;
use ark_ec::ProjectiveCurve;
use ark_ff::PrimeField;
use blst::{blst_fp, blst_p1};
use kzg::G1Mul;
use kzg::{Fr, FFTG1, G1};

pub const G1_NEGATIVE_GENERATOR: blst_p1 = blst_p1 {
    x: blst_fp {
        l: [
            0x5cb38790fd530c16,
            0x7817fc679976fff5,
            0x154f95c7143ba1c1,
            0xf0ae6acdf3d0e747,
            0xedce6ecc21dbf440,
            0x120177419e0bfb75,
        ],
    },
    y: blst_fp {
        l: [
            0xff526c2af318883a,
            0x92899ce4383b0270,
            0x89d7738d9fa9d055,
            0x12caf35ba344c12a,
            0x3cff1b76964b5317,
            0x0e44d2ede9774430,
        ],
    },
    z: blst_fp {
        l: [
            0x760900000002fffd,
            0xebf4000bc40c0002,
            0x5f48985753c758ba,
            0x77ce585370525745,
            0x5c071a97a256ec6d,
            0x15f65ec3fa80e493,
        ],
    },
};

/** The G1 generator */
pub const G1_GENERATOR: blst_p1 = blst_p1 {
    x: blst_fp {
        l: [
            0x5cb38790fd530c16,
            0x7817fc679976fff5,
            0x154f95c7143ba1c1,
            0xf0ae6acdf3d0e747,
            0xedce6ecc21dbf440,
            0x120177419e0bfb75,
        ],
    },
    y: blst_fp {
        l: [
            0xbaac93d50ce72271,
            0x8c22631a7918fd8e,
            0xdd595f13570725ce,
            0x51ac582950405194,
            0x0e1c8c3fad0059c0,
            0x0bbc3efc5008a26a,
        ],
    },
    z: blst_fp {
        l: [
            0x760900000002fffd,
            0xebf4000bc40c0002,
            0x5f48985753c758ba,
            0x77ce585370525745,
            0x5c071a97a256ec6d,
            0x15f65ec3fa80e493,
        ],
    },
};

/** The G1 identity/infinity */
#[rustfmt::skip]
pub const G1_IDENTITY: ArkG1 = ArkG1(blst_p1 {
    x: blst_fp { l: [0, 0, 0, 0, 0, 0], },
    y: blst_fp { l: [0, 0, 0, 0, 0, 0], },
    z: blst_fp { l: [0, 0, 0, 0, 0, 0], },
});

pub fn g1_linear_combination(out: &mut ArkG1, points: &[ArkG1], scalars: &[BlstFr], _len: usize) {
    let mut ark_points = Vec::new();
    let mut ark_scalars = Vec::new();

    for point in points.iter() {
        ark_points.push(
            blst_p1_into_pc_g1projective(&point.0)
                .unwrap()
                .into_affine(),
        );
    }

    for scalar in scalars.iter() {
        ark_scalars.push(ArkFr::into_repr(&blst_fr_into_pc_fr(scalar)));
    }

    let res = VariableBaseMSM::multi_scalar_mul(ark_points.as_slice(), ark_scalars.as_slice());
    *out = pc_g1projective_into_blst_p1(res).unwrap();
}

pub fn make_data(data: usize) -> Vec<ArkG1> {
    let mut vec = Vec::new();
    if data != 0 {
        vec.push(ArkG1(G1_GENERATOR));
        for i in 1..data as u64 {
            let mut temp = vec[(i - 1) as usize];
            vec.push(temp.add_or_dbl(&ArkG1(G1_GENERATOR)));
        }
    }
    vec
}

impl FFTG1<ArkG1> for FFTSettings {
    fn fft_g1(&self, data: &[ArkG1], inverse: bool) -> Result<Vec<ArkG1>, String> {
        if data.len() > self.max_width {
            return Err(String::from("data length is longer than allowed max width"));
        }
        if !data.len().is_power_of_two() {
            return Err(String::from("data length is not power of 2"));
        }

        let stride: usize = self.max_width / data.len();
        let mut ret = vec![ArkG1::default(); data.len()];

        let roots = if inverse {
            &self.reverse_roots_of_unity
        } else {
            &self.expanded_roots_of_unity
        };

        fft_g1_fast(&mut ret, data, 1, roots, stride, 1);

        if inverse {
            let inv_fr_len = BlstFr::from_u64(data.len() as u64).inverse();
            ret[..data.len()]
                .iter_mut()
                .for_each(|f| *f = f.mul(&inv_fr_len));
        }

        Ok(ret)
    }
}

pub fn fft_g1_slow(
    ret: &mut [ArkG1],
    data: &[ArkG1],
    stride: usize,
    roots: &[BlstFr],
    roots_stride: usize,
    _width: usize,
) {
    for i in 0..data.len() {
        ret[i] = data[0].mul(&roots[0]);
        for j in 1..data.len() {
            let jv = data[j * stride];
            let r = roots[((i * j) % data.len()) * roots_stride];
            let v = jv.mul(&r);
            ret[i] = ret[i].add_or_dbl(&v);
        }
    }
}

pub fn fft_g1_fast(
    ret: &mut [ArkG1],
    data: &[ArkG1],
    stride: usize,
    roots: &[BlstFr],
    roots_stride: usize,
    _width: usize,
) {
    let half = ret.len() / 2;
    if half > 0 {
        #[cfg(feature = "parallel")]
        {
            let (lo, hi) = ret.split_at_mut(half);
            rayon::join(
                || fft_g1_fast(hi, &data[stride..], stride * 2, roots, roots_stride * 2, 1),
                || fft_g1_fast(lo, data, stride * 2, roots, roots_stride * 2, 1),
            );
        }

        #[cfg(not(feature = "parallel"))]
        {
            fft_g1_fast(
                &mut ret[..half],
                data,
                stride * 2,
                roots,
                roots_stride * 2,
                1,
            );
            fft_g1_fast(
                &mut ret[half..],
                &data[stride..],
                stride * 2,
                roots,
                roots_stride * 2,
                1,
            );
        }

        for i in 0..half {
            let y_times_root = ret[i + half].mul(&roots[i * roots_stride]);
            ret[i + half] = ret[i].sub(&y_times_root);
            ret[i] = ret[i].add_or_dbl(&y_times_root);
        }
    } else {
        ret[0] = data[0];
    }
}

pub fn log_2_byte(b: u8) -> usize {
    let mut r = u8::from(b > 0xF) << 2;
    let mut b = b >> r;
    let shift = u8::from(b > 0x3) << 1;
    b >>= shift + 1;
    r |= shift | b;
    r.into()
}
