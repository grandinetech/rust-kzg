#![allow(non_camel_case_types)]
use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};
use ark_poly_commit::kzg10::{
    Commitment, Powers, Proof, Randomness, UniversalParams, VerifierKey, KZG10,
};
use ark_poly_commit::*;
use ark_std::UniformRand;

use super::utils::{
    blst_fr_into_pc_fr, blst_p1_into_pc_g1projective, blst_poly_into_pc_poly, pc_fr_into_blst_fr,
    pc_g1projective_into_blst_p1, PolyData, blst_p2_into_pc_g2projective, pc_g2projective_into_blst_p2
};
use super::{/*Fp , Fr as BlstFr,*/ P2};
use crate::fft_g1::G1_IDENTITY;
use crate::kzg_types::{FsFr as BlstFr, ArkG1, ArkG2};
use ark_bls12_381::{g1, Bls12_381, Fr, g2};
use ark_ec::msm::VariableBaseMSM;
use ark_ec::{
    models::short_weierstrass_jacobian::GroupAffine, AffineCurve, PairingEngine, ProjectiveCurve,
};
use ark_ff::{/*biginteger::BigInteger256, */Field, PrimeField};
use ark_poly::univariate::DensePolynomial as DensePoly;
use ark_std::{marker::PhantomData, ops::Div, test_rng, vec, Zero, One};
use blst::{blst_fp, blst_fp2};
use kzg::{Fr as FrTrait, Poly, FFTSettings as FFTTrait, FFTFr};
use rand::rngs::StdRng;
use std::ops::{Neg, MulAssign};

use super::fft::SCALE2_ROOT_OF_UNITY;

type UniPoly_381 = DensePoly<<Bls12_381 as PairingEngine>::Fr>;
type KZG_Bls12_381 = KZG10<Bls12_381, UniPoly_381>;

// pub const G2_GENERATOR: P2 = P2 {
//     x: blst_fp2 {
//         fp: [
//             blst_fp {
//                 l: [
//                     0xf5f28fa202940a10,
//                     0xb3f5fb2687b4961a,
//                     0xa1a893b53e2ae580,
//                     0x9894999d1a3caee9,
//                     0x6f67b7631863366b,
//                     0x058191924350bcd7,
//                 ],
//             },
//             blst_fp {
//                 l: [
//                     0xa5a9c0759e23f606,
//                     0xaaa0c59dbccd60c3,
//                     0x3bb17e18e2867806,
//                     0x1b1ab6cc8541b367,
//                     0xc2b6ed0ef2158547,
//                     0x11922a097360edf3,
//                 ],
//             },
//         ],
//     },
//     y: blst_fp2 {
//         fp: [
//             blst_fp {
//                 l: [
//                     0x4c730af860494c4a,
//                     0x597cfa1f5e369c5a,
//                     0xe7e6856caa0a635a,
//                     0xbbefb5e96e0d495f,
//                     0x07d3a975f0ef25a2,
//                     0x0083fd8e7e80dae5,
//                 ],
//             },
//             blst_fp {
//                 l: [
//                     0xadc0fc92df64b05d,
//                     0x18aa270a2b1461dc,
//                     0x86adac6a3be4eba0,
//                     0x79495c4ec93da33a,
//                     0xe7175850a43ccaed,
//                     0x0b2bc2a163de1bf2,
//                 ],
//             },
//         ],
//     },
//     z: blst_fp2 {
//         fp: [
//             blst_fp {
//                 l: [
//                     0x760900000002fffd,
//                     0xebf4000bc40c0002,
//                     0x5f48985753c758ba,
//                     0x77ce585370525745,
//                     0x5c071a97a256ec6d,
//                     0x15f65ec3fa80e493,
//                 ],
//             },
//             blst_fp {
//                 l: [
//                     0x0000000000000000,
//                     0x0000000000000000,
//                     0x0000000000000000,
//                     0x0000000000000000,
//                     0x0000000000000000,
//                     0x0000000000000000,
//                 ],
//             },
//         ],
//     },
// };

/*This segment has been copied from https://github.com/arkworks-rs/poly-commit/blob/master/src/kzg10/mod.rs,
Due to being private and, therefore, unreachable*/
pub fn trim(
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

pub struct KZG<E: PairingEngine, P: UVPolynomial<E::Fr>> {
    _engine: PhantomData<E>,
    _poly: PhantomData<P>,
}

/*This segment has been copied from https://github.com/arkworks-rs/poly-commit/blob/master/src/kzg10/mod.rs,
Due to being private and, therefore, unreachable*/
impl<E, P> KZG<E, P>
where
    E: PairingEngine,
    P: UVPolynomial<E::Fr, Point = E::Fr>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    pub fn open<'a>(
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

    pub(crate) fn open_with_witness_polynomial<'a>(
        powers: &Powers<Bls12_381>,
        point: Fr,
        randomness: &Randomness<Fr, DensePoly<Fr>>,
        witness_polynomial: &DensePoly<Fr>,
        hiding_witness_polynomial: Option<&DensePoly<Fr>>,
    ) -> Result<Proof<Bls12_381>, Error> {
        Self::check_degree_is_too_large(witness_polynomial.degree(), powers.size())?;
        let (num_leading_zeros, witness_coeffs) =
            skip_leading_zeros_and_convert_to_bigints(witness_polynomial);

        let w = VariableBaseMSM::multi_scalar_mul(
            &powers.powers_of_g[num_leading_zeros..],
            &witness_coeffs,
        );

        let random_v = if let Some(_hiding_witness_polynomial) = hiding_witness_polynomial {
            let blinding_p = &randomness.blinding_polynomial;
            let blinding_evaluation = blinding_p.evaluate(&point);


            Some(blinding_evaluation)
        } else {
            None
        };

        Ok(Proof {
            w: w.into_affine(),
            random_v,
        })
    }

    pub(crate) fn check_degree_is_too_large(degree: usize, num_powers: usize) -> Result<(), Error> {
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
fn skip_leading_zeros_and_convert_to_bigints<F: PrimeField, P: UVPolynomial<F>>(
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
    let coeffs = p.iter().map(|s| s.into_repr()).collect::<Vec<_>>();
    coeffs
}

// pub(crate) const fr_one = BlstFr{blst::blst_fr{
//     l: [
//         8589934590,
//         6378425256633387010,
//         11064306276430008309,
//         1739710354780652911,
//     ]
// }};

// pub(crate) const fr_zero: BlstFr = BlstFr {
//     l: [0x0, 0x0, 0x0, 0x0],
// };

pub struct FFTSettings {
    pub max_width: usize,
    pub root_of_unity: BlstFr,
    pub expanded_roots_of_unity: Vec<BlstFr>,
    pub reverse_roots_of_unity: Vec<BlstFr>,
    pub domain: Radix2EvaluationDomain<Fr>,
}

// impl Default for FFTSettings {
//     fn default() -> FFTSettings {
//         FFTSettings {
//             max_width: 0,
//             root_of_unity: BlstFr::zero(),
//             expanded_roots_of_unity: Vec::new(),
//             reverse_roots_of_unity: Vec::new(),
//             domain: Radix2EvaluationDomain::<Fr>::new(0 as usize).unwrap(),
//         }
//     }
// }

/*This segment has been copied from https://github.com/arkworks-rs/algebra/blob/master/poly/src/domain/utils.rs,
Due to being private and, therefore, unreachable*/
// pub(crate) fn compute_powers_serial(size: usize, root: Fr) -> Vec<Fr> {
//     compute_powers_and_mul_by_const_serial(size, root, blst_fr_into_pc_fr(&BlstFr::one()))
// }

// pub(crate) fn compute_powers_and_mul_by_const_serial(size: usize, root: Fr, c: Fr) -> Vec<Fr> {
//     // let mut generated_powers = vec![c, root.clone()];
//     // while !(generated_powers.last().unwrap().eq(c)) && generated_powers.len() <= size{
//     //     generated_powers.push(generated_powers.last().unwrap().mul_assign(&root));
//     // }
//     // generated_powers;

//     let mut value = c;
//     let arr: Vec<Fr> = (0..size)
//         .map(|_| {
//             let old_value = value;
//             value *= root;
//             old_value
//         })
//         .collect();
//     let mut i = 2;
//     while i <= size && arr[i - 1] != c {
//         i += 1;
//     }
//     arr[..i].to_vec()
// }

/*This segment has been copied from https://github.com/arkworks-rs/algebra/blob/master/poly/src/domain/radix2/fft.rs,
Due to being private and, therefore, unreachable*/

// pub(super) fn roots_of_unity(domain: &Radix2EvaluationDomain<Fr>, root: Fr) -> Vec<Fr> {
//     compute_powers_serial(domain.size as usize, root)
// }

// pub fn expand_root_of_unity(root: &BlstFr, size: usize) -> Result<Vec<BlstFr>, String>{
//     let first = compute_powers_serial(size, blst_fr_into_pc_fr(root));
//     let second = DensePoly::from_coefficients_vec(first);

//     Ok(pc_poly_into_blst_poly(second).unwrap().coeffs)
// }

pub fn expand_root_of_unity(root: &BlstFr, width: usize) -> Result<Vec<BlstFr>, String> {
    let mut generated_powers = vec![BlstFr::one(), root.clone()];

    while !(generated_powers.last().unwrap().is_one()) {
        if generated_powers.len() > width {
            return Err(String::from("Root of unity multiplied for too long"));
        }

        generated_powers.push(generated_powers.last().unwrap().mul(&root));
    }

    Ok(generated_powers)
}

impl FFTSettings {
    pub fn from_scale(max_scale: usize) -> Result<FFTSettings, String> {
        if max_scale >= SCALE2_ROOT_OF_UNITY.len() {
            return Err(String::from(
                "Scale is expected to be within root of unity matrix row size",
            ));
        }
        let max_width: usize = 1 << max_scale;
        let domain = Radix2EvaluationDomain::<Fr>::new(max_width as usize).unwrap();

        // let  roots = pc_poly_into_blst_poly(DensePoly::from_coefficients_vec(
        //     roots_of_unity(&domain, domain.group_gen),
        // ))
        // .unwrap()
        // .coeffs;

        let roots =
            expand_root_of_unity(&pc_fr_into_blst_fr(domain.group_gen), domain.size as usize)
                .unwrap();

        let mut reverse = roots.clone();
        reverse.reverse();

        Ok(FFTSettings {
            max_width: max_width,
            root_of_unity: pc_fr_into_blst_fr(domain.group_gen),
            expanded_roots_of_unity: roots,
            reverse_roots_of_unity: reverse,
            domain: domain
        })
    }
}

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
    fn default() -> KZGSettings {
        KZGSettings {
            fs: FFTSettings::default(),
            secret_g1: Vec::new(),
            secret_g2: Vec::new(),
            length: 0,
            params: KZG_Bls12_381::setup(1, false, &mut test_rng()).unwrap(),
            rand: test_rng(),
            rand2: Randomness::empty(),
        }
    }
}

pub fn default_kzg() -> KZGSettings {
        KZGSettings {
            fs: FFTSettings::default(),
            secret_g1: Vec::new(),
            secret_g2: Vec::new(),
            length: 0,
            params: KZG_Bls12_381::setup(1, false, &mut test_rng()).unwrap(),
            rand: test_rng(),
            rand2: Randomness::empty(),
    }
}

pub fn generate_trusted_setup(len: usize, _secret: [u8; 32usize]) -> (Vec<ArkG1>, Vec<ArkG2>) {
    let mut s_pow = Fr::from(1);
    let s = Fr::rand(&mut test_rng());
    let mut s1 = Vec::new();
    let mut s2 = Vec::new();
    for _i in 0..len{
        let mut temp = g1::G1Affine::new(g1::G1_GENERATOR_X, g1::G1_GENERATOR_Y, true).into_projective();
        temp.mul_assign(s_pow);
        s1.push(pc_g1projective_into_blst_p1(temp).unwrap());
        let mut temp = g2::G2Affine::new(g2::G2_GENERATOR_X, g2::G2_GENERATOR_Y, true).into_projective();
        temp.mul_assign(s_pow);
        s2.push(pc_g2projective_into_blst_p2(temp).unwrap());
        s_pow *= s;
    }

    (s1, s2)
}

// pub(crate) fn new_fft_settings(max_scale: u64) -> FFTSettings {
//     FFTSettings::default()
// }

pub(crate) fn new_kzg_settings(
    secret_g1: &Vec<ArkG1>,
    _secret_g2: &Vec<ArkG2>,
    length: u64,
    fs: FFTSettings,
) -> KZGSettings {
    let params = KZG_Bls12_381::setup(length as usize, true, &mut test_rng()).unwrap();
    let mut temp = Vec::new();
    for i in 0..length{
        temp.push(blst_p1_into_pc_g1projective(&secret_g1[i as usize].0).unwrap().into_affine());
    }
    
    
    let mut s_pow = Fr::from(1);
    let s = Fr::rand(&mut test_rng());
    let mut s1 = Vec::new();
    let mut s2 = Vec::new();
    for _i in 0..length{
        let mut temp = params.powers_of_g[0].into_projective();
        temp.mul_assign(s_pow);
        s1.push(pc_g1projective_into_blst_p1(temp).unwrap());
        let mut temp = params.h.into_projective();
        temp.mul_assign(s_pow);
        s2.push(pc_g2projective_into_blst_p2(temp).unwrap());
        s_pow *= s;
    }


    
    KZGSettings {
        // secret_g1: secret_g1.clone(),
        // secret_g2: secret_g2.clone(),
        secret_g1: s1,
        secret_g2: s2,
        length: length,
        params: params,
        fs: fs,
        ..Default::default()
    }
}

// pub(crate) fn fr_from_uint64(num: u64) -> BlstFr {
//     let fr = Fr::new(BigInteger256::from(num));
//     pc_fr_into_blst_fr(fr)
// }

// pub(crate) fn new_poly(len: usize) -> PolyData {
//     PolyData {
//         coeffs: vec![BlstFr::zero(); len],
//     }
// }

#[derive(Debug, PartialEq)]
pub(crate) struct TooLongPoly;

pub(crate) fn commit_to_poly(p: &PolyData, ks: &KZGSettings) -> Result<ArkG1, String> {
    if p.coeffs.len() > ks.length as usize {
        return Err(String::from("Poly given is too long"))
    } else if blst_poly_into_pc_poly(&p).unwrap().is_zero() {
        Ok(G1_IDENTITY)
    } else {
        let (powers, _) = trim(&ks.params, &ks.params.max_degree() - 1).unwrap();
        let (com, _rand) = KZG_Bls12_381::commit(
            &powers,
            &blst_poly_into_pc_poly(&p).unwrap(),
            None,
            None,
        )
        .unwrap();
        Ok(pc_g1projective_into_blst_p1(com.0.into_projective()).unwrap())
    }
}

pub(crate) fn compute_proof_single(p: &PolyData, x: &BlstFr, ks: &KZGSettings) -> ArkG1 {
    let (powers, _) = trim(&ks.params, &ks.params.max_degree() - 1).unwrap();
    let proof = KZG::<Bls12_381, UniPoly_381>::open(
        &powers,
        &blst_poly_into_pc_poly(p).unwrap(),
        blst_fr_into_pc_fr(x),
        &ks.rand2,
    )
    .unwrap();
    pc_g1projective_into_blst_p1(proof.w.into_projective()).unwrap()
}

pub(crate) fn eval_poly(p: &PolyData, x: &BlstFr) -> BlstFr {
    let poly = blst_poly_into_pc_poly(p).unwrap();
    // println!("PC POLY {:?} AND BL<S POLY {:?}",poly, p);
    pc_fr_into_blst_fr(poly.evaluate(&blst_fr_into_pc_fr(x)))
}

pub(crate) fn check_proof_single(
    com: &ArkG1,
    proof: &ArkG1,
    x: &BlstFr,
    value: &BlstFr,
    ks: &KZGSettings,
) -> bool {
    let (_powers, vk) = trim(&ks.params, &ks.params.max_degree() - 1).unwrap();
    let projective = blst_p1_into_pc_g1projective(&com.0).unwrap();
    let affine = GroupAffine::<g1::Parameters>::from(projective.clone());
    let mut com = Commitment::empty();
    com.0 = affine;
    let arkproof = Proof{w: blst_p1_into_pc_g1projective(&proof.0).unwrap().into_affine(), random_v: None};
    KZG_Bls12_381::check(
        &vk,
        &com,
        blst_fr_into_pc_fr(x),
        blst_fr_into_pc_fr(value),
        &arkproof,
    )
    .unwrap()
}

pub(crate) fn fr_add(x: &BlstFr, y: &BlstFr) -> BlstFr {
    let pcx = blst_fr_into_pc_fr(x);
    let pcy = blst_fr_into_pc_fr(y);
    let sum = pcx + pcy;
    pc_fr_into_blst_fr(sum)
}

// pub(crate) fn fr_mul(x: &BlstFr, roots: &BlstFr) -> BlstFr {
//     let pcx = blst_fr_into_pc_fr(x);
//     let PCroots = blst_fr_into_pc_fr(roots);
//     let mul = pcx * PCroots;
//     pc_fr_into_blst_fr(mul)
// }

pub(crate) fn compute_proof_multi(
    p: &PolyData,
    x: &BlstFr,
    n: usize,
    ks: &KZGSettings,
) -> ArkG1 {
    let rng = &mut test_rng();
    let mut pcdivisor = DensePoly::rand(n, rng);

    let pcx = blst_fr_into_pc_fr(x);

    let x_pow_n = pcx.pow(&[n as u64]);
    let fr_neg = x_pow_n.neg();
    let _temp = std::mem::replace(
        &mut pcdivisor.coeffs[0],
        blst_fr_into_pc_fr(&BlstFr::zero()),
    );
    for x in 1..n - 1 {
        let _temp = std::mem::replace(&mut pcdivisor.coeffs[x as usize], fr_neg);
    }
    let _temp = std::mem::replace(
        &mut pcdivisor.coeffs[n],
        blst_fr_into_pc_fr(&BlstFr::one()),
    );

    let q = &blst_poly_into_pc_poly(p).unwrap() / &pcdivisor;

    let (powers, _) = trim(&ks.params, &ks.params.max_degree() - 1).unwrap();
    let (com, _rand) = KZG_Bls12_381::commit(&powers, &q, None, None).unwrap();

    pc_g1projective_into_blst_p1(com.0.into_projective()).unwrap()
}

pub(crate) fn check_proof_multi(
    com: &ArkG1,
    proof: &ArkG1,
    x: &BlstFr,
    ys: &Vec<BlstFr>,
    n: usize,
    ks: &KZGSettings,
) -> bool {
    let mut interp = PolyData::new(n).unwrap();

    interp.coeffs = ks.fs.fft_fr(ys, true).unwrap();

    let inv_x = x.inverse();
    let mut inv_x_pow = x.clone();
    for i in 1..n {
        interp.coeffs[i] = interp.coeffs[i].mul(&inv_x_pow);
        inv_x_pow = inv_x_pow.mul(&inv_x);
    }

    let x_pow = inv_x_pow.inverse();
    // let mut xn2 = blst_p2_into_pc_g2projective(&G2_GENERATOR).unwrap();
    let mut xn2 = ks.params.h.into_projective();
    // let mut temp = ks.params.h.into_projective();
    xn2.mul_assign(blst_fr_into_pc_fr(&x_pow));
    // println!("TEST1: {:?}", temp);
    // println!("TEST2: {:?}", xn2);
    // println!("TEST3: {:?}", blst_p2_into_pc_g2projective(&xn2.0));

    // let xn_minus_yn = pc_g2projective_into_blst_p2(blst_p2_into_pc_g2projective(&ks.secret_g2[n]).unwrap()- temp).unwrap();
    let xn_minus_yn = ks.params.neg_powers_of_h[&n].neg().into_projective() - xn2;


    let is1 = blst_p1_into_pc_g1projective(&commit_to_poly(&interp, ks).unwrap().0).unwrap();

    let commit_minus_interp = blst_p1_into_pc_g1projective(&com.0).unwrap() - is1;
    // let temp = pc_g2projective_into_blst_p2(ks.params.h.into_projective()).unwrap();
    let _test = pc_g2projective_into_blst_p2(ks.params.h.into_projective()).unwrap();
    let _a1 = Bls12_381::pairing(commit_minus_interp, blst_p2_into_pc_g2projective(&G2_GENERATOR).unwrap());
    let _a2 = Bls12_381::pairing(blst_p1_into_pc_g1projective(&proof.0).unwrap(), xn_minus_yn);
    Bls12_381::product_of_pairings(&[
            // (commit_minus_interp.into_affine().into(), blst_p2_into_pc_g2projective(&G2_GENERATOR).unwrap().into_affine().into()),
            (commit_minus_interp.into_affine().into(), ks.params.h.into()),
            (blst_p1_into_pc_g1projective(&proof.0).unwrap().into_affine().into(), xn_minus_yn.into_affine().into()),
        ])
        .is_one()
    // println!("TEST1: {:?}", a1);
    // println!("TEST2: {:?}", a2);
}


pub const G2_GENERATOR:ArkG2 = ArkG2(P2{x: blst_fp2{fp: [blst_fp{l: [0xf5f28fa202940a10, 0xb3f5fb2687b4961a, 0xa1a893b53e2ae580, 0x9894999d1a3caee9,
                                     0x6f67b7631863366b, 0x058191924350bcd7]},
                                    blst_fp{l:[0xa5a9c0759e23f606, 0xaaa0c59dbccd60c3, 0x3bb17e18e2867806, 0x1b1ab6cc8541b367,
                                     0xc2b6ed0ef2158547, 0x11922a097360edf3]}]},
                                  y: blst_fp2{fp:[blst_fp{l:[0x4c730af860494c4a, 0x597cfa1f5e369c5a, 0xe7e6856caa0a635a, 0xbbefb5e96e0d495f,
                                     0x07d3a975f0ef25a2, 0x0083fd8e7e80dae5]},
                                    blst_fp{l:[0xadc0fc92df64b05d, 0x18aa270a2b1461dc, 0x86adac6a3be4eba0, 0x79495c4ec93da33a,
                                     0xe7175850a43ccaed, 0x0b2bc2a163de1bf2]}]},
                                  z: blst_fp2{fp:[blst_fp{l:[0x760900000002fffd, 0xebf4000bc40c0002, 0x5f48985753c758ba, 0x77ce585370525745,
                                     0x5c071a97a256ec6d, 0x15f65ec3fa80e493]},
                                    blst_fp{l:[0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000,
                                     0x0000000000000000, 0x0000000000000000]}]}});

// fn pairings_verify(a1: &ArkG1, a2: &ArkG2, b1:&ArkG1, b2: &ArkG2) -> bool {
//     let mut loop0: blst::blst_fp12 = blst::blst_fp12::default();
//     let mut loop1: blst::blst_fp12 = blst::blst_fp12::default();
//     let mut gt_point: blst::blst_fp12 = blst::blst_fp12::default();
//     let mut aa1: blst::blst_p1_affine = blst::blst_p1_affine::default();
//     let mut bb1: blst::blst_p1_affine = blst::blst_p1_affine::default();
//     let mut aa2: blst::blst_p2_affine = blst::blst_p2_affine::default();
//     let mut bb2: blst::blst_p2_affine = blst::blst_p2_affine::default();

//     // As an optimisation, we want to invert one of the pairings,
//     // so we negate one of the points.
//     let mut a1neg = a1.clone().0;
//     unsafe{
//         blst::blst_p1_cneg(&mut a1neg, true);

//         blst::blst_p1_to_affine(&mut aa1, &a1neg);
//         blst::blst_p1_to_affine(&mut bb1, &b1.0 as *const _);
//         blst::blst_p2_to_affine(&mut aa2, &a2.0 as *const _);
//         blst::blst_p2_to_affine(&mut bb2, &b2.0 as *const _);

//         blst::blst_miller_loop(&mut loop0, &aa2, &aa1);
//         blst::blst_miller_loop(&mut loop1, &bb2, &bb1);

// // let b = g2::G2Affine::new(g2::G2_GENERATOR_X, g2::G2_GENERATOR_Y, false).into_projective();

//         blst::blst_fp12_mul(&mut gt_point, &loop0, &loop1);
//         blst::blst_final_exp(&mut gt_point, &gt_point);
//                 println!("TEST1: {:?}", gt_point);
//                 // println!("TEST1: {:?}", G2_GENERATOR);
//         blst::blst_fp12_is_one(&gt_point as *const _)
//     }
// }