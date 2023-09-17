#![allow(non_camel_case_types)]

use rand::SeedableRng;
use std::borrow::Borrow;

use ark_ec::VariableBaseMSM;
use ark_ec::pairing::Pairing;
use ark_ec::scalar_mul::fixed_base::FixedBase;
use ark_poly::Radix2EvaluationDomain;
use ark_poly_commit::kzg10::{
    Commitment, Powers, Proof, Randomness, UniversalParams, VerifierKey, KZG10,
};
use ark_poly_commit::*;
use ark_std::{
    marker::PhantomData, ops::Div, rand::RngCore, test_rng, vec, One, UniformRand, Zero,
};

use kzg::{cfg_into_iter, G1Mul};

use super::utils::{
    blst_fr_into_pc_fr, blst_p1_into_pc_g1projective, blst_p2_into_pc_g2projective,
    blst_poly_into_pc_poly, pc_fr_into_blst_fr, pc_g1projective_into_blst_p1,
    pc_g2projective_into_blst_p2, PolyData,
};
use super::P2;
use crate::fft_g1::{G1_GENERATOR, G1_IDENTITY};
use crate::kzg_types::{ArkG1, ArkG2, FsFr as BlstFr};
use ark_bls12_381::{g1, g2, Bls12_381, Fr as ArkFr, Fr};
use ark_ec::CurveGroup;
use ark_ec::{
    models::short_weierstrass::Affine,
    models::short_weierstrass::Projective,
    // ProjectiveCurve,
};
use ark_ff::{BigInteger256, PrimeField};
use ark_poly::univariate::DensePolynomial as DensePoly;
use blst::{blst_fp, blst_fp2};
use kzg::{FFTFr, Fr as FrTrait, Poly};
use rand::rngs::StdRng;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::ops::{MulAssign, Neg};

use ark_ec::AffineRepr;
use std::ops::Mul;

pub type UniPoly_381 = DensePoly<<Bls12_381 as Pairing>::ScalarField>;
type KZG_Bls12_381 = KZG10<Bls12_381, UniPoly_381>;

/*This segment has been copied from https://github.com/arkworks-rs/poly-commit/blob/master/src/kzg10/mod.rs,
Due to being private and, therefore, unreachable*/
fn trim(
    pp: &UniversalParams<Bls12_381>,
    mut supported_degree: usize,
) -> Result<(Powers<Bls12_381>, VerifierKey<Bls12_381>), Error> {
    if supported_degree == 1 {
        supported_degree += 1;
    }
    let powers_of_g = pp.powers_of_g[..=supported_degree].to_vec();
    let powers_of_gamma_g = (0..=supported_degree)
        .map(|i| pp.powers_of_gamma_g[&i])
        .collect();

    let powers = Powers {
        powers_of_g: ark_std::borrow::Cow::Owned(powers_of_g),
        powers_of_gamma_g: ark_std::borrow::Cow::Owned(powers_of_gamma_g),
    };
    let vk = VerifierKey {
        g: pp.powers_of_g[0],
        gamma_g: pp.powers_of_gamma_g[&0],
        h: pp.h,
        beta_h: pp.beta_h,
        prepared_h: pp.prepared_h.clone(),
        prepared_beta_h: pp.prepared_beta_h.clone(),
    };
    Ok((powers, vk))
}

#[allow(clippy::upper_case_acronyms)]
pub struct KZG<E: Pairing, P: DenseUVPolynomial<E::ScalarField>> {
    _engine: PhantomData<E>,
    _poly: PhantomData<P>,
}

pub struct setup_type {
    pub params: UniversalParams<Bls12_381>,
    pub g1_secret: Vec<Projective<g1::Config>>,
    pub g2_secret: Vec<Projective<g2::Config>>,
}

/*This segment has been copied from https://github.com/arkworks-rs/poly-commit/blob/master/src/kzg10/mod.rs,
Due to being private and, therefore, unreachable and/or in need of modification*/
impl<E, P> KZG<E, P>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    #![allow(non_camel_case_types)]
    /// Constructs public parameters when given as input the maximum degree `degree`
    /// for the polynomial commitment scheme.
    pub fn setup<R: RngCore>(
        max_degree: usize,
        produce_g2_powers: bool,
        rng: &mut R,
    ) -> Result<setup_type, Error> {
        if max_degree < 1 {
            return Err(Error::DegreeIsZero);
        }

        let beta = Fr::rand(rng);
        let g = blst_p1_into_pc_g1projective(&G1_GENERATOR).unwrap();
        let gamma_g: Projective<g1::Config> = Projective::rand(rng);
        let h = blst_p2_into_pc_g2projective(&G2_GENERATOR).unwrap();

        let mut powers_of_beta = vec![Fr::one()];

        let mut cur = beta;
        for _ in 0..max_degree {
            powers_of_beta.push(cur);
            cur *= &beta;
        }

        let window_size = FixedBase::get_mul_window_size(max_degree + 1);

        let scalar_bits: usize = Fr::MODULUS_BIT_SIZE.try_into().unwrap();
        let g_table = FixedBase::get_window_table(scalar_bits, window_size, g);
        let powers_of_g = FixedBase::msm::<Projective<g1::Config>>(
            scalar_bits,
            window_size,
            &g_table,
            &powers_of_beta,
        );

        let gamma_g_table = FixedBase::get_window_table(scalar_bits, window_size, gamma_g);
        let mut powers_of_gamma_g = FixedBase::msm::<Projective<g1::Config>>(
            scalar_bits,
            window_size,
            &gamma_g_table,
            &powers_of_beta,
        );
        // Add an additional power of gamma_g, because we want to be able to support
        // up to D queries.
        powers_of_gamma_g.push(powers_of_gamma_g.last().unwrap().mul(beta));

        let powers_of_g = Projective::normalize_batch(&powers_of_g);
        let powers_of_gamma_g =
            Projective::normalize_batch(&powers_of_gamma_g)
                .into_iter()
                .enumerate()
                .collect();

        let neg_powers_of_h = if produce_g2_powers {
            let mut neg_powers_of_beta = vec![Fr::one()];
            let mut cur = Fr::one() / beta;
            for _ in 0..max_degree {
                neg_powers_of_beta.push(cur);
                cur /= &beta;
            }

            let neg_h_table = FixedBase::get_window_table(scalar_bits, window_size, h);
            let neg_powers_of_h = FixedBase::msm::<Projective<g2::Config>>(
                scalar_bits,
                window_size,
                &neg_h_table,
                &neg_powers_of_beta,
            );

            let affines = Projective::normalize_batch(&neg_powers_of_h);
            let mut affines_map = BTreeMap::new();
            affines.into_iter().enumerate().for_each(|(i, a)| {
                affines_map.insert(i, a);
            });
            affines_map
        } else {
            BTreeMap::new()
        };

        let h = h.into_affine();
        let beta_h = h.mul(beta).into_affine();
        let prepared_h = h.into();
        let prepared_beta_h = beta_h.into();

        let (s1, s2) = generate_trusted_setup_test(max_degree, beta);

        let pp = UniversalParams {
            powers_of_g,
            powers_of_gamma_g,
            h,
            beta_h,
            neg_powers_of_h,
            prepared_h,
            prepared_beta_h,
        };
        let res = setup_type {
            params: pp,
            g1_secret: s1,
            g2_secret: s2,
        };
        Ok(res)
    }

    fn open(
        powers: &Powers<Bls12_381>,
        p: &DensePoly<Fr>,
        point: Fr,
        rand: &Randomness<Fr, DensePoly<Fr>>,
    ) -> Result<Proof<Bls12_381>, Error> {
        Self::check_degree_is_too_large(p.degree(), powers.size())?;

        let (witness_poly, hiding_witness_poly) =
            KZG_Bls12_381::compute_witness_polynomial(p, point, rand)?;

        let proof = Self::open_with_witness_polynomial(
            powers,
            point,
            rand,
            &witness_poly,
            hiding_witness_poly.as_ref(),
        );

        proof
    }

    fn open_with_witness_polynomial(
        powers: &Powers<Bls12_381>,
        point: Fr,
        randomness: &Randomness<Fr, DensePoly<Fr>>,
        witness_polynomial: &DensePoly<Fr>,
        hiding_witness_polynomial: Option<&DensePoly<Fr>>,
    ) -> Result<Proof<Bls12_381>, Error> {
        Self::check_degree_is_too_large(witness_polynomial.degree(), powers.size())?;
        let (num_leading_zeros, witness_coeffs) =
            skip_leading_zeros_and_convert_to_bigints(witness_polynomial);

        let ark_scalars: Vec<BigInteger256> = {
            cfg_into_iter!(witness_coeffs)
                .map(|scalar| scalar.into())
                .collect()
        };
        let w: Projective<g1::Config> = VariableBaseMSM::msm_bigint(
            &powers.powers_of_g[num_leading_zeros..],
            &ark_scalars,
        );

        let random_v = if let Some(_hiding_witness_polynomial) = hiding_witness_polynomial {
            let blinding_p = &randomness.blinding_polynomial;
            let blinding_evaluation = blinding_p.evaluate(&point);

            Some(blinding_evaluation)
        } else {
            None
        };

        Ok(Proof {
            w: Into::into(w),
            random_v,
        })
    }

    fn check_degree_is_too_large(degree: usize, num_powers: usize) -> Result<(), Error> {
        let num_coefficients = degree + 1;
        if num_coefficients > num_powers {
            Err(Error::TooManyCoefficients {
                num_coefficients,
                num_powers,
            })
        } else {
            Ok(())
        }
    }
}

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
    pub domain: Radix2EvaluationDomain<Fr>,
}

pub fn expand_root_of_unity(root: &BlstFr, width: usize) -> Result<Vec<BlstFr>, String> {
    let mut generated_powers = vec![BlstFr::one(), *root];

    while !(generated_powers.last().unwrap().is_one()) {
        if generated_powers.len() > width {
            return Err(String::from("Root of unity multiplied for too long"));
        }

        generated_powers.push(generated_powers.last().unwrap().mul(root));
    }

    Ok(generated_powers)
}

#[derive(Debug, Clone)]
pub struct KZGSettings {
    pub fs: FFTSettings,
    pub secret_g1: Vec<ArkG1>,
    pub secret_g2: Vec<ArkG2>,
    pub length: u64,
    pub params: UniversalParams<Bls12_381>,
    pub rand: StdRng,
    pub rand2: Randomness<Fr, UniPoly_381>,
}

impl Default for KZGSettings {
    fn default() -> Self {
        Self {
            fs: FFTSettings::default(),
            secret_g1: Vec::new(),
            secret_g2: Vec::new(),
            length: 0,
            params: KZG_Bls12_381::setup(1, false, &mut test_rng()).unwrap(),
            rand: StdRng::seed_from_u64(0), // This is def wrong, just using rn
            rand2: Randomness::empty(),
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
    /* Might be wrong */
    let s = Fr::from(BigInteger256::new([temp[0], temp[1], temp[2], temp[3]]));
    let mut s1 = Vec::new();
    let mut s2 = Vec::new();
    for _i in 0..len {
        let mut temp =
            g1::G1Affine::new(g1::G1_GENERATOR_X, g1::G1_GENERATOR_Y).into_group();
        temp.mul_assign(s_pow);
        s1.push(pc_g1projective_into_blst_p1(temp).unwrap());
        let mut temp =
            g2::G2Affine::new(g2::G2_GENERATOR_X, g2::G2_GENERATOR_Y).into_group();
        temp.mul_assign(s_pow);
        s2.push(pc_g2projective_into_blst_p2(temp).unwrap());
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
        let mut temp = blst_p1_into_pc_g1projective(&G1_GENERATOR).unwrap();
        temp.mul_assign(s_pow);
        s1.push(temp);
        let mut temp = blst_p2_into_pc_g2projective(&G2_GENERATOR).unwrap();
        temp.mul_assign(s_pow);
        s2.push(temp);
        s_pow *= s;
    }

    (s1, s2)
}

pub fn generate_rng_seed(secret_g1: &[ArkG1]) -> StdRng {
    let mut output = Vec::<u8>::new();
    for val in &secret_g1[secret_g1.len() - 1].0.x.l {
        output.extend_from_slice(&val.to_be_bytes());
    }
    StdRng::from_seed(output[..32].try_into().unwrap())
}

pub fn new_kzg_settings(
    secret_g1: &[ArkG1],
    _secret_g2: &[ArkG2],
    length: u64,
    ffs: &FFTSettings,
) -> KZGSettings {
    let length = length + 1;
    let mut rng = generate_rng_seed(secret_g1);
    let mut setup = KZG::<Bls12_381, UniPoly_381>::setup(length as usize, false, &mut rng).unwrap();

    let mut temp = Vec::new();
    let mut temp2 = Vec::new();
    let mut temp3 = Vec::new();

    for i in 0..length {
        temp.push(pc_g1projective_into_blst_p1(setup.g1_secret[i as usize]).unwrap());
        temp2.push(pc_g2projective_into_blst_p2(setup.g2_secret[i as usize]).unwrap());
        temp3.push(setup.g1_secret[i as usize].into_affine());
    }

    setup.params.powers_of_g = temp3;

    KZGSettings {
        secret_g1: temp,
        secret_g2: temp2,
        length,
        params: setup.params,
        fs: ffs.borrow().clone(),
        ..KZGSettings::default()
    }
}

pub fn commit_to_poly(p: &PolyData, ks: &KZGSettings) -> Result<ArkG1, String> {
    if p.coeffs.len() > ks.length as usize {
        Err(String::from("Poly given is too long"))
    } else if blst_poly_into_pc_poly(p).unwrap().is_zero() {
        Ok(G1_IDENTITY)
    } else {
        let (powers, _) = trim(&ks.params, &ks.params.max_degree() - 1).unwrap();
        let (com, _rand) =
            KZG_Bls12_381::commit(&powers, &blst_poly_into_pc_poly(p).unwrap(), None, None)
                .unwrap();
        Ok(pc_g1projective_into_blst_p1(com.0.into_group()).unwrap())
    }
}

pub fn compute_proof_single(p: &PolyData, x: &BlstFr, ks: &KZGSettings) -> ArkG1 {
    let (powers, _) = trim(&ks.params, &ks.params.max_degree() - 1).unwrap();
    let proof = KZG::<Bls12_381, UniPoly_381>::open(
        &powers,
        &blst_poly_into_pc_poly(p).unwrap(),
        blst_fr_into_pc_fr(x),
        &ks.rand2,
    )
    .unwrap();
    pc_g1projective_into_blst_p1(proof.w.into_group()).unwrap()
}

pub fn eval_poly(p: &PolyData, x: &BlstFr) -> BlstFr {
    let poly = blst_poly_into_pc_poly(p).unwrap();
    pc_fr_into_blst_fr(poly.evaluate(&blst_fr_into_pc_fr(x)))
}

pub fn check_proof_single(
    com: &ArkG1,
    proof: &ArkG1,
    x: &BlstFr,
    value: &BlstFr,
    ks: &KZGSettings,
) -> bool {
    let (_powers, vk) = trim(&ks.params, &ks.params.max_degree() - 1).unwrap();
    let projective = blst_p1_into_pc_g1projective(&com.0).unwrap();
    let affine = Affine::<g1::Config>::from(projective);
    let mut com = Commitment::empty();
    com.0 = affine;
    let arkproof = Proof {
        w: blst_p1_into_pc_g1projective(&proof.0)
            .unwrap()
            .into_affine(),
        random_v: None,
    };
    KZG_Bls12_381::check(
        &vk,
        &com,
        blst_fr_into_pc_fr(x),
        blst_fr_into_pc_fr(value),
        &arkproof,
    )
    .unwrap()
}

pub fn compute_proof_multi(p: &PolyData, x: &BlstFr, n: usize, ks: &KZGSettings) -> ArkG1 {
    let mut divisor = PolyData::new(n + 1).unwrap();
    let x_pow_n = x.pow(n);

    divisor.set_coeff_at(0, &x_pow_n.negate());

    for i in 1..n {
        divisor.set_coeff_at(i, &BlstFr::zero());
    }
    divisor.set_coeff_at(n, &BlstFr::one());

    let mut p = p.clone();
    let q = p.div(&divisor).unwrap();

    commit_to_poly(&q, ks).unwrap()
}

pub fn check_proof_multi(
    com: &ArkG1,
    proof: &ArkG1,
    x: &BlstFr,
    ys: &[BlstFr],
    n: usize,
    ks: &KZGSettings,
) -> bool {
    let mut interp = PolyData::new(n).unwrap();

    interp.coeffs = ks.fs.fft_fr(ys, true).unwrap();

    let inv_x = x.inverse();
    let mut inv_x_pow = inv_x;
    for i in 1..n {
        interp.coeffs[i] = interp.coeffs[i].mul(&inv_x_pow);
        inv_x_pow = inv_x_pow.mul(&inv_x);
    }

    let x_pow = inv_x_pow.inverse();
    let mut xn2 = ks.params.h.into_group();
    xn2.mul_assign(blst_fr_into_pc_fr(&x_pow));
    let xn_minus_yn = blst_p2_into_pc_g2projective(&ks.secret_g2[n]).unwrap() - xn2;

    let is1 = blst_p1_into_pc_g1projective(&commit_to_poly(&interp, ks).unwrap().0).unwrap();

    let commit_minus_interp = blst_p1_into_pc_g1projective(&com.0).unwrap() - is1;
    pairings_verify(
        &pc_g1projective_into_blst_p1(commit_minus_interp).unwrap(),
        &pc_g2projective_into_blst_p2(ks.params.h.into_group()).unwrap(),
        proof,
        &pc_g2projective_into_blst_p2(xn_minus_yn).unwrap(),
    )
}

pub const G2_GENERATOR: ArkG2 = ArkG2(P2 {
    x: blst_fp2 {
        fp: [
            blst_fp {
                l: [
                    0xf5f28fa202940a10,
                    0xb3f5fb2687b4961a,
                    0xa1a893b53e2ae580,
                    0x9894999d1a3caee9,
                    0x6f67b7631863366b,
                    0x058191924350bcd7,
                ],
            },
            blst_fp {
                l: [
                    0xa5a9c0759e23f606,
                    0xaaa0c59dbccd60c3,
                    0x3bb17e18e2867806,
                    0x1b1ab6cc8541b367,
                    0xc2b6ed0ef2158547,
                    0x11922a097360edf3,
                ],
            },
        ],
    },
    y: blst_fp2 {
        fp: [
            blst_fp {
                l: [
                    0x4c730af860494c4a,
                    0x597cfa1f5e369c5a,
                    0xe7e6856caa0a635a,
                    0xbbefb5e96e0d495f,
                    0x07d3a975f0ef25a2,
                    0x0083fd8e7e80dae5,
                ],
            },
            blst_fp {
                l: [
                    0xadc0fc92df64b05d,
                    0x18aa270a2b1461dc,
                    0x86adac6a3be4eba0,
                    0x79495c4ec93da33a,
                    0xe7175850a43ccaed,
                    0x0b2bc2a163de1bf2,
                ],
            },
        ],
    },
    z: blst_fp2 {
        fp: [
            blst_fp {
                l: [
                    0x760900000002fffd,
                    0xebf4000bc40c0002,
                    0x5f48985753c758ba,
                    0x77ce585370525745,
                    0x5c071a97a256ec6d,
                    0x15f65ec3fa80e493,
                ],
            },
            blst_fp {
                l: [
                    0x0000000000000000,
                    0x0000000000000000,
                    0x0000000000000000,
                    0x0000000000000000,
                    0x0000000000000000,
                    0x0000000000000000,
                ],
            },
        ],
    },
});

pub const G2_NEGATIVE_GENERATOR: ArkG2 = ArkG2(P2 {
    x: blst_fp2 {
        fp: [
            blst_fp {
                l: [
                    0xf5f28fa202940a10,
                    0xb3f5fb2687b4961a,
                    0xa1a893b53e2ae580,
                    0x9894999d1a3caee9,
                    0x6f67b7631863366b,
                    0x058191924350bcd7,
                ],
            },
            blst_fp {
                l: [
                    0xa5a9c0759e23f606,
                    0xaaa0c59dbccd60c3,
                    0x3bb17e18e2867806,
                    0x1b1ab6cc8541b367,
                    0xc2b6ed0ef2158547,
                    0x11922a097360edf3,
                ],
            },
        ],
    },
    y: blst_fp2 {
        fp: [
            blst_fp {
                l: [
                    0x6d8bf5079fb65e61,
                    0xc52f05df531d63a5,
                    0x7f4a4d344ca692c9,
                    0xa887959b8577c95f,
                    0x4347fe40525c8734,
                    0x197d145bbaff0bb5,
                ],
            },
            blst_fp {
                l: [
                    0x0c3e036d209afa4e,
                    0x0601d8f4863f9e23,
                    0xe0832636bacc0a84,
                    0xeb2def362a476f84,
                    0x64044f659f0ee1e9,
                    0x0ed54f48d5a1caa7,
                ],
            },
        ],
    },
    z: blst_fp2 {
        fp: [
            blst_fp {
                l: [
                    0x760900000002fffd,
                    0xebf4000bc40c0002,
                    0x5f48985753c758ba,
                    0x77ce585370525745,
                    0x5c071a97a256ec6d,
                    0x15f65ec3fa80e493,
                ],
            },
            blst_fp {
                l: [
                    0x0000000000000000,
                    0x0000000000000000,
                    0x0000000000000000,
                    0x0000000000000000,
                    0x0000000000000000,
                    0x0000000000000000,
                ],
            },
        ],
    },
});

pub fn pairings_verify(a1: &ArkG1, a2: &ArkG2, b1: &ArkG1, b2: &ArkG2) -> bool {
    let ark_a1_neg = blst_p1_into_pc_g1projective(&a1.0)
        .unwrap()
        .neg()
        .into_affine();
    let ark_b1 = blst_p1_into_pc_g1projective(&b1.0).unwrap().into_affine();
    let ark_a2 = blst_p2_into_pc_g2projective(a2).unwrap().into_affine();
    let ark_b2 = blst_p2_into_pc_g2projective(b2).unwrap().into_affine();
        
    Bls12_381::multi_pairing([ark_a1_neg, ark_b1], [ark_a2, ark_b2])
    .0.is_one()
}
