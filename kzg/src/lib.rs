#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use common_utils::{blob_to_polynomial, compute_r_powers, fr_batch_inv, evaluate_polynomial_in_evaluation_form, compute_challenge, verify_blob_kzg_proof_rust, validate_batched_input, compute_challenges_and_evaluate_polynomial};
use eip_4844::FIELD_ELEMENTS_PER_BLOB;

pub mod eip_4844;
pub mod common_utils;

pub trait Fr: Default + Clone + PartialEq {
    fn null() -> Self;

    fn zero() -> Self;

    fn one() -> Self;

    #[cfg(feature = "rand")]
    fn rand() -> Self;

    fn from_bytes(bytes: &[u8]) -> Result<Self, String>;

    fn from_bytes_unchecked(bytes: &[u8]) -> Result<Self, String> {
        Self::from_bytes(bytes)
    }

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

    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

pub trait G1: Clone + Default + PartialEq {
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

    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

pub trait G1Mul<TFr: Fr>: G1 + Clone {
    fn mul(&self, b: &TFr) -> Self;

    // Instead of creating separate trait, keep linear comb here for simplicity
    fn g1_lincomb(points: &[Self], scalars: &[TFr], len: usize) -> Self;
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

pub trait PairingVerify<TG1: G1, TG2: G2> {
    fn verify(a1: &TG1, a2: &TG2, b1: &TG1, b2: &TG2) -> bool;
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

    // Default implementation not as efficient, should be implemented by type itself
    fn from_coeffs(coeffs: &[Coeff]) -> Self {
        let mut poly = Self::new(coeffs.len());
        
        for i in 0..coeffs.len() {
            poly.set_coeff_at(i, &coeffs[i]);
        }

        poly
    }

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

    fn get_fft_settings(&self) -> &Fs;
    
    fn get_g1_secret(&self) -> &[Coeff2];

    fn get_g2_secret(&self) -> &[Coeff3];
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

pub fn poly_to_kzg_commitment<TFr: Fr, TG1: G1 + G1Mul<TFr>, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(p: &TPoly, s: &TKZGSettings) -> TG1 {
    TG1::g1_lincomb(&s.get_g1_secret(), &p.get_coeffs(), FIELD_ELEMENTS_PER_BLOB)
}

pub fn blob_to_kzg_commitment_rust<TFr: Fr, TG1: G1 + G1Mul<TFr>, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>> (
    blob: &[TFr],
    settings: &TKZGSettings,
) -> Result<TG1, String> {
    let polynomial = blob_to_polynomial(blob)?;

    Ok(poly_to_kzg_commitment(&polynomial, settings))
}

pub fn verify_kzg_proof_batch<TFr: Fr, TG1: G1 + G1Mul<TFr> + PairingVerify<TG1, TG2>, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(
    commitments_g1: &[TG1],
    zs_fr: &[TFr],
    ys_fr: &[TFr],
    proofs_g1: &[TG1],
    ts: &TKZGSettings,
) -> Result<bool, String> {
    let n = commitments_g1.len();
    let mut c_minus_y: Vec<TG1> = Vec::with_capacity(n);
    let mut r_times_z: Vec<TFr> = Vec::with_capacity(n);

    // Compute the random lincomb challenges
    let r_powers = compute_r_powers(commitments_g1, zs_fr, ys_fr, proofs_g1)?;

    // Compute \sum r^i * Proof_i
    let proof_lincomb = TG1::g1_lincomb(proofs_g1, &r_powers, n);

    for i in 0..n {
        // Get [y_i]
        let ys_encrypted = TG1::generator().mul(&ys_fr[i]);
        // Get C_i - [y_i]
        c_minus_y.push(commitments_g1[i].sub(&ys_encrypted));
        // Get r^i * z_i
        r_times_z.push(r_powers[i].mul(&zs_fr[i]));
    }

    // Get \sum r^i z_i Proof_i
    let proof_z_lincomb = TG1::g1_lincomb(proofs_g1, &r_times_z, n);
    // Get \sum r^i (C_i - [y_i])
    let mut c_minus_y_lincomb = TG1::g1_lincomb(&c_minus_y, &r_powers, n);

    // Get C_minus_y_lincomb + proof_z_lincomb
    let rhs_g1 = c_minus_y_lincomb.add_or_dbl(&proof_z_lincomb);

    // Do the pairing check!
    Ok(TG1::verify(
        &proof_lincomb,
        &ts.get_g2_secret()[1],
        &rhs_g1,
        &TG2::generator(),
    ))
}

pub fn compute_kzg_proof_rust<TFr: Fr + Copy, TG1: G1 + G1Mul<TFr>, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(
    blob: &[TFr],
    z: &TFr,
    s: &TKZGSettings,
) -> Result<(TG1, TFr), String> {
    if blob.len() != FIELD_ELEMENTS_PER_BLOB {
        return Err(String::from("Incorrect field elements count."));
    }

    let polynomial = blob_to_polynomial(blob)?;
    let y = evaluate_polynomial_in_evaluation_form(&polynomial, z, s)?;

    let mut tmp: TFr;
    // let roots_of_unity: &Vec<TFr> = &s.get_roots_of_unity_at().roots_of_unity;

    let mut m: usize = 0;
    let mut q: TPoly = TPoly::new(FIELD_ELEMENTS_PER_BLOB);

    let mut inverses_in: Vec<TFr> = vec![TFr::default(); FIELD_ELEMENTS_PER_BLOB];
    let mut inverses: Vec<TFr> = vec![TFr::default(); FIELD_ELEMENTS_PER_BLOB];

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        if z.equals(&s.get_roots_of_unity_at(i)) {
            // We are asked to compute a KZG proof inside the domain
            m = i + 1;
            inverses_in[i] = TFr::one();
            continue;
        }
        // (p_i - y) / (ω_i - z)
        q.set_coeff_at(i, &polynomial.get_coeff_at(i).sub(&y));
        inverses_in[i] = s.get_roots_of_unity_at(i).sub(z);
    }

    fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB)?;

    for (i, inverse) in inverses.iter().enumerate().take(FIELD_ELEMENTS_PER_BLOB) {
        q.set_coeff_at(i, &q.get_coeff_at(i).mul(inverse));
    }

    if m != 0 {
        // ω_{m-1} == z
        m -= 1;
        q.set_coeff_at(m, &TFr::zero());
        for i in 0..FIELD_ELEMENTS_PER_BLOB {
            if i == m {
                continue;
            }
            // Build denominator: z * (z - ω_i)
            tmp = z.sub(&s.get_roots_of_unity_at(i));
            inverses_in[i] = tmp.mul(z);
        }

        fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB)?;

        for i in 0..FIELD_ELEMENTS_PER_BLOB {
            if i == m {
                continue;
            }
            // Build numerator: ω_i * (p_i - y)
            tmp = polynomial.get_coeff_at(i).sub(&y);
            tmp = tmp.mul(&s.get_roots_of_unity_at(i));
            // Do the division: (p_i - y) * ω_i / (z * (z - ω_i))
            tmp = tmp.mul(&inverses[i]);
            q.set_coeff_at(m, &q.get_coeff_at(m).add(&tmp))
        }
    }

    let proof = TG1::g1_lincomb(&s.get_g1_secret(), &q.get_coeffs(), FIELD_ELEMENTS_PER_BLOB);
    Ok((proof, y))
}

pub fn compute_blob_kzg_proof_rust<TFr: Fr + Copy, TG1: G1 + G1Mul<TFr>, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(
    blob: &[TFr],
    commitment: &TG1,
    ts: &TKZGSettings,
) -> Result<TG1, String> {
    if !commitment.is_inf() && !commitment.is_valid() {
        return Err("Invalid commitment".to_string());
    }

    let evaluation_challenge_fr = compute_challenge(blob, commitment)?;
    let (proof, _) = compute_kzg_proof_rust::<_,_,_,_,_,_>(blob, &evaluation_challenge_fr, ts)?;
    Ok(proof)
}

pub fn verify_blob_kzg_proof_batch_rust<TFr: Fr + Copy, TG1: G1 + G1Mul<TFr> + PairingVerify<TG1, TG2>, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(
    blobs: &[Vec<TFr>],
    commitments_g1: &[TG1],
    proofs_g1: &[TG1],
    ts: &TKZGSettings,
) -> Result<bool, String> {
    // Exit early if we are given zero blobs
    if blobs.is_empty() {
        return Ok(true);
    }

    // For a single blob, just do a regular single verification
    if blobs.len() == 1 {
        return verify_blob_kzg_proof_rust(&blobs[0], &commitments_g1[0], &proofs_g1[0], ts);
    }

    if blobs.len() != commitments_g1.len() || blobs.len() != proofs_g1.len() {
        return Err("Invalid amount of arguments".to_string());
    }

    #[cfg(feature = "parallel")]
    {
        let num_blobs = blobs.len();
        let num_cores = num_cpus::get_physical();

        if num_blobs > num_cores {
            validate_batched_input(commitments_g1, proofs_g1)?;

            // Process blobs in parallel subgroups
            let blobs_per_group = num_blobs / num_cores;

            blobs
                .par_chunks(blobs_per_group)
                .enumerate()
                .map(|(i, blob_group)| {
                    let num_blobs_in_group = blob_group.len();
                    let commitment_group = &commitments_g1
                        [blobs_per_group * i..blobs_per_group * i + num_blobs_in_group];
                    let proof_group =
                        &proofs_g1[blobs_per_group * i..blobs_per_group * i + num_blobs_in_group];
                    let (evaluation_challenges_fr, ys_fr) =
                        compute_challenges_and_evaluate_polynomial(
                            blob_group,
                            commitment_group,
                            ts,
                        )?;

                    verify_kzg_proof_batch(
                        commitment_group,
                        &evaluation_challenges_fr,
                        &ys_fr,
                        proof_group,
                        ts,
                    )
                })
                .try_reduce(|| true, |a, b| Ok(a && b))
        } else {
            // Each group contains either one or zero blobs, so iterate
            // over the single blob verification function in parallel
            (blobs, commitments_g1, proofs_g1)
                .into_par_iter()
                .map(|(blob, commitment, proof)| {
                    verify_blob_kzg_proof_rust(blob, commitment, proof, ts)
                })
                .try_reduce(|| true, |a, b| Ok(a && b))
        }
    }

    #[cfg(not(feature = "parallel"))]
    Ok({
        validate_batched_input(commitments_g1, proofs_g1)?;
        let (evaluation_challenges_fr, ys_fr) =
            compute_challenges_and_evaluate_polynomial(blobs, commitments_g1, ts)?;

        verify_kzg_proof_batch(
            commitments_g1,
            &evaluation_challenges_fr,
            &ys_fr,
            proofs_g1,
            ts,
        )?
    })
}
