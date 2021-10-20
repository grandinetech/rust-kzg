//! this module contains an implementation of Kate-Zaverucha-Goldberg polynomial commitments

use super::poly::Poly;
use bls12_381::*;
use rand::Rng;
pub type blsScalar = bls12_381::Scalar;

/// KZG polinomial commitments on Bls12-381. This structure contains the trusted setup.
pub struct Kzg {
    pub pow_tau_g1: Vec<G1Projective>,
    pub pow_tau_g2: Vec<G2Projective>,
}

pub type Proof = G1Projective;
pub type Commitment = G1Projective;

impl Kzg {
    fn eval_at_tau_g1(&self, poly: &Poly) -> G1Projective {
        poly.0
            .iter()
            .enumerate()
            .fold(G1Projective::identity(), |acc, (n, k)| {
                acc + self.pow_tau_g1[n] * k
            })
    }

    fn eval_at_tau_g2(&self, poly: &Poly) -> G2Projective {
        poly.0
            .iter()
            .enumerate()
            .fold(G2Projective::identity(), |acc, (n, k)| {
                acc + self.pow_tau_g2[n] * k
            })
    }

    fn z_poly_of(points: &[(blsScalar, blsScalar)]) -> Poly {
        points.iter().fold(Poly::one(), |acc, (z, _y)| {
            &acc * &Poly::newFromCoeffs(vec![-z, blsScalar::one()])
        })
    }

    /// Generate the trusted setup. Is expected that this function is called
    ///   in a safe evironment what will be destroyed after its execution
    /// The `n` parameter is the maximum number of points that can be proved
    pub fn trusted_setup(n: usize) -> Self {
        let mut rng = rand::thread_rng();
        let rnd: [u64; 4] = [rng.gen(), rng.gen(), rng.gen(), rng.gen()];
        let tau = blsScalar::from_raw(rnd);

        let pow_tau_g1: Vec<G1Projective> = (0..n)
            .into_iter()
            .scan(blsScalar::one(), |acc, _| {
                let v = *acc;
                *acc *= tau;
                Some(v)
            })
            .map(|tau_pow| G1Affine::generator() * tau_pow)
            .collect();

        let pow_tau_g2: Vec<G2Projective> = (0..n)
            .into_iter()
            .scan(blsScalar::one(), |acc, _| {
                let v = *acc;
                *acc *= tau;
                Some(v)
            })
            .map(|tau_pow| G2Affine::generator() * tau_pow)
            .collect();

        Self {
            pow_tau_g1,
            pow_tau_g2,
        }
    }

    /// Generate a polinomial and its commitment from a `set` of points
    #[allow(non_snake_case)]
    pub fn poly_commitment_from_set(&self, set: &[(blsScalar, blsScalar)]) -> (Poly, Commitment) {
        let poly = Poly::lagrange(set);
        let commitment = self.eval_at_tau_g1(&poly);

        (poly, commitment)
    }

    /// Generates a proof that `points` exists in `set`
    #[allow(non_snake_case)]
    pub fn prove(&self, poly: &Poly, points: &[(blsScalar, blsScalar)]) -> Proof {
        // compute a lagrange poliomial I that have all the points to proof that are in the set
        // compute the polinomial Z that has roots (y=0) in all x's of I,
        //   so this is I=(x-x0)(x-x1)...(x-xn)
        let I = Poly::lagrange(points);
        let Z = Self::z_poly_of(points);

        // now compute that Q = ( P - I(x) ) / Z(x)
        // also check that the division does not have remainder
        let mut poly = poly.clone();
        poly -= &I;
        let (Q, remainder) = poly / Z;
        assert!(remainder.is_zero());

        // the proof is evaluating the Q at tau in G1
        self.eval_at_tau_g1(&Q)
    }

    /// Verifies that `points` exists in `proof`
    /// # Example
    /// ```
    /// use crate::kzg::{BlsScalar, Kzg};
    /// // Create a trustd setup that allows maximum 4 points (degree+1)
    /// let kzg = Kzg::trusted_setup(5);
    ///
    /// // define the set of points (the "population"), and create a polinomial
    /// // for them, as well its polinomial commitment, see the polinomial commitment
    /// // like the "hash" of the polinomial
    /// let set = vec![
    ///    (BlsScalar::from(1), BlsScalar::from(2)),
    ///    (BlsScalar::from(2), BlsScalar::from(3)),
    ///    (BlsScalar::from(3), BlsScalar::from(4)),
    ///    (BlsScalar::from(4), BlsScalar::from(57)),
    /// ];
    /// let (p, c) = kzg.poly_commitment_from_set(&set);
    ///
    /// // generate a proof that (1,2) and (2,3) are in the set
    /// let proof01 = kzg.prove(&p, &vec![set[0].clone(), set[1].clone()]);
    ///  
    /// // prove that (1,2) and (2,3) are in the set
    /// assert!(kzg.verify(&c, &vec![set[0].clone(), set[1].clone()], &proof01));
    /// // other proofs will fail since the proof only works for exactly (1,2) AND (2,3)
    /// assert!(!kzg.verify(&c, &vec![set[0].clone()], &proof01));
    /// assert!(!kzg.verify(&c, &vec![set[0].clone(), set[2].clone()], &proof01));
    ///
    /// // prove and verify that the whole set exists in the whole set
    /// let proof0123 = kzg.prove(&p, &set);
    /// assert!(kzg.verify(&c, &set, &proof0123));
    /// ```

    #[allow(non_snake_case)]
    pub fn verify(
        &self,
        commitment: &G1Projective,
        points: &[(blsScalar, blsScalar)],
        proof: &G1Projective,
    ) -> bool {
        let I = Poly::lagrange(points);
        let Z = Self::z_poly_of(points);

        let e1 = pairing(&proof.into(), &self.eval_at_tau_g2(&Z).into());

        let e2 = pairing(
            &(commitment - self.eval_at_tau_g1(&I)).into(),
            &G2Affine::generator(),
        );
        e1 == e2
    }
}