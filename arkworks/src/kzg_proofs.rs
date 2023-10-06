#![allow(non_camel_case_types)]

use rand::SeedableRng;
use std::borrow::Borrow;

use ark_ec::VariableBaseMSM;
use ark_ec::pairing::Pairing;
use ark_ec::scalar_mul::fixed_base::FixedBase;
use ark_poly::GeneralEvaluationDomain;
// use ark_poly_commit::kzg10::{
//     Commitment, Powers, Proof, Randomness, UniversalParams, VerifierKey, KZG10,
// };
use ark_poly_commit::*;
use ark_std::{
    marker::PhantomData, ops::Div, rand::RngCore, test_rng, vec, One, UniformRand, Zero,
};

use kzg::cfg_into_iter;
// FIXME: parallel needs these uses
// #[cfg(feature = "parallel")]
// {
// use rayon::iter::IntoParallelIterator;
// use rayon::iter::ParallelIterator;
// }

use super::utils::{
    blst_fr_into_pc_fr, blst_p1_into_pc_g1projective,
    blst_poly_into_pc_poly,
    PolyData,
};
use crate::consts::{G1_GENERATOR, G1_IDENTITY, G2_GENERATOR};
use crate::kzg_types::{ArkG1, ArkG2, ArkFr as BlstFr};
use crate::utils::blst_p2_into_pc_g2projective;
use ark_bls12_381::{g1, g2, Bls12_381, Fr};
use ark_ec::CurveGroup;
use ark_ec::{
    models::short_weierstrass::Affine,
    models::short_weierstrass::Projective,
    // ProjectiveCurve,
};
use ark_ff::{BigInteger256, PrimeField};
use ark_poly::univariate::DensePolynomial as DensePoly;
use kzg::{FFTFr, Fr as FrTrait, Poly};
use rand::rngs::StdRng;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::ops::{MulAssign, Neg};

use ark_ec::AffineRepr;
use std::ops::Mul;

pub type UniPoly_381 = DensePoly<<Bls12_381 as Pairing>::ScalarField>;
// type KZG_Bls12_381 = KZG10<Bls12_381, UniPoly_381>;

/*This segment has been copied from https://github.com/arkworks-rs/poly-commit/blob/master/src/kzg10/mod.rs,
Due to being private and, therefore, unreachable*/
// fn trim(
//     pp: &UniversalParams<Bls12_381>,
//     mut supported_degree: usize,
// ) -> Result<(Powers<Bls12_381>, VerifierKey<Bls12_381>), Error> {
//     if supported_degree == 1 {
//         supported_degree += 1;
//     }
//     let powers_of_g = pp.powers_of_g[..=supported_degree].to_vec();
//     let powers_of_gamma_g = (0..=supported_degree)
//         .map(|i| pp.powers_of_gamma_g[&i])
//         .collect();

//     let powers = Powers {
//         powers_of_g: ark_std::borrow::Cow::Owned(powers_of_g),
//         powers_of_gamma_g: ark_std::borrow::Cow::Owned(powers_of_gamma_g),
//     };
//     let vk = VerifierKey {
//         g: pp.powers_of_g[0],
//         gamma_g: pp.powers_of_gamma_g[&0],
//         h: pp.h,
//         beta_h: pp.beta_h,
//         prepared_h: pp.prepared_h.clone(),
//         prepared_beta_h: pp.prepared_beta_h.clone(),
//     };
//     Ok((powers, vk))
// }

#[allow(clippy::upper_case_acronyms)]
pub struct KZG<E: Pairing, P: DenseUVPolynomial<E::ScalarField>> {
    _engine: PhantomData<E>,
    _poly: PhantomData<P>,
}

pub struct setup_type {
    // pub params: UniversalParams<Bls12_381>,
    pub g1_secret: Vec<Projective<g1::Config>>,
    pub g2_secret: Vec<Projective<g2::Config>>,
}

// /*This segment has been copied from https://github.com/arkworks-rs/poly-commit/blob/master/src/kzg10/mod.rs,
// Due to being private and, therefore, unreachable and/or in need of modification*/
// impl<E, P> KZG<E, P>
// where
//     E: Pairing,
//     P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
//     for<'a, 'b> &'a P: Div<&'b P, Output = P>,
// {
//     #![allow(non_camel_case_types)]
//     /// Constructs public parameters when given as input the maximum degree `degree`
//     /// for the polynomial commitment scheme.
//     pub fn setup<R: RngCore>(
//         max_degree: usize,
//         produce_g2_powers: bool,
//         rng: &mut R,
//     ) -> Result<setup_type, Error> {
//         if max_degree < 1 {
//             return Err(Error::DegreeIsZero);
//         }

//         let beta = Fr::rand(rng);
//         let g = blst_p1_into_pc_g1projective(&G1_GENERATOR.0).unwrap();
//         let gamma_g: Projective<g1::Config> = Projective::rand(rng);
//         let h = blst_p2_into_pc_g2projective(&G2_GENERATOR).unwrap();

//         let mut powers_of_beta = vec![Fr::one()];

//         let mut cur = beta;
//         for _ in 0..max_degree {
//             powers_of_beta.push(cur);
//             cur *= &beta;
//         }

//         let window_size = FixedBase::get_mul_window_size(max_degree + 1);

//         let scalar_bits: usize = Fr::MODULUS_BIT_SIZE.try_into().unwrap();
//         let g_table = FixedBase::get_window_table(scalar_bits, window_size, g);
//         let powers_of_g = FixedBase::msm::<Projective<g1::Config>>(
//             scalar_bits,
//             window_size,
//             &g_table,
//             &powers_of_beta,
//         );

//         let gamma_g_table = FixedBase::get_window_table(scalar_bits, window_size, gamma_g);
//         let mut powers_of_gamma_g = FixedBase::msm::<Projective<g1::Config>>(
//             scalar_bits,
//             window_size,
//             &gamma_g_table,
//             &powers_of_beta,
//         );
//         // Add an additional power of gamma_g, because we want to be able to support
//         // up to D queries.
//         powers_of_gamma_g.push(powers_of_gamma_g.last().unwrap().mul(beta));

//         let powers_of_g = Projective::normalize_batch(&powers_of_g);
//         let powers_of_gamma_g =
//             Projective::normalize_batch(&powers_of_gamma_g)
//                 .into_iter()
//                 .enumerate()
//                 .collect();

//         let neg_powers_of_h = if produce_g2_powers {
//             let mut neg_powers_of_beta = vec![Fr::one()];
//             let mut cur = Fr::one() / beta;
//             for _ in 0..max_degree {
//                 neg_powers_of_beta.push(cur);
//                 cur /= &beta;
//             }

//             let neg_h_table = FixedBase::get_window_table(scalar_bits, window_size, h);
//             let neg_powers_of_h = FixedBase::msm::<Projective<g2::Config>>(
//                 scalar_bits,
//                 window_size,
//                 &neg_h_table,
//                 &neg_powers_of_beta,
//             );

//             let affines = Projective::normalize_batch(&neg_powers_of_h);
//             let mut affines_map = BTreeMap::new();
//             affines.into_iter().enumerate().for_each(|(i, a)| {
//                 affines_map.insert(i, a);
//             });
//             affines_map
//         } else {
//             BTreeMap::new()
//         };

//         let h = h.into_affine();
//         let beta_h = h.mul(beta).into_affine();
//         let prepared_h = h.into();
//         let prepared_beta_h = beta_h.into();

//         let (s1, s2) = generate_trusted_setup_test(max_degree, beta);

//         let pp = UniversalParams {
//             powers_of_g,
//             powers_of_gamma_g,
//             h,
//             beta_h,
//             neg_powers_of_h,
//             prepared_h,
//             prepared_beta_h,
//         };
//         let res = setup_type {
//             // params: pp,
//             g1_secret: s1,
//             g2_secret: s2,
//         };
//         Ok(res)
//     }

//     fn open(
//         powers: &Powers<Bls12_381>,
//         p: &DensePoly<Fr>,
//         point: Fr,
//         rand: &Randomness<Fr, DensePoly<Fr>>,
//     ) -> Result<Proof<Bls12_381>, Error> {
//         Self::check_degree_is_too_large(p.degree(), powers.size())?;

//         let (witness_poly, hiding_witness_poly) =
//             KZG_Bls12_381::compute_witness_polynomial(p, point, rand)?;

//         let proof = Self::open_with_witness_polynomial(
//             powers,
//             point,
//             rand,
//             &witness_poly,
//             hiding_witness_poly.as_ref(),
//         );

//         proof
//     }

//     fn open_with_witness_polynomial(
//         powers: &Powers<Bls12_381>,
//         point: Fr,
//         randomness: &Randomness<Fr, DensePoly<Fr>>,
//         witness_polynomial: &DensePoly<Fr>,
//         hiding_witness_polynomial: Option<&DensePoly<Fr>>,
//     ) -> Result<Proof<Bls12_381>, Error> {
//         Self::check_degree_is_too_large(witness_polynomial.degree(), powers.size())?;
//         let (num_leading_zeros, witness_coeffs) =
//             skip_leading_zeros_and_convert_to_bigints(witness_polynomial);

//         let ark_scalars: Vec<BigInteger256> = {
//             cfg_into_iter!(witness_coeffs)
//                 .map(|scalar| scalar.into())
//                 .collect()
//         };
//         let w: Projective<g1::Config> = VariableBaseMSM::msm_bigint(
//             &powers.powers_of_g[num_leading_zeros..],
//             &ark_scalars,
//         );

//         let random_v = if let Some(_hiding_witness_polynomial) = hiding_witness_polynomial {
//             let blinding_p = &randomness.blinding_polynomial;
//             let blinding_evaluation = blinding_p.evaluate(&point);

//             Some(blinding_evaluation)
//         } else {
//             None
//         };

//         Ok(Proof {
//             w: Into::into(w),
//             random_v,
//         })
//     }

//     fn check_degree_is_too_large(degree: usize, num_powers: usize) -> Result<(), Error> {
//         let num_coefficients = degree + 1;
//         if num_coefficients > num_powers {
//             Err(Error::TooManyCoefficients {
//                 num_coefficients,
//                 num_powers,
//             })
//         } else {
//             Ok(())
//         }
//     }
// }

fn skip_leading_zeros_and_convert_to_bigints<F: PrimeField, P: DenseUVPolynomial<F>>(
    p: &P,
) -> (usize, Vec<F::BigInt>) {
    let mut num_leading_zeros = 0;
    while num_leading_zeros < p.coeffs().len() && p.coeffs()[num_leading_zeros].is_zero() {
        num_leading_zeros += 1;
    }
    let coeffs = convert_to_bigints(&p.coeffs()[num_leading_zeros..]);
    (num_leading_zeros, coeffs)
}

fn convert_to_bigints<F: PrimeField>(p: &[F]) -> Vec<F::BigInt> {
    let coeffs = p.iter().map(|s| (*s).into()).collect::<Vec<_>>();
    coeffs
}

#[derive(Debug, Clone)]
pub struct FFTSettings {
    pub max_width: usize,
    pub root_of_unity: BlstFr,
    pub expanded_roots_of_unity: Vec<BlstFr>,
    pub reverse_roots_of_unity: Vec<BlstFr>,
    pub roots_of_unity: Vec<BlstFr>,
    pub domain: GeneralEvaluationDomain<Fr>,
}

pub fn expand_root_of_unity(root: &BlstFr, width: usize) -> Result<Vec<BlstFr>, String> {
    let mut generated_powers = vec![BlstFr::one(), *root];

    while !(generated_powers.last().unwrap().is_one()) {
        if generated_powers.len() > width {
            return Err(String::from("Root of unity multiplied for too long"));
        }

        generated_powers.push(generated_powers.last().unwrap().mul(root));
    }

    if generated_powers.len() != width + 1 {
        return Err(String::from("Root of unity has invalid scale"));
    }

    Ok(generated_powers)
}

#[derive(Debug, Clone)]
pub struct KZGSettings {
    pub fs: FFTSettings,
    pub secret_g1: Vec<ArkG1>,
    pub secret_g2: Vec<ArkG2>,
}

impl Default for KZGSettings {
    fn default() -> Self {
        Self {
            fs: FFTSettings::default(),
            secret_g1: Vec::new(),
            secret_g2: Vec::new(),
        }
    }
}

fn read_be_u64(input: &mut &[u8]) -> u64 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u64>());
    *input = rest;
    u64::from_be_bytes(int_bytes.try_into().unwrap())
}

pub fn generate_trusted_setup(len: usize, secret: [u8; 32usize]) -> (Vec<ArkG1>, Vec<ArkG2>) {
    let mut s_pow = Fr::from(1);
    let mut temp = vec![0; 4];
    for i in 0..4 {
        temp[i] = read_be_u64(&mut &secret[i * 8..(i + 1) * 8]);
    }
    let s = Fr::new_unchecked(BigInteger256::new([temp[0], temp[1], temp[2], temp[3]]));
    let mut s1 = Vec::new();
    let mut s2 = Vec::new();
    for _i in 0..len {
        let mut temp =
            g1::G1Affine::new_unchecked(g1::G1_GENERATOR_X, g1::G1_GENERATOR_Y).into_group();
        temp.mul_assign(s_pow);
        s1.push(ArkG1{proj: temp});
        let mut temp =
            g2::G2Affine::new_unchecked(g2::G2_GENERATOR_X, g2::G2_GENERATOR_Y).into_group();
        temp.mul_assign(s_pow);
        s2.push(ArkG2{proj: temp});
        s_pow *= s;
    }
    (s1, s2)
}

fn generate_trusted_setup_test(
    len: usize,
    s: Fr,
) -> (
    Vec<Projective<g1::Config>>,
    Vec<Projective<g2::Config>>,
) {
    let mut s_pow = Fr::from(1);
    let mut s1 = Vec::new();
    let mut s2 = Vec::new();
    for _i in 0..len {
        let mut temp = G1_GENERATOR.proj;
        temp.mul_assign(s_pow);
        s1.push(temp);
        let mut temp = G2_GENERATOR.proj;
        temp.mul_assign(s_pow);
        s2.push(temp);
        s_pow *= s;
    }

    (s1, s2)
}


pub fn eval_poly(p: &PolyData, x: &BlstFr) -> BlstFr {
    let poly = blst_poly_into_pc_poly(&p.coeffs);
    BlstFr { fr: poly.evaluate(&x.fr) }
}

pub fn pairings_verify(a1: &ArkG1, a2: &ArkG2, b1: &ArkG1, b2: &ArkG2) -> bool {
    let ark_a1_neg = a1.proj
        .neg()
        .into_affine();
    let ark_b1 = b1.proj.into_affine();
    let ark_a2 = a2.proj.into_affine();
    let ark_b2 = b2.proj.into_affine();
        
    Bls12_381::multi_pairing([ark_a1_neg, ark_b1], [ark_a2, ark_b2])
    .0.is_one()
}
