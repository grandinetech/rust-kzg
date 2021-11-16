use kzg::{FFTFr, FFTG1, Fr, G1, G1Mul, Poly};
use crate::kzg_types::{FsFFTSettings, FsFr, FsG1, FsPoly};

impl FsFFTSettings {
    pub fn toeplitz_part_1(&self, x: &[FsG1]) -> Vec<FsG1> {
        let n = x.len();
        let n2 = n * 2;
        let mut x_ext = Vec::new();
        for i in 0..n {
            x_ext.push(x[i]);
        }

        for i in n..n2 {
            x_ext.push(FsG1::identity());
        }

        let ret = self.fft_g1(&x_ext, false).unwrap();

        assert_eq!(ret.len(), n2);

        ret
    }

    /// poly and x_ext_fft should be of same length
    pub fn toeplitz_part_2(&self, poly: &FsPoly, x_ext_fft: &[FsG1]) -> Vec<FsG1> {
        assert_eq!(poly.len(), x_ext_fft.len());

        let coeffs_fft = self.fft_fr(&poly.coeffs, false).unwrap();
        let mut ret = Vec::new();

        for i in 0..poly.len() {
            ret.push(x_ext_fft[i].mul(&coeffs_fft[i]));
        }

        assert_eq!(ret.len(), poly.len());

        ret
    }

    pub fn toeplitz_part_3(&self, h_ext_fft: &[FsG1]) -> Vec<FsG1> {
        let n2 = h_ext_fft.len();
        let n = n2 / 2;

        let mut ret = self.fft_g1(&h_ext_fft, true).unwrap();

        for i in n..n2 {
            ret[i] = FsG1::identity();
        }

        assert_eq!(ret.len(), n2);

        ret
    }
}


impl FsPoly {
    pub fn toeplitz_coeffs_stride(&self, offset: usize, stride: usize) -> FsPoly {
        let n = self.len();
        let k = n / stride;
        let k2 = k * 2;

        let mut ret = FsPoly { coeffs: vec![FsFr::default(); k2] };
        ret.coeffs[0] = self.coeffs[n - 1 - offset];

        let mut i = 1;
        while i <= k + 1 && i < k2 {
            ret.coeffs[i] = FsFr::zero();
            i += 1;
        }

        let mut i = k + 2;
        let mut j = 2 * stride - offset - 1;
        while i < k2 {
            ret.coeffs[i] = self.coeffs[j];

            i += 1;
            j += stride;
        }

        assert_eq!(ret.len(), n * 2 / stride);

        ret
    }

    pub fn toeplitz_coeffs_step(&self) -> FsPoly {
        let ret = self.toeplitz_coeffs_stride(0, 1);

        assert_eq!(ret.len(), self.len() * 2);

        ret
    }
}