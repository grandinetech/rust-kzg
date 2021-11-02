pub type Scalar = blst::blst_scalar;

pub trait Fr: Clone {
    // Assume that Fr can't fail on creation

    fn default() -> Self; // -> Result<Self, String>;

    fn zero() -> Self; // -> Result<Self, String>;

    fn one() -> Self; // -> Result<Self, String>;

    fn rand() -> Self; // -> Result<Self, String>;

    fn from_u64_arr(u: &[u64; 4]) -> Self;

    fn from_u64(u: u64) -> Self;

    fn is_one(&self) -> bool;

    fn is_zero(&self) -> bool;

    fn sqr(&self) -> Self;

    fn mul(&self, b: &Self) -> Self;

    fn add(&self, b: &Self) -> Self;

    fn sub(&self, b: &Self) -> Self;

    fn eucl_inverse(&self) -> Self;

    fn negate(&self) -> Self;

    fn inverse(&self) -> Self;

    fn pow(&self, n: usize) -> Self;

    fn equals(&self, b: &Self) -> bool;

    fn get_scalar(&self) -> Scalar;

    fn from_scalar(scalar: &Scalar) -> Self;

    // Other teams, aside from the c-kzg bindings team, may as well leave its body empty
    fn destroy(&mut self);
}

pub trait G1<Coeff: Fr>: Clone {
    fn default() -> Self;

    fn rand() -> Self;

    fn add_or_double(&self, b: &Self) -> Self;

    fn equals(&self, b: &Self) -> bool;

    fn mul(&self, b: &Coeff) -> Self;

    fn sub(&self, b: &Self) -> Self;

    // Other teams, aside from the c-kzg bindings team, may as well leave its body empty
    fn destroy(&mut self);
}

pub trait G2: Clone {
    // TODO: populate with needed fns
}

pub trait FFTFr<Coeff: Fr> {
    fn fft_fr(&self, data: &[Coeff], inverse: bool) -> Result<Vec<Coeff>, String>;
}

pub trait FFTG1<TFr: Fr, Coeff: G1<TFr>> {
    fn fft_g1(&self, data: &[Coeff], inverse: bool) -> Result<Vec<Coeff>, String>;
}

pub trait DAS<Coeff: Fr> {
    fn das_fft_extension(&self, evens: &[Coeff]) -> Result<Vec<Coeff>, String>;
}

pub trait ZeroPoly<Coeff: Fr, Polynomial: Poly<Coeff>> {
    fn do_zero_poly_mul_partial(&self, idxs: &[usize], stride: usize)
        -> Result<Polynomial, String>;

    fn reduce_partials(
        &self,
        domain_size: usize,
        partials: &[Polynomial],
    ) -> Result<Polynomial, String>;

    fn zero_poly_via_multiplication(
        &self,
        domain_size: usize,
        idxs: &[usize],
    ) -> Result<(Vec<Coeff>, Polynomial), String>;
}

pub trait FFTSettings<Coeff: Fr>: Clone {
    fn default() -> Self;

    fn new(scale: usize) -> Result<Self, String>;

    fn get_max_width(&self) -> usize;

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> Coeff;

    fn get_expanded_roots_of_unity(&self) -> &[Coeff];

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> Coeff;

    fn get_reversed_roots_of_unity(&self) -> &[Coeff];

    // Other teams, aside from the c-kzg bindings team, may as well leave its body empty
    fn destroy(&mut self);
}

pub trait Poly<Coeff: Fr>: Clone {
    fn default() -> Self;

    fn new(size: usize) -> Result<Self, String>;

    fn get_coeff_at(&self, i: usize) -> Coeff;

    fn set_coeff_at(&mut self, i: usize, x: &Coeff);

    fn get_coeffs(&self) -> &[Coeff];

    fn len(&self) -> usize;

    fn eval(&self, x: &Coeff) -> Coeff;

    fn scale(&mut self);

    fn unscale(&mut self);

    fn inverse(&mut self, new_len: usize) -> Result<Self, String>;

    fn div(&mut self, x: &Self) -> Result<Self, String>;

    // Other teams, aside from the c-kzg bindings team, may as well leave its body empty
    fn destroy(&mut self);
}
