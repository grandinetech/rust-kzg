use crate::kzg_proofs::FFTSettings;
use crate::kzg_types::FsFr as BlstFr;
use kzg::{FFTFr, Fr as FFr};

pub const SCALE2_ROOT_OF_UNITY: [[u64; 4]; 32] = [
    [
        0x0000000000000001,
        0x0000000000000000,
        0x0000000000000000,
        0x0000000000000000,
    ],
    [
        0xffffffff00000000,
        0x53bda402fffe5bfe,
        0x3339d80809a1d805,
        0x73eda753299d7d48,
    ],
    [
        0x0001000000000000,
        0xec03000276030000,
        0x8d51ccce760304d0,
        0x0000000000000000,
    ],
    [
        0x7228fd3397743f7a,
        0xb38b21c28713b700,
        0x8c0625cd70d77ce2,
        0x345766f603fa66e7,
    ],
    [
        0x53ea61d87742bcce,
        0x17beb312f20b6f76,
        0xdd1c0af834cec32c,
        0x20b1ce9140267af9,
    ],
    [
        0x360c60997369df4e,
        0xbf6e88fb4c38fb8a,
        0xb4bcd40e22f55448,
        0x50e0903a157988ba,
    ],
    [
        0x8140d032f0a9ee53,
        0x2d967f4be2f95155,
        0x14a1e27164d8fdbd,
        0x45af6345ec055e4d,
    ],
    [
        0x5130c2c1660125be,
        0x98d0caac87f5713c,
        0xb7c68b4d7fdd60d0,
        0x6898111413588742,
    ],
    [
        0x4935bd2f817f694b,
        0x0a0865a899e8deff,
        0x6b368121ac0cf4ad,
        0x4f9b4098e2e9f12e,
    ],
    [
        0x4541b8ff2ee0434e,
        0xd697168a3a6000fe,
        0x39feec240d80689f,
        0x095166525526a654,
    ],
    [
        0x3c28d666a5c2d854,
        0xea437f9626fc085e,
        0x8f4de02c0f776af3,
        0x325db5c3debf77a1,
    ],
    [
        0x4a838b5d59cd79e5,
        0x55ea6811be9c622d,
        0x09f1ca610a08f166,
        0x6d031f1b5c49c834,
    ],
    [
        0xe206da11a5d36306,
        0x0ad1347b378fbf96,
        0xfc3e8acfe0f8245f,
        0x564c0a11a0f704f4,
    ],
    [
        0x6fdd00bfc78c8967,
        0x146b58bc434906ac,
        0x2ccddea2972e89ed,
        0x485d512737b1da3d,
    ],
    [
        0x034d2ff22a5ad9e1,
        0xae4622f6a9152435,
        0xdc86b01c0d477fa6,
        0x56624634b500a166,
    ],
    [
        0xfbd047e11279bb6e,
        0xc8d5f51db3f32699,
        0x483405417a0cbe39,
        0x3291357ee558b50d,
    ],
    [
        0xd7118f85cd96b8ad,
        0x67a665ae1fcadc91,
        0x88f39a78f1aeb578,
        0x2155379d12180caa,
    ],
    [
        0x08692405f3b70f10,
        0xcd7f2bd6d0711b7d,
        0x473a2eef772c33d6,
        0x224262332d8acbf4,
    ],
    [
        0x6f421a7d8ef674fb,
        0xbb97a3bf30ce40fd,
        0x652f717ae1c34bb0,
        0x2d3056a530794f01,
    ],
    [
        0x194e8c62ecb38d9d,
        0xad8e16e84419c750,
        0xdf625e80d0adef90,
        0x520e587a724a6955,
    ],
    [
        0xfece7e0e39898d4b,
        0x2f69e02d265e09d9,
        0xa57a6e07cb98de4a,
        0x03e1c54bcb947035,
    ],
    [
        0xcd3979122d3ea03a,
        0x46b3105f04db5844,
        0xc70d0874b0691d4e,
        0x47c8b5817018af4f,
    ],
    [
        0xc6e7a6ffb08e3363,
        0xe08fec7c86389bee,
        0xf2d38f10fbb8d1bb,
        0x0abe6a5e5abcaa32,
    ],
    [
        0x5616c57de0ec9eae,
        0xc631ffb2585a72db,
        0x5121af06a3b51e3c,
        0x73560252aa0655b2,
    ],
    [
        0x92cf4deb77bd779c,
        0x72cf6a8029b7d7bc,
        0x6e0bcd91ee762730,
        0x291cf6d68823e687,
    ],
    [
        0xce32ef844e11a51e,
        0xc0ba12bb3da64ca5,
        0x0454dc1edc61a1a3,
        0x019fe632fd328739,
    ],
    [
        0x531a11a0d2d75182,
        0x02c8118402867ddc,
        0x116168bffbedc11d,
        0x0a0a77a3b1980c0d,
    ],
    [
        0xe2d0a7869f0319ed,
        0xb94f1101b1d7a628,
        0xece8ea224f31d25d,
        0x23397a9300f8f98b,
    ],
    [
        0xd7b688830a4f2089,
        0x6558e9e3f6ac7b41,
        0x99e276b571905a7d,
        0x52dd465e2f094256,
    ],
    [
        0x474650359d8e211b,
        0x84d37b826214abc6,
        0x8da40c1ef2bb4598,
        0x0c83ea7744bf1bee,
    ],
    [
        0x694341f608c9dd56,
        0xed3a181fabb30adc,
        0x1339a815da8b398f,
        0x2c6d4e4511657e1e,
    ],
    [
        0x63e7cb4906ffc93f,
        0xf070bb00e28a193d,
        0xad1715b02e5713b5,
        0x4b5371495990693f,
    ],
];

impl FFTFr<BlstFr> for FFTSettings {
    fn fft_fr(&self, data: &[BlstFr], inverse: bool) -> Result<Vec<BlstFr>, String> {
        if data.len() > self.max_width {
            return Err(String::from(
                "data length is longer than allowed max width",
            ));
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
            ret[..data.len()].iter_mut().for_each(|f| *f = BlstFr::mul(f, &inv_fr_len));
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
                fft_fr_fast(&mut ret[half..], &data[stride..], stride * 2, roots, roots_stride * 2);
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

pub fn fft_fr_slow( ret: &mut [BlstFr], data: &[BlstFr], stride: usize, roots: &[BlstFr], roots_stride: usize,) {
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
