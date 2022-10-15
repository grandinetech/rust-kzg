use std::convert::TryInto;

use kzg::{KZGSettings, Fr, Poly};

use crate::types::fr::FsFr;
use crate::types::g1::FsG1;

use crate::kzg_proofs::g1_linear_combination;
use crate::types::kzg_settings::FsKZGSettings;
use crate::types::poly::FsPoly;

// is .h failo

// typedef blst_p1 g1_t;         /**< Internal G1 group element type */
// typedef blst_p2 g2_t;         /**< Internal G2 group element type */
// typedef blst_fr fr_t;         /**< Internal Fr field element type */

// typedef g1_t KZGCommitment;
// typedef g1_t KZGProof;
// typedef fr_t BLSFieldElement;

/**
 * Montgomery batch inversion in finite field
 *
 * @param[out] out The inverses of @p a, length @p len
 * @param[in]  a   A vector of field elements, length @p len
 * @param[in]  len Length
 */

fn fr_batch_inv(out : &mut Vec<FsFr>, a : &Vec<FsFr>, len: usize) {
    let prod : &mut Vec<FsFr> = &mut vec![FsFr::default(); len];
    // let mut inv : &mut FsFr;
    let mut i: usize = 1;

    prod[0] = a[0];

    while i < len {
        prod[i] = a[i].mul(&prod[i - 1]);
        i += 1;
    }

    let mut inv: &mut FsFr = &mut prod[len - 1].eucl_inverse();

    i = len - 1;
    while i > 0{
        out[i] = prod[i - 1].mul(inv);
        *inv = a[i].mul(inv);
        i -= 1;
    }
    out[0] = *inv;
}

fn bytes_to_bls_field(out : &mut FsFr, bytes: [u8; 32usize])
{
    *out = FsFr::from_scalar(bytes);
}


fn g1_lincomb(out : &mut FsG1, points : &[FsG1], scalars : &[FsFr], num_points : usize){
    g1_linear_combination(out, points, scalars, 
        num_points) 
}

fn blob_to_kzg_commitment(out : &mut FsG1, blob : Vec<FsFr>, s : &FsKZGSettings) {
    g1_lincomb(out, &s.secret_g1, &blob, s.secret_g2.len());
  }

fn verify_kzg_proof(out : &mut bool, polynomial_kzg : &FsG1, z : &FsFr,
    y : &FsFr, kzg_proof : &FsG1, s : &FsKZGSettings){
        *out = match s.check_proof_single(polynomial_kzg, kzg_proof, z, y) {
            Ok(v) => v,
            Err(_) => false,
        };
}


fn compute_kzg_proof(out : &mut FsG1, p : &mut FsPoly,
    x : &FsFr, s : &FsKZGSettings){
        if p.len() > s.secret_g1.len(){
            return;
        }

        let mut y:FsFr = FsFr::default();
        evaluate_polynomial_in_evaluation_form(&mut y, p, x, s);
      
        let mut tmp: FsFr = FsFr::default();
        let roots_of_unity: &Vec<FsFr> = &s.fs.expanded_roots_of_unity; // gali buti ne tas
        let mut i: usize = 0;
        let mut m: usize = 0;
      
        let mut q: FsPoly = FsPoly::new(p.len()).unwrap();
      
        let mut inverses_in: Vec<FsFr> = vec![FsFr::default(); p.len()];
        let mut inverses: Vec<FsFr> = vec![FsFr::default(); p.len()];
      
        while i < q.len() {
          if x.equals(&roots_of_unity[i]) {
            m = i + 1;
            continue;
          }
          // (p_i - y) / (ω_i - x)
          q.coeffs[i] = p.coeffs[i].sub(&y);
          inverses_in[i] = roots_of_unity[i].sub(x);
          i = i + 1;
        }
      
        fr_batch_inv(&mut inverses, &inverses_in, q.len());
      
        i = 0;
        while i < q.len() {
            q.coeffs[i] = q.coeffs[i].mul(&inverses[i]);
            i += 1;
        }
      
        if m > 0 { // ω_m == x
            q.coeffs[m] = FsFr::zero();
            m -= 1;
            i = 0;
            while i < q.coeffs.len() {
                if i == m{
                    continue;
                }
                // (p_i - y) * ω_i / (x * (x - ω_i))
                tmp = x.sub(&roots_of_unity[i]);
                inverses_in[i] = tmp.mul(x);
                i += 1;
            }
            fr_batch_inv(&mut inverses, &inverses_in, q.coeffs.len());
            i = 0;
            while i < q.coeffs.len() {
                tmp = p.coeffs[i].sub(&y);
                tmp = tmp.mul(&roots_of_unity[i]);
                tmp = tmp.mul(&inverses[i]);
                q.coeffs[m] = q.coeffs[m].add(&tmp);
                i += 1;
            }
        }
            
        g1_lincomb(out, &s.secret_g1, &q.coeffs, q.coeffs.len());
}


fn evaluate_polynomial_in_evaluation_form(out : &mut FsFr, p : &FsPoly, 
    x : &FsFr, s : &FsKZGSettings) {
    let mut tmp: FsFr = FsFr::default();

    let mut inverses_in: Vec<FsFr> = vec![FsFr::default(); p.len()];
    let mut inverses: Vec<FsFr> = vec![FsFr::default(); p.len()];
    let mut i: usize = 0;
    let roots_of_unity: &Vec<FsFr> = &s.fs.expanded_roots_of_unity; // gali buti ne tas  

    while i < p.len(){
      if x.equals(&roots_of_unity[i]) {
        *out = p.coeffs[i];
      }

        inverses_in[i] =x.sub(&roots_of_unity[i]);
        i += 1;
    }
    fr_batch_inv(&mut inverses, &inverses_in, p.len());
  
    *out = FsFr::zero();
    i = 0;
    while i < p.len() {
      tmp = inverses[i].mul(&roots_of_unity[i]);
      tmp = tmp.mul(&p.coeffs[i]);
      *out = out.add(&tmp);
      i += 1;
    }
    tmp = FsFr::from_u64(p.len().try_into().unwrap());
    *out = match out.div(&tmp)
    {
        Ok(v) => v,
        Err(_) =>  panic!()
    };
    tmp = x.pow(p.len());
    tmp = tmp.sub(&FsFr::one());
    *out = out.mul(&tmp);
  
  }


// kompiliavimo komanda: $env:CARGO_INCREMENTAL=0; cargo build
// kita: cargo test --package blst_from_scratch --test eip_4844 -- tests::test_g1_lincomb --exact --nocapture