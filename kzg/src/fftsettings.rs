use super::{Error, Fr, G1, Poly};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFTSettings {
    pub max_width: usize,
    pub root_of_unity: Fr,
    pub expanded_roots_of_unity: *mut Fr,
    pub reverse_roots_of_unity: *mut Fr
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct BlstFp {
    pub l: [u64; 6]
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct BlstFp2 {
    pub fp: [BlstFp; 2]
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct BlstP2 {
    pub x: BlstFp2,
    pub y: BlstFp2,
    pub z: BlstFp2
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct BlstP1 {
    pub x: BlstFp,
    pub y: BlstFp,
    pub z: BlstFp
}

const G1_GENERATOR: G1 = G1 {
    x: BlstFp {
        l: [0x5cb38790fd530c16,
            0x7817fc679976fff5,
            0x154f95c7143ba1c1,
            0xf0ae6acdf3d0e747,
            0xedce6ecc21dbf440,
            0x120177419e0bfb75
        ]
    },
    y: BlstFp {
        l: [0xbaac93d50ce72271,
            0x8c22631a7918fd8e,
            0xdd595f13570725ce,
            0x51ac582950405194,
            0x0e1c8c3fad0059c0,
            0x0bbc3efc5008a26a
        ]
    },
    z: BlstFp {
        l: [0x760900000002fffd,
            0xebf4000bc40c0002,
            0x5f48985753c758ba,
            0x77ce585370525745,
            0x5c071a97a256ec6d,
            0x15f65ec3fa80e493
        ]
    }
};

extern "C" {
    fn new_fft_settings(settings: *mut FFTSettings, max_scale: u32) -> Error;
    fn free_fft_settings(settings: *mut FFTSettings);
    fn fft_fr(output: *mut Fr, input: *const Fr, inverse: bool, n: u64, fs: *const FFTSettings) -> Error;
    fn fft_g1(output: *mut G1, input: *const G1, inverse: bool, n: u64, fs: *const FFTSettings) -> Error;
    fn poly_mul(output: *mut Poly, a: *const Poly, b: *const Poly, fs: *const FFTSettings) -> Error;
    // Blst
    fn g1_add_or_dbl(out: *mut G1, a: *const G1, b: *const G1);
    fn g1_equal(a: *const G1, b: *const G1) -> bool;
    fn g1_mul(out: *mut G1, a: *const G1, b: *const Fr);
}

impl FFTSettings {
    pub fn default() -> Self {
        Self {
            max_width: 16,
            root_of_unity: Fr::default(),
            expanded_roots_of_unity: &mut Fr::default(),
            reverse_roots_of_unity: &mut Fr::default()
        }
    }

    pub fn new(max_scale: u32) -> Result<Self, Error> {
        let mut settings = FFTSettings::default();
        unsafe {
            return match new_fft_settings(&mut settings, max_scale) {
                Error::KzgOk => Ok(settings),
                e => {
                    println!("Error in \"FFTSettings::new\" ==> {:?}", e);
                    Err(e)
                }
            }
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            free_fft_settings(self);
        }
    }

    pub fn fft_fr(&self, input: &mut Vec<Fr>, inverse: bool) -> Vec<Fr> {
        let output = match FFTSettings::_fft_fr(input.as_mut_ptr(), inverse, input.len() as u64, self) {
            Ok(fr) => fr,
            Err(e) => panic!("Error in \"FFTSettings::fft_fr\" ==> {:?}", e)
        };
        output
    }

    fn _fft_fr(input: *const Fr, inverse: bool, n: u64, fs: *const FFTSettings) -> Result<Vec<Fr>, Error> {
        let mut output = vec![Fr::default(); n as usize];
        unsafe {
            return match fft_fr(output.as_mut_ptr(), input, inverse, n, fs) {
                Error::KzgOk => Ok(output),
                e => Err(e)
            }
        }
    }

    pub fn fft_g1(&self, input: &mut Vec<G1>, inverse: bool) -> Vec<G1> {
        let output = match FFTSettings::_fft_g1(input.as_mut_ptr(), inverse, input.len() as u64, self) {
            Ok(g) => g,
            Err(e) => panic!("Error in \"FFTSettings::fft_g1\" ==> {:?}", e)
        };
        output
    }

    fn _fft_g1(input: *mut G1, inverse: bool, n: u64, fs: *const FFTSettings) -> Result<Vec<G1>, Error> {
        let mut output = vec![G1::default(); n as usize];
        unsafe {
            return match fft_g1(output.as_mut_ptr(), input, inverse, n, fs) {
                Error::KzgOk => Ok(output),
                e => Err(e)
            }
        }
    }

    // https://github.com/benjaminion/c-kzg/blob/63612c11192cea02b2cb78aa677f570041b6b763/src/fft_g1.c#L95
    pub fn make_data(n: usize) -> Vec<G1> {
        // It's not mentioned in c-kzg implementation but for safety reasons allocate (n + 1) space
        // because we'll try to access one element further its initial size in the first for loop
        let mut out_val = vec![G1_GENERATOR; n + 1];
        let out_ptr: *mut G1 = out_val.as_mut_ptr();
        // Multiples of g1_gen
        if n == 0 { return vec![G1::default(); 0]; }
        for i in 1..n as isize {
            unsafe {
                *out_ptr.offset(i + 1) = FFTSettings::add_or_dbl(*out_ptr.offset(i - 1), &G1_GENERATOR);
            }
        }
        unsafe {
            let mut out = vec![G1::default(); 0];
            let mut i: isize = 0;
            while !out_ptr.offset(i).is_null() && i < n as isize {
                out.push(*out_ptr.offset(i) as G1);
                i += 1;
            }
            return out
        }
    }

    fn add_or_dbl(a: G1, b: *const G1) -> G1 {
        let mut out = G1::default();
        unsafe {
            g1_add_or_dbl(&mut out, &a, b);
        }
        out
    }

    pub fn g1_equal(a: *const G1, b: *const G1) -> bool {
        unsafe {
            return g1_equal(a, b);
        }
    }

    pub fn poly_mul(a: *const Poly, b: *const Poly, fs: *const FFTSettings) -> Result<Poly, Error> {
        let mut output = Poly::default();
        unsafe {
            return match poly_mul(&mut output, a, b, fs) {
                Error::KzgOk => Ok(output),
                e => {
                    println!("Error in \"FFTSettings::poly_mul\" ==> {:?}", e);
                    Err(e)
                }
            }
        }
    }

    // Used only for benchmarks
    pub fn bench_fft_fr(scale: u64) {
        let mut fs = match FFTSettings::new(scale as u32) {
            Ok(s) => s,
            Err(_) => FFTSettings::default()
        };
        let mut data = vec![Fr::default(); fs.max_width];
        for i in 0..fs.max_width {
            data[i] = Fr::rand();
        }
        fs.fft_fr(&mut data, false);
        fs.destroy();
    }

    // Used only for benchmarks
    pub fn bench_fft_g1(scale: u64) {
        let mut fs = match FFTSettings::new(scale as u32) {
            Ok(s) => s,
            Err(_) => FFTSettings::default()
        };
        let mut data = vec![G1::default(); fs.max_width];
        for i in 0..fs.max_width {
            data[i] = FFTSettings::rand_g1();
        }
        fs.fft_g1(&mut data, false);
        fs.destroy();
    }

    pub fn rand_g1() -> G1 {
        let mut ret = G1::default();
        let random = Fr::rand();
        unsafe {
            g1_mul(&mut ret, &G1_GENERATOR, &random);
        }
        ret
    }
}
