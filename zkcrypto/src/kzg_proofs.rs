use core::borrow::Borrow;
use crate::zkfr::blsScalar as Scalar;
use crate::poly::ZPoly as Poly;

use crate::kzg_types::ZkG1Projective as G1;
use crate::kzg_types::ZkG2Projective as G2;
use crate::kzg_types::ZkG1Affine as G1Affine;
use crate::kzg_types::ZkG2Affine as G2Affine;

use crate::curve::multiscalar_mul::msm_variable_base;
use crate::curve::pairings::G2Prepared;

use kzg::FFTSettings;
use crate::fftsettings::{ZkFFTSettings};

use rand_core::{CryptoRng, RngCore, OsRng};
use rand::Rng;
use ff::Field;

use kzg::{Poly as OtherPoly, Fr, FFTFr};
use std::ops::MulAssign;
use crate::kzg_types::pairings_verify;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Holds a commitment to a polynomial in a form of a [`G1Affine`]-bls12_381
/// point.
pub(crate) struct Commitment(
    /// The commitment is a group element.
    pub(crate) G1Affine,
);

impl From<G1Affine> for Commitment {
    fn from(point: G1Affine) -> Commitment {
        Commitment(point)
    }
}

impl From<G1> for Commitment {
    fn from(point: G1) -> Commitment {
        Commitment(point.into())
    }
}

/// CommitKey is used to commit to a polynomial which is bounded by the
/// max_degree.
#[derive(Debug, Clone, PartialEq)]
pub struct CommitKey {
    /// Group elements of the form `{ \beta^i G }`, where `i` ranges from 0 to
    /// `degree`.
    pub(crate) powers_of_g: Vec<G1Affine>,
}

impl CommitKey {

    pub(crate) fn max_degree(&self) -> usize {
        self.powers_of_g.len() - 1
    }

    fn check_commit_degree_is_within_bounds(&self, poly_degree: usize,) -> Result<(), String> {
        match (poly_degree == 0, poly_degree > self.max_degree()) {
            (true, _) => Err(String::from("cannot commit to polynomial of zero degree")),
            (false, true) => Err(String::from("proving key is not large enough to commit to said polynomial")),
            (false, false) => Ok(()),
        }
    }

    pub(crate) fn truncate(&self, mut truncated_degree: usize) -> Result<CommitKey, String> {
        match truncated_degree {
            // Check that the truncated degree is not zero
            0 => Err(String::from("cannot trim PublicParameters to a maximum size of zero")),
            // Check that max degree is less than truncated degree
            i if i > self.max_degree() => Err(String::from("cannot trim more than the maximum degree")),
            i => {
                if i == 1 {
                    truncated_degree += 1
                };
                let truncated_powers = Self {
                    powers_of_g: self.powers_of_g[..=truncated_degree].to_vec(),
                };
                Ok(truncated_powers)
            }
        }
    }
}

/// Opening Key is used to verify opening proofs made about a committed
/// polynomial.
#[derive(Clone, Debug)]
pub struct OpeningKey {
    /// The generator of G1.
    pub(crate) g: G1Affine,
    /// The generator of G2.
    pub(crate) h: G2Affine,
    /// \beta times the above generator of G2.
    pub(crate) beta_h: G2Affine,
    /// The generator of G2, prepared for use in pairings.
    pub(crate) prepared_h: G2Prepared,
    /// \beta times the above generator of G2, prepared for use in pairings.
    pub(crate) prepared_beta_h: G2Prepared,
}

impl OpeningKey {
    pub(crate) fn new(
        g: G1Affine,
        h: G2Affine,
        beta_h: G2Affine,
    ) -> OpeningKey {
        let prepared_h = G2Prepared::from(h);
        let prepared_beta_h = G2Prepared::from(beta_h);
        OpeningKey {
            g,
            h,
            beta_h,
            prepared_h,
            prepared_beta_h,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PublicParameters {
    /// Key used to generate proofs for composed circuits.
    pub(crate) commit_key: CommitKey,
    /// Key used to verify proofs for composed circuits.
    pub(crate) opening_key: OpeningKey,
}

impl PublicParameters {

    /// Setup generates the public parameters using a random number generator.
    /// This method will in most cases be used for testing and exploration.
    /// In reality, a `Trusted party` or a `Multiparty Computation` will be used
    /// to generate the SRS. Returns an error if the configured degree is less
    /// than one.
    pub fn setup<R: RngCore + CryptoRng>(max_degree: usize, mut rng: &mut R) -> Result<PublicParameters, String> {
        // Cannot commit to constants
        if max_degree < 1 {
            return Err(String::from("cannot create PublicParameters with max degree 0"));
        }

        // Generate the secret scalar beta
        let beta = random_scalar(&mut rng);

        // Compute powers of beta up to and including beta^max_degree
        let powers_of_beta = powers_of(&beta, max_degree);

        // Powers of G1 that will be used to commit to a specified polynomial
        let g = random_g1_point(&mut rng);
        let powers_of_g: Vec<G1> =
            slow_multiscalar_mul_single_base(&powers_of_beta, g);
        assert_eq!(powers_of_g.len(), max_degree + 1);

        // Normalise all projective points
        let mut normalised_g = vec![G1Affine::identity(); max_degree + 1];
        G1::batch_normalize(&powers_of_g, &mut normalised_g);

        // Compute beta*G2 element and stored cached elements for verifying
        // multiple proofs.
        let h: G2Affine = random_g2_point(&mut rng).into();
        let beta_h: G2Affine = (h * beta).into();

        Ok(PublicParameters {
            commit_key: CommitKey {
                powers_of_g: normalised_g,
            },
            opening_key: OpeningKey::new(g.into(), h, beta_h),
        })
    }

    pub fn trim(&self, truncated_degree: usize) -> Result<(CommitKey, OpeningKey), String> {
        let truncated_prover_key = self.commit_key.truncate(truncated_degree)?;
        let opening_key = self.opening_key.clone();
        Ok((truncated_prover_key, opening_key))
    }

    pub fn max_degree(&self) -> usize {
        self.commit_key.max_degree()
    }
}

#[derive(Copy, Clone, Debug)]
/// Proof that a polynomial `p` was correctly evaluated at a point `z`
/// producing the evaluated point p(z).
pub(crate) struct Proof {
    /// This is a commitment to the witness polynomial.
    pub(crate) commitment_to_witness: Commitment,
    /// This is the result of evaluating a polynomial at the point `z`.
    pub(crate) evaluated_point: Scalar,
    /// This is the commitment to the polynomial that you want to prove a
    /// statement about.
    pub(crate) commitment_to_polynomial: Commitment,
}

pub(crate) fn commit( commitkey: &CommitKey, polynomial: &Poly) -> Result<Commitment, String> {
    // Check whether we can safely commit to this polynomial
    commitkey.check_commit_degree_is_within_bounds(polynomial.degree())?;

    // Compute commitment
    Ok(Commitment::from(msm_variable_base(&commitkey.powers_of_g, &polynomial.coeffs)))
}

pub(crate) fn open_single(ck: &CommitKey, polynomial: &Poly, value: &Scalar, point: &Scalar) -> Result<Proof, String> {
    let witness_poly = compute_single_witness(polynomial, point);
    Ok(Proof {
        commitment_to_witness: commit(&ck, &witness_poly)?,
        evaluated_point: *value,
        commitment_to_polynomial: commit(&ck, polynomial)?,
    })
}

fn compute_single_witness(polynomial: &Poly, point: &Scalar) -> Poly {
    // Computes `f(x) / x-z`, returning it as the witness poly
    polynomial.ruffini(*point)
}

pub(crate) fn powers_of(scalar: &Scalar, max_degree: usize) -> Vec<Scalar> {
    let mut powers = Vec::with_capacity(max_degree + 1);
    powers.push(Scalar::one());
    for i in 1..=max_degree {
        powers.push(powers[i - 1] * scalar);
    }
    powers
}

pub(crate) fn random_scalar<R: RngCore + CryptoRng>(rng: &mut R) -> Scalar {
    Scalar::random(rng)
}

pub(crate) fn random_g1_point<R: RngCore + CryptoRng>(rng: &mut R) -> G1 {
    G1Affine::generator() * random_scalar(rng)
}

pub(crate) fn random_g2_point<R: RngCore + CryptoRng>(rng: &mut R) -> G2 {
    G2Affine::generator() * random_scalar(rng)
}

pub(crate) fn slow_multiscalar_mul_single_base(scalars: &[Scalar], base: G1) -> Vec<G1> {
    scalars.iter().map(|s| base * *s).collect()
}

pub(crate) fn check(op_key: &OpeningKey, point: Scalar, proof: Proof) -> bool {
    let inner_a: G1Affine = (proof.commitment_to_polynomial.0
        - (op_key.g * proof.evaluated_point))
        .into();

    let inner_b: G2Affine = (op_key.beta_h - (op_key.h * point)).into();
    let prepared_inner_b = G2Prepared::from(-inner_b);

    let pairing = crate::curve::pairings::multi_miller_loop(&[
        (&inner_a, &op_key.prepared_h),
        (&proof.commitment_to_witness.0, &prepared_inner_b),
    ])
    .final_exponentiation();

    pairing == crate::curve::pairings::Gt::identity()
}

// ========================== KZG Proofs implementation ===========================

pub struct KZGSettings {
    pub fs: ZkFFTSettings,
    pub secret_g1: Vec<G1>,
    pub secret_g2: Vec<G2>,
    pub length: u64,
    public_params: PublicParameters,
}

impl Default for KZGSettings {
    fn default() -> KZGSettings {
        KZGSettings {
            fs: ZkFFTSettings::default(),
            secret_g1: Vec::new(),
            secret_g2: Vec::new(),
            length: 0,
            public_params: PublicParameters::setup(1, &mut OsRng).unwrap(),
        }
    }
}

pub fn default_kzg() -> KZGSettings {
    KZGSettings {
            fs: ZkFFTSettings::default(),
            secret_g1: Vec::new(),
            secret_g2: Vec::new(),
            length: 0,
            public_params: PublicParameters::setup(1, &mut OsRng).unwrap(),
        }
}

// This code was taken from 'https://github.com/adria0/a0kzg/blob/main/src/kzg.rs' and adapted
pub fn generate_trusted_setup(n: usize, _secret: [u8; 32usize]) -> (Vec<G1>, Vec<G2>) {
    let mut rng = rand::thread_rng();
    let rnd: [u64; 4] = [rng.gen(), rng.gen(), rng.gen(), rng.gen()];
    let tau = Scalar::from_raw(rnd);

    let pow_tau_g1: Vec<G1> = (0..n)
        .into_iter()
        .scan(Scalar::one(), |acc, _| {
            let v = *acc;
            *acc *= tau;
            Some(v)
        })
        .map(|tau_pow| G1Affine::generator() * tau_pow)
        .collect();

    let pow_tau_g2: Vec<G2> = (0..n)
        .into_iter()
        .scan(Scalar::one(), |acc, _| {
            let v = *acc;
            *acc *= tau;
            Some(v)
        })
        .map(|tau_pow| G2Affine::generator() * tau_pow)
        .collect();

    (pow_tau_g1, pow_tau_g2)
}

pub(crate) fn new_kzg_settings(_secret_g1: Vec<G1>, _secret_g2: Vec<G2>, secrets_len: u64, _fs: &ZkFFTSettings) -> KZGSettings {
    KZGSettings {
        fs: _fs.borrow().clone(),
        secret_g1: _secret_g1,
        secret_g2: _secret_g2,
        length: secrets_len,
        public_params: PublicParameters::setup(secrets_len as usize, &mut OsRng).unwrap(),
    }
}

pub(crate) fn commit_to_poly(p: &Poly, ks: &KZGSettings) -> Result<G1, String> {
    if p.coeffs.len() > ks.secret_g1.len() {
        return Err(String::from("Poly given is too long"));
    } 

    else if p.is_zero() {
        Ok(G1::identity())
    }

    else {
        let (powers, _) = ks.public_params.trim(ks.public_params.max_degree() - 1).unwrap();
        let com = commit(&powers, &p).unwrap();
        Ok(G1::from(com.0))
    }
}

pub(crate) fn compute_proof_single(p: &Poly, fr: &Scalar, ks: &KZGSettings) -> G1 {
    let (powers, _) = ks.public_params.trim(ks.public_params.max_degree() - 1).unwrap();

    let proof = open_single(&powers, &p, &random_scalar(&mut OsRng), &fr).unwrap();

    G1::from(proof.commitment_to_witness.0)
}

pub(crate) fn eval_poly(poly: &Poly, point: &Scalar) -> Scalar {
    if poly.is_zero() {
        return Scalar::zero();
    }

    // Compute powers of points
    let powers = powers_of(point, poly.coeffs.len());

    let p_evals: Vec<_> = poly.coeffs
        .iter()
        .zip(powers.into_iter())
        .map(|(c, p)| p * c)
        .collect();
    let mut sum = Scalar::zero();
    for eval in p_evals.into_iter() {
        sum += &eval;
    }
    sum
}

pub(crate) fn check_proof_single(com: &G1, proof: &G1, x: &Scalar, value: &Scalar, ks: &KZGSettings) -> bool {
    let (_powers, op_key) = ks.public_params.trim(ks.public_params.max_degree() - 1).unwrap();

    let proof2 = Proof{
        commitment_to_witness: Commitment::from(G1Affine::from(proof)), 
        evaluated_point: *value, 
        commitment_to_polynomial: Commitment::from(*com)};

    check(&op_key, *x, proof2)
}

pub(crate) fn compute_proof_multi(p: &Poly, x: &Scalar, n: usize, ks: &KZGSettings) -> G1 {
    let mut divisor = Poly::new(n+1).unwrap();
    let x_pow_n = <Scalar as Fr>::pow(&x, n);

    divisor.set_coeff_at(0, &x_pow_n.negate());

    for i in 1..n {
        divisor.set_coeff_at(i, &Scalar::zero());
    }
    divisor.set_coeff_at(n, &Scalar::one());

    let mut p = p.clone();

    let q = p.div(&divisor).unwrap();

    commit_to_poly(&q, &ks).unwrap()
}

pub(crate) fn check_proof_multi(com: &G1, proof: &G1, x: &Scalar, ys: &Vec<Scalar>, n: usize, ks: &KZGSettings) -> bool {
    let mut interp = Poly::new(n).unwrap();

    interp.coeffs = ks.fs.fft_fr(ys, true).unwrap();

    let inv_x = x.inverse();
    let mut inv_x_pow = inv_x.clone();
    for i in 1..n {
        interp.coeffs[i] = interp.coeffs[i].mul(&inv_x_pow);
        inv_x_pow = inv_x_pow.mul(&inv_x);
    }

    let x_pow = inv_x_pow.inverse();
    let mut xn2 = G2::from(ks.public_params.opening_key.h);

    xn2.mul_assign(&x_pow);

    let xn_minus_yn = &ks.secret_g2[n] - xn2;

    let is1 = &commit_to_poly(&interp, ks).unwrap();

    let commit_minus_interp = com - is1;

    pairings_verify(&commit_minus_interp, &G2::from(ks.public_params.opening_key.h), &proof, &xn_minus_yn)
}
