use std::convert::TryInto;
use std::env::set_current_dir;

use kzg::{Fr, KZGSettings, Poly, G1, G2, FFTSettings};
use rand::{Rng, rngs::StdRng, SeedableRng};
use ssz_rs::{U256, serialize};

use sha2::{Sha256, Digest};

fn u64_to_bytes(x: u64) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[0..8].copy_from_slice(&x.to_le_bytes());
    bytes
}

// Tests taken from https://github.com/dankrad/c-kzg/blob/4844/min-bindings/python/tests.py
pub fn bytes_to_bls_field_test<TFr: Fr>
(
    bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> TFr,
    bytes_from_bls_field: &dyn Fn(&TFr) -> [u8; 32usize]
)
{
    let x: u64 = 329;    
    let x_bytes = u64_to_bytes(x);
    
    let x_bls = bytes_to_bls_field(&x_bytes);
    
    assert_eq!(bytes_from_bls_field(&x_bls), x_bytes);
    assert_eq!(x, x_bls.to_u64_arr()[0]);
}

const EXPECTED_POWERS: [[u64; 4usize]; 11] = [
    [1, 0, 0, 0],
    [32930439, 0, 0, 0],
    [1084413812732721, 0, 0, 0],
    [15773128324309817559, 1935, 0, 0],
    [17639716667354648417, 63748557064, 0, 0],
    [14688837229838358055, 2099267969765560859, 0, 0],
    [17806839894568993937, 15217493595388594120, 3747534, 0],
    [17407663719861420663, 10645919139951883969, 123407966953127, 0],
    [9882663619548185281, 9079722283539367550, 5594831647882181930, 220],
    [4160126872399834567, 5941227867469556516, 11658769961926678707, 7254684264],
    [4000187329613806065, 4317886535621327299, 17988956659770583631, 238899937640724696]
];

// Simple test of compute_powers
pub fn compute_powers_test<TFr: Fr>
( 
    bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> TFr,
    compute_powers: &dyn Fn(&TFr, usize) -> Vec<TFr>
) 
{
    let x: u64 = 32930439;
    let n = 11;
    
    let x_bytes: [u8; 32] = u64_to_bytes(x);
    
    let x_bls = bytes_to_bls_field(&x_bytes);
    
    let powers = compute_powers(&x_bls, n);
    
    for (p, expected_p) in powers.iter().zip(EXPECTED_POWERS.iter()) {
        assert_eq!(expected_p, &p.to_u64_arr());
    }
}

pub fn evaluate_polynomial_in_evaluation_form_test<TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>
>(
    bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> TFr,
    load_trusted_setup: &dyn Fn(&str) -> TKZGSettings,
    evaluate_polynomial_in_evaluation_form: &dyn Fn(&TPoly, &TFr, &TKZGSettings) -> TFr
)
{
    let lvals: [[u64; 4]; 4] = [
        [16, 13, 0, 0], // 239807672958224171024
        [10, 13, 0, 0], // 239807672958224171018
        [281474976710667, 17006436628450967565, 10183145419576640720, 0], // 3465144826073652318776269530687742778510060141723586134027
        [18446462594437873676, 7474466853796666379, 11954817552772682548, 8353516859464449351] // 52435875175126190475982595682112313518914282969839895044573213904131443392524
    ];

    let mut lvals_bls: TPoly = TPoly::new(lvals.len()).unwrap();
    for (i, lval) in lvals.iter().enumerate() {
        let lval_bytes: [u8; 32] = lval.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>().try_into().unwrap();
        let lval_bls = bytes_to_bls_field(&lval_bytes);
        lvals_bls.set_coeff_at(i, &lval_bls);
    }
    
    let x: u64 = 2;
    let x_bytes: [u8; 32] = u64_to_bytes(x);
    let x_bls = bytes_to_bls_field(&x_bytes);
    
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup("src/tests/tiny_trusted_setup.txt");
    
    let y_bls = evaluate_polynomial_in_evaluation_form(&lvals_bls, &x_bls, &ts);
    
    assert_eq!(y_bls.to_u64_arr(),  [28, 13, 0, 0]);
}

pub fn compute_commitment_for_blobs_test<TFr : Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>
>(
    load_trusted_setup: &dyn Fn(&str) -> TKZGSettings,
    bytes_to_bls_field: &dyn Fn(&[u8; 32usize]) -> TFr,
    bytes_from_bls_field: &dyn Fn(&TFr) -> [u8; 32usize],
    bytes_from_g1: &dyn Fn(&TG1) -> [u8; 48usize],
    compute_powers: &dyn Fn(&TFr, usize) -> Vec<TFr>,
    vector_lincomb: &dyn Fn(&[Vec<TFr>], &[TFr]) -> Vec<TFr>,
    g1_lincomb: &dyn Fn(&[TG1], &[TFr]) -> TG1,
    evaluate_polynomial_in_evaluation_form: &dyn Fn(&TPoly, &TFr, &TKZGSettings) -> TFr,
    blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
    compute_kzg_proof: &dyn Fn(&TPoly, &TFr, &TKZGSettings) -> TG1,
    verify_kzg_proof: &dyn Fn(&TG1, &TFr, &TFr, &TG1, &TKZGSettings) -> bool,
)
{
    // Commit to a few random blobs
    const BLOB_SIZE: usize = 4096;
    const MAX_BLOBS_PER_BLOCK: usize = 16;

    let mut rng = StdRng::seed_from_u64(0);
    
    let mut blobs = Vec::new();

    let mut blobs_sedes: ssz_rs::List<ssz_rs::Vector<[u8; 32], BLOB_SIZE>, MAX_BLOBS_PER_BLOCK> = ssz_rs::List::default();
    for _ in 0..3 {
        let mut vec = Vec::new();
        let mut vec_sedes: ssz_rs::Vector<[u8; 32], BLOB_SIZE> = ssz_rs::Vector::default();
        for j in 0 .. BLOB_SIZE{
            let bytes: [u8; 32] = rng.gen();
            
            let fr = bytes_to_bls_field(&bytes);
            
            let tmp_bytes: [u8; 32] = bytes_from_bls_field(&fr);
            vec.push(fr);
            vec_sedes[j] = tmp_bytes;
        }
        blobs_sedes.push(vec_sedes);
        blobs.push(vec);
    }

    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup("src/tests/trusted_setup.txt");

    let kzg_commitments = blobs.iter().map(|blob| 
        blob_to_kzg_commitment(blob, &ts)
        ).collect::<Vec<TG1>>();

    let mut kzg_commitments_sedes:ssz_rs::List<ssz_rs::Vector<u8, 48>, MAX_BLOBS_PER_BLOCK> = ssz_rs::List::default();
    for comm in kzg_commitments.iter(){
        let r: [u8; 48usize] = bytes_from_g1(comm);
        let mut vec: ssz_rs::Vector<u8, 48> = ssz_rs::Vector::default();
        for (i, u) in r.iter().enumerate(){
            vec[i] = *u;
        }
        kzg_commitments_sedes.push(vec);
    }
    
    //  Compute polynomial commitments for these blobs
    // We don't follow the spec exactly to get the hash, but it shouldn't matter since it's random data

    let encoded_blobs = serialize(&blobs_sedes).unwrap();
    let encoded_commitments = serialize(&kzg_commitments_sedes).unwrap();

    let hashed: [u8; 32] = Sha256::digest([encoded_blobs, encoded_commitments].concat()).into();

    let r: TFr = bytes_to_bls_field(&hashed);

    let r_powers = compute_powers(&r, blobs.len());
    
    let values = vector_lincomb(&blobs, &r_powers);

    let mut aggregated_poly: TPoly = TPoly::new(values.len()).unwrap();
    for (i, value) in values.iter().enumerate() {
        aggregated_poly.set_coeff_at(i, value);
    }

    let aggregated_poly_commitment = g1_lincomb(&kzg_commitments, &r_powers);


    let simple_commitment = blob_to_kzg_commitment(&values, &ts);

    // Compute proof

    let mut values_sedes: ssz_rs::List<U256, BLOB_SIZE> = ssz_rs::List::default();

    for value in values.iter(){
        let bytes: [u8; 32] = bytes_from_bls_field(value);
        values_sedes.push(ssz_rs::U256(bytes));
    }

    let encoded_polynomial = serialize(&values_sedes).unwrap();
    let encoded_polynomial_length = serialize(&values.len()).unwrap();

    let bytes: [u8; 48usize] = bytes_from_g1(&aggregated_poly_commitment);

    let mut vec: ssz_rs::Vector<u8, 48> = ssz_rs::Vector::default();
    for u in bytes.iter(){
        vec.push(*u);
    }
    let encoded_commitment = serialize(&vec).unwrap();

    let k = [encoded_polynomial, encoded_polynomial_length, encoded_commitment].concat();
    let hashed_polynomial_and_commitment: [u8; 32] = Sha256::digest(k).into();

    let x = bytes_to_bls_field(&hashed_polynomial_and_commitment);

    let proof = compute_kzg_proof(&mut aggregated_poly, &x, &ts);

    // Verify proof

    let y = evaluate_polynomial_in_evaluation_form(&aggregated_poly, &x, &ts);

    assert_eq!(bytes_from_g1(&simple_commitment),  bytes_from_g1(&aggregated_poly_commitment));
     
    assert!(verify_kzg_proof(&simple_commitment, &x, &y, &proof, &ts), "Simple verification failed");  
    
    assert!(verify_kzg_proof(&aggregated_poly_commitment, &x, &y, &proof, &ts), "Verification failed");

    let mut x2_bytes: [u8; 32] = rng.gen();
    while x2_bytes == hashed_polynomial_and_commitment {
        x2_bytes = rng.gen();
    }

    let x2 = bytes_to_bls_field(&x2_bytes);

    let y2 = evaluate_polynomial_in_evaluation_form(&aggregated_poly, &x2, &ts);

    assert!(!verify_kzg_proof(&aggregated_poly_commitment, &x2, &y2, &proof, &ts), "Verification should fail");
}

// Test for the simplified 4844 interface

pub fn eip4844_test<TFr : Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>
>(load_trusted_setup: &dyn Fn(&str) -> TKZGSettings, 
blob_to_kzg_commitment: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
compute_aggregate_kzg_proof: &dyn Fn(&[TFr], &TKZGSettings) -> TG1,
verify_aggregate_kzg_proof: &dyn Fn(&[TFr], &[TG1], &TG1) -> bool) {
    const BLOB_SIZE: usize = 4096;

    let mut rng = StdRng::seed_from_u64(0);

    let mut blobs = (0..3)
        .map(|_| {
            (0..BLOB_SIZE)
                .map(|_| TFr::from_u64_arr(&rng.gen()))
                .collect::<Vec<TFr>>()
        })
        .collect::<Vec<Vec<TFr>>>();
    
    set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();
    let ts = load_trusted_setup("src/tests/tiny_trusted_setup.txt");

    let kzg_commitments = blobs.iter().map(|blob| 
        blob_to_kzg_commitment(blob, &ts)
        ).collect::<Vec<TG1>>();
    
    // Compute proof for these blobs

    let proof = compute_aggregate_kzg_proof(&blobs.concat(), &ts);

    // Verify proof

    assert!(verify_aggregate_kzg_proof(&blobs.concat(), &kzg_commitments, &proof), "verify failed");

    // Verification fails at wrong value

    blobs[0][0] = if blobs[0][0].equals(&TFr::zero()) {
        TFr::one()
    } else {
        TFr::zero()
    };

    assert!(!verify_aggregate_kzg_proof(&blobs.concat(), &kzg_commitments, &proof), "verify succeeded incorrectly");
}