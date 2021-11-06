use crate::zkfr::blsScalar as Scalar;
// use crate::fftsettings::{ZkFFTSettings};
use crate::kzg_types::ZkG1Projective as G1;
// use crate::kzg_types::ZkG2Projective as G2;
use crate::kzg_types::ZkG1Affine as G1Affine;
// use crate::kzg_types::ZkG2Affine as G2Affine;
// use crate::curve::fp::Fp;
use crate::poly::ZPoly as Poly;
use crate::curve::multiscalar_mul::msm_variable_base;
// use rand::Rng;

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
}

pub(crate) fn commit_to_poly( commitkey: &CommitKey, polynomial: &Poly) -> Result<Commitment, String> {
    // Check whether we can safely commit to this polynomial
    commitkey.check_commit_degree_is_within_bounds(polynomial.degree())?;

    // Compute commitment
    Ok(Commitment::from(msm_variable_base(&commitkey.powers_of_g, &polynomial.coeffs)))
}

pub(crate) fn compute_proof_single(ck: &CommitKey, polynomial: &Poly, value: &Scalar, point: &Scalar) -> Result<Proof, String> {
    let witness_poly = compute_single_witness(polynomial, point);
    Ok(Proof {
        commitment_to_witness: commit_to_poly(&ck, &witness_poly)?,
        evaluated_point: *value,
        commitment_to_polynomial: commit_to_poly(&ck, polynomial)?,
    })
}

fn compute_single_witness(polynomial: &Poly, point: &Scalar) -> Poly {
    // Computes `f(x) / x-z`, returning it as the witness poly
    polynomial.ruffini(*point)
}