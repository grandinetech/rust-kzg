#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

pub mod eip_4844;

pub trait Fr: Default + Clone {
    fn null() -> Self;

    fn zero() -> Self;

    fn one() -> Self;

    #[cfg(feature = "rand")]
    fn rand() -> Self;

    fn from_bytes(bytes: &[u8]) -> Result<Self, String>;

    fn from_hex(hex: &str) -> Result<Self, String>;

    fn from_u64_arr(u: &[u64; 4]) -> Self;

    fn from_u64(u: u64) -> Self;

    fn to_bytes(&self) -> [u8; 32];

    fn to_u64_arr(&self) -> [u64; 4];

    fn is_one(&self) -> bool;

    fn is_zero(&self) -> bool;

    fn is_null(&self) -> bool;

    fn sqr(&self) -> Self;

    fn mul(&self, b: &Self) -> Self;

    fn add(&self, b: &Self) -> Self;

    fn sub(&self, b: &Self) -> Self;

    fn eucl_inverse(&self) -> Self;

    fn negate(&self) -> Self;

    fn inverse(&self) -> Self;

    fn pow(&self, n: usize) -> Self;

    fn div(&self, b: &Self) -> Result<Self, String>;

    fn equals(&self, b: &Self) -> bool;
}

pub trait G1: Clone + Default {
    fn identity() -> Self;

    fn generator() -> Self;

    fn negative_generator() -> Self;

    #[cfg(feature = "rand")]
    fn rand() -> Self;

    fn from_bytes(bytes: &[u8]) -> Result<Self, String>;

    fn from_hex(hex: &str) -> Result<Self, String>;

    fn to_bytes(&self) -> [u8; 48];

    fn add_or_dbl(&mut self, b: &Self) -> Self;

    fn is_inf(&self) -> bool;

    fn is_valid(&self) -> bool;

    fn dbl(&self) -> Self;

    fn add(&self, b: &Self) -> Self;

    fn sub(&self, b: &Self) -> Self;

    fn equals(&self, b: &Self) -> bool;
}

pub trait G1Mul<Fr>: Clone {
    fn mul(&self, b: &Fr) -> Self;
}

pub trait G2: Clone + Default {
    fn generator() -> Self;

    fn negative_generator() -> Self;

    fn from_bytes(bytes: &[u8]) -> Result<Self, String>;

    fn to_bytes(&self) -> [u8; 96];

    fn add_or_dbl(&mut self, b: &Self) -> Self;

    fn dbl(&self) -> Self;

    fn sub(&self, b: &Self) -> Self;

    fn equals(&self, b: &Self) -> bool;
}

pub trait G2Mul<Fr>: Clone {
    fn mul(&self, b: &Fr) -> Self;
}

pub trait FFTFr<Coeff: Fr> {
    fn fft_fr(&self, data: &[Coeff], inverse: bool) -> Result<Vec<Coeff>, String>;
}

pub trait FFTG1<Coeff: G1> {
    fn fft_g1(&self, data: &[Coeff], inverse: bool) -> Result<Vec<Coeff>, String>;
}

pub trait DAS<Coeff: Fr> {
    fn das_fft_extension(&self, evens: &[Coeff]) -> Result<Vec<Coeff>, String>;
}

pub trait ZeroPoly<Coeff: Fr, Polynomial: Poly<Coeff>> {
    /// Calculates the minimal polynomial that evaluates to zero for powers of roots of unity at the
    /// given indices.
    /// The returned polynomial has a length of `idxs.len() + 1`.
    ///
    /// Uses straightforward long multiplication to calculate the product of `(x - r^i)` where `r`
    /// is a root of unity and the `i`s are the indices at which it must evaluate to zero.
    fn do_zero_poly_mul_partial(&self, idxs: &[usize], stride: usize)
        -> Result<Polynomial, String>;

    /// Reduce partials using a specified domain size.
    /// Calculates the product of all polynomials via FFT and then applies an inverse FFT to produce
    /// a new Polynomial.
    fn reduce_partials(
        &self,
        domain_size: usize,
        partials: &[Polynomial],
    ) -> Result<Polynomial, String>;

    /// Calculate the minimal polynomial that evaluates to zero for powers of roots of unity that
    /// correspond to missing indices.
    /// This is done simply by multiplying together `(x - r^i)` for all the `i` that are missing
    /// indices, using a combination of direct multiplication ([`Self::do_zero_poly_mul_partial()`])
    /// and iterated multiplication via convolution (#reduce_partials).
    /// Also calculates the FFT (the "evaluation polynomial").
    fn zero_poly_via_multiplication(
        &self,
        domain_size: usize,
        idxs: &[usize],
    ) -> Result<(Vec<Coeff>, Polynomial), String>;
}

pub trait FFTSettings<Coeff: Fr>: Default + Clone {
    fn new(scale: usize) -> Result<Self, String>;

    fn get_max_width(&self) -> usize;

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> Coeff;

    fn get_expanded_roots_of_unity(&self) -> &[Coeff];

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> Coeff;

    fn get_reversed_roots_of_unity(&self) -> &[Coeff];

    fn get_roots_of_unity_at(&self, i: usize) -> Coeff;

    fn get_roots_of_unity(&self) -> &[Coeff];
}

pub trait FFTSettingsPoly<Coeff: Fr, Polynomial: Poly<Coeff>, FSettings: FFTSettings<Coeff>> {
    fn poly_mul_fft(
        a: &Polynomial,
        b: &Polynomial,
        len: usize,
        fs: Option<&FSettings>,
    ) -> Result<Polynomial, String>;
}

pub trait Poly<Coeff: Fr>: Default + Clone {
    fn new(size: usize) -> Self;

    fn get_coeff_at(&self, i: usize) -> Coeff;

    fn set_coeff_at(&mut self, i: usize, x: &Coeff);

    fn get_coeffs(&self) -> &[Coeff];

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn eval(&self, x: &Coeff) -> Coeff;

    fn scale(&mut self);

    fn unscale(&mut self);

    fn inverse(&mut self, new_len: usize) -> Result<Self, String>;

    fn div(&mut self, x: &Self) -> Result<Self, String>;

    fn long_div(&mut self, x: &Self) -> Result<Self, String>;

    fn fast_div(&mut self, x: &Self) -> Result<Self, String>;

    fn mul_direct(&mut self, x: &Self, len: usize) -> Result<Self, String>;
}

pub trait PolyRecover<Coeff: Fr, Polynomial: Poly<Coeff>, FSettings: FFTSettings<Coeff>> {
    fn recover_poly_coeffs_from_samples(
        samples: &[Option<Coeff>],
        fs: &FSettings,
    ) -> Result<Polynomial, String>;

    fn recover_poly_from_samples(
        samples: &[Option<Coeff>],
        fs: &FSettings,
    ) -> Result<Polynomial, String>;
}

pub trait KZGSettings<
    Coeff1: Fr,
    Coeff2: G1,
    Coeff3: G2,
    Fs: FFTSettings<Coeff1>,
    Polynomial: Poly<Coeff1>,
>: Default + Clone
{
    fn new(
        secret_g1: &[Coeff2],
        secret_g2: &[Coeff3],
        length: usize,
        fs: &Fs,
    ) -> Result<Self, String>;

    fn commit_to_poly(&self, p: &Polynomial) -> Result<Coeff2, String>;

    fn compute_proof_single(&self, p: &Polynomial, x: &Coeff1) -> Result<Coeff2, String>;

    fn check_proof_single(
        &self,
        com: &Coeff2,
        proof: &Coeff2,
        x: &Coeff1,
        value: &Coeff1,
    ) -> Result<bool, String>;

    fn compute_proof_multi(&self, p: &Polynomial, x: &Coeff1, n: usize) -> Result<Coeff2, String>;

    fn check_proof_multi(
        &self,
        com: &Coeff2,
        proof: &Coeff2,
        x: &Coeff1,
        values: &[Coeff1],
        n: usize,
    ) -> Result<bool, String>;

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> Coeff1;

    fn get_roots_of_unity_at(&self, i: usize) -> Coeff1;
}

pub trait FK20SingleSettings<
    Coeff1: Fr,
    Coeff2: G1,
    Coeff3: G2,
    Fs: FFTSettings<Coeff1>,
    Polynomial: Poly<Coeff1>,
    Ks: KZGSettings<Coeff1, Coeff2, Coeff3, Fs, Polynomial>,
>: Default + Clone
{
    fn new(ks: &Ks, n2: usize) -> Result<Self, String>;

    fn data_availability(&self, p: &Polynomial) -> Result<Vec<Coeff2>, String>;

    fn data_availability_optimized(&self, p: &Polynomial) -> Result<Vec<Coeff2>, String>;
}

pub trait FK20MultiSettings<
    Coeff1: Fr,
    Coeff2: G1,
    Coeff3: G2,
    Fs: FFTSettings<Coeff1>,
    Polynomial: Poly<Coeff1>,
    Ks: KZGSettings<Coeff1, Coeff2, Coeff3, Fs, Polynomial>,
>: Default + Clone
{
    fn new(ks: &Ks, n2: usize, chunk_len: usize) -> Result<Self, String>;

    fn data_availability(&self, p: &Polynomial) -> Result<Vec<Coeff2>, String>;

    fn data_availability_optimized(&self, p: &Polynomial) -> Result<Vec<Coeff2>, String>;
}
