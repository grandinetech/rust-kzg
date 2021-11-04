use crate::kzg_proofs::FFTSettings;
use crate::kzg_types::ArkG1;
use crate::kzg_types::FsFr as BlstFr;
use blst::{
    blst_fp, blst_p1, blst_p1_add_or_double, blst_p1_cneg, blst_p1_mult, blst_scalar,
    blst_scalar_from_fr,
};
use kzg::{FFTSettings as Fs, Fr, FFTG1, G1};
use std::mem::size_of;

//Needed for g1_mul with Ark
//use crate::utils::{blst_p1_into_pc_g1projective, pc_g1projective_into_blst_p1, blst_fr_into_pc_fr};
//use std::ops::MulAssign;

/** The G1 generator */
pub(crate) const G1_GENERATOR: blst_p1 = blst_p1 {
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
pub(crate) const G1_IDENTITY: ArkG1 = ArkG1(blst_p1{
    x: blst_fp {
        l: [0, 0, 0, 0, 0, 0],
    },
    y: blst_fp {
        l: [0, 0, 0, 0, 0, 0],
    },
    z: blst_fp {
        l: [0, 0, 0, 0, 0, 0],
    },
});

pub fn make_data(data: usize) -> Vec<ArkG1> {
    let mut vec = Vec::new();
    if data == 0 {
        vec
    } else {
        vec.push(ArkG1(G1_GENERATOR));
        for i in 1..data as u64 {
            let mut temp = vec[(i - 1) as usize].clone();
            vec.push(temp.add_or_double(&ArkG1(G1_GENERATOR)));
        }
        vec
    }
}

impl FFTG1<ArkG1> for FFTSettings {
    fn fft_g1(&self, data: &[ArkG1], inverse: bool) -> Result<Vec<ArkG1>, String> {
        let n = self.max_width as usize;

        let stride: usize = self.max_width / data.len();
        let mut out = vec![ArkG1::default(); data.len()];
        if inverse {
            let mut inv_len = BlstFr::default();
            inv_len = Fr::from_u64(self.max_width as u64);
            inv_len = inv_len.inverse();

            fft_g1_fast(
                &mut out,
                data,
                1,
                self.get_reversed_roots_of_unity(),
                stride,
                1
            );
            for i in 0..n {
                out[i] = g1_mul(&out[i], &inv_len);
            }
            Ok(out)
        } else {
            fft_g1_fast(
                &mut out,
                data,
                1,
                self.get_expanded_roots_of_unity(),
                stride,
                1
            );
            Ok(out)
        }
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
    let mut v;
    let mut last;
    let mut jv;

    let mut r;

    for i in 0..data.len() {
        last = g1_mul(&data[0], &roots[0]);
        for j in 1..data.len() {
            jv = data[j * stride].clone();
            r = roots[((i * j) % data.len()) * roots_stride];
            v = g1_mul(&jv, &r);
            unsafe {
                blst_p1_add_or_double(&mut last.0, &last.0, &v.0);
            }
            ret[i].0.x = last.0.x;
            ret[i].0.y = last.0.y;
            ret[i].0.z = last.0.z;
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
        fft_g1_fast(&mut ret[..half], data, stride * 2, roots, roots_stride * 2, 1);
        fft_g1_fast(
            &mut ret[half..],
            &data[stride..],
            stride * 2,
            roots,
            roots_stride * 2,
            1
        );
        for i in 0..half {
            let y_times_root = g1_mul(&ret[i + half], &roots[i * roots_stride]);
            ret[i + half] = g1_sub(&ret[i], &y_times_root);
            unsafe {
                blst_p1_add_or_double(&mut ret[i].0, &ret[i].0, &y_times_root.0);
            }
        }
    } else {
        for i in 0..ret.len() {
            ret[i].0.x = data[i].0.x;
            ret[i].0.y = data[i].0.y;
            ret[i].0.z = data[i].0.z;
        }
    }
}

pub fn g1_sub(a: &ArkG1, b: &ArkG1) -> ArkG1 {
    let mut bneg = b.0;
    let mut out = blst_p1::default();
    unsafe {
        blst_p1_cneg(&mut bneg, true);
        blst_p1_add_or_double(&mut out, &a.0, &bneg);
    }
    ArkG1(out)
}

// Slower than Blst but it is using Ark functions and less lines
// pub fn g1_mul( a: &ArkG1, b: &BlstFr) -> ArkG1 {

//     let mut ap1 = blst_p1_into_pc_g1projective(&a.0).unwrap();
//     let bfr = blst_fr_into_pc_fr(&b);
//     ap1.mul_assign(bfr);
//     let result = pc_g1projective_into_blst_p1(ap1).unwrap();
//     result
// }

pub fn g1_mul(a: &ArkG1, b: &BlstFr) -> ArkG1 {
    let mut s: blst_scalar = blst_scalar::default();
    unsafe {
        blst_scalar_from_fr(&mut s, &b.0 as *const _);
    }
    // Count the number of bytes to be multiplied.
    let mut i = size_of::<blst_scalar>();

    while (i != 0) && (s.b[i - 1] == 0) {
        i = i - 1;
    }
    if i == 0 {
        G1_IDENTITY
    } else if i == 1 && s.b[0] == 1 {
        ArkG1(a.0)
    } else {
        // Count the number of bits to be multiplied.
        let mut out = blst_p1::default();
        unsafe {
            blst_p1_mult(
                &mut out,
                &a.0 as *const _,
                &s.b as *const _,
                8 * i - 7 + log_2_byte(s.b[i - 1]),
            );
        }
        ArkG1(out)
    }
}

pub fn log_2_byte(b: u8) -> usize {
    let mut r = if b > 0xF { 1 } else { 0 } << 2;
    let mut b = b >> r;
    let shift = if b > 0x3 { 1 } else { 0 } << 1;
    b >>= shift + 1;
    r |= shift | b;
    r.into()
}
