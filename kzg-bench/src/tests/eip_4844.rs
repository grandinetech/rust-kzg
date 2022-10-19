use std::{convert::TryInto};

use kzg::{Fr, KZGSettings, Poly, G1, G2, FFTSettings};
use rand::Rng;
use ssz_rs::{U256, Serialize, serialize};

// use sha256::digest;

// use hex_literal::hex;
use sha2::{Sha256, Sha512, Digest};

// Tests taken from https://github.com/dankrad/c-kzg/blob/4844/min-bindings/python/tests.py
pub fn bytes_to_bls_field_test<TFr: Fr>
(
    bytes_to_bls_field: &dyn Fn(&mut TFr, [u8; 32usize])
)
{
    let x: u64 = 329;
    let mut x_bls = TFr::default();
    
    let mut x_bytes: [u8; 32] = [0; 32];
    x_bytes[..8].copy_from_slice(&x.to_le_bytes());
    
    bytes_to_bls_field(&mut x_bls, x_bytes);
    
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
    compute_powers: &dyn Fn(&TFr, usize) -> Vec<TFr>,
    bytes_to_bls_field: &dyn Fn(&mut TFr, [u8; 32usize])
) 
{
    let x: u64 = 32930439;
    let n = 11;

    let mut x_bls = TFr::default();
    
    let mut x_bytes: [u8; 32] = [0; 32];
    x_bytes[..8].copy_from_slice(&x.to_le_bytes());
    
    bytes_to_bls_field(&mut x_bls, x_bytes);
    
    let powers = compute_powers(&x_bls, n);
    
    for (p, expected_p) in powers.iter().zip(EXPECTED_POWERS.iter()) {
        assert_eq!(expected_p, &p.to_u64_arr());        
    }
}

// ts = ckzg.load_trusted_setup("tiny_trusted_setup.txt")

// lvals = [239807672958224171024, 239807672958224171018,
//          3465144826073652318776269530687742778510060141723586134027,
//          52435875175126190475982595682112313518914282969839895044573213904131443392524]

// def int_to_bls_field(x):
//     return ckzg.bytes_to_bls_field(x.to_bytes(32, "little"))

// poly = ckzg.alloc_polynomial(tuple(map(int_to_bls_field, lvals)))

// y = ckzg.evaluate_polynomial_in_evaluation_form(poly, int_to_bls_field(2), ts)

// assert ckzg.int_from_bls_field(y) == 239807672958224171036

pub fn evaluate_polynomial_in_evaluation_form_test<TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>
>(
    evaluate_polynomial_in_evaluation_form: &dyn Fn(&mut TFr, &TPoly, &TFr, &TKZGSettings),
    bytes_to_bls_field: &dyn Fn(&mut TFr, [u8; 32usize]),
    load_trusted_setup: &dyn Fn(&str) -> TKZGSettings
)
{
    // let lvals: [u64; 4] = [239807672958224171024, 239807672958224171018,
    //                        3465144826073652318776269530687742778510060141723586134027,
    //                        52435875175126190475982595682112313518914282969839895044573213904131443392524];
    let lvals: [[u64; 4]; 4] = [
        [16, 13, 0, 0],
        [10, 13, 0, 0],
        [281474976710667, 17006436628450967565, 10183145419576640720, 0],
        [18446462594437873676, 7474466853796666379, 11954817552772682548, 8353516859464449351]
    ];

    let mut lvals_bls: TPoly = TPoly::new(lvals.len()).unwrap();
    for (i, lval) in lvals.iter().enumerate() {
        let mut lval_bls = TFr::default();
        let lval_bytes: [u8; 32] = lval.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>().try_into().unwrap();
        bytes_to_bls_field(&mut lval_bls, lval_bytes);
        lvals_bls.set_coeff_at(i, &lval_bls);
    }
    
    let x: u64 = 2;
    let mut x_bls = TFr::default();
    let mut x_bytes: [u8; 32] = [0; 32];
    x_bytes[..8].copy_from_slice(&x.to_le_bytes());
    bytes_to_bls_field(&mut x_bls, x_bytes);

    let mut y_bls = TFr::default();

    let ts = load_trusted_setup("tests/tiny_trusted_setup.txt");
    
    evaluate_polynomial_in_evaluation_form(&mut y_bls, &lvals_bls, &x_bls, &ts);
    
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
    bytes_to_bls_field: &dyn Fn(&mut TFr, [u8; 32usize]),
    blob_to_kzg_commitment: &dyn Fn(&mut TG1, &Vec<TFr>, &TKZGSettings),
    bytes_from_g1: &dyn Fn(&mut [u8; 48usize], &TG1),
    compute_powers: &dyn Fn(&TFr, usize) -> Vec<TFr>
)
{

    // Commit to a few random blobs
    const BLOB_SIZE: usize = 4096;
    const MAX_BLOBS_PER_BLOCK: usize = 16;

    // https://github.com/ethereum/py-ssz/blob/36f3406f814a5e5f4efb059a6928afc2d9d253b4/ssz/codec.py
    let mut blobs_sedes: ssz_rs::List<ssz_rs::Vector<U256, BLOB_SIZE>, MAX_BLOBS_PER_BLOCK> = ssz_rs::List::default();
    // nezinau ar gerai, pythone teste buvo 48 baitai
    let mut kzg_commitments_sedes:ssz_rs::List<ssz_rs::Vector<u8, 48>, MAX_BLOBS_PER_BLOCK> = ssz_rs::List::default();
    // println!("{}", blobs_sedes.capacity());
    // println!("{}", blobs_sedes.len());

    // current directory is not kzg-bench. Proof below:
    // let res = env::current_dir();
    // let res_str = match res {
    //     Ok(path) => path.into_os_string().into_string().unwrap(),
    //     Err(_) => "FAILED".to_string()
    // };
    // println!("{}", res_str);

    // nustatyti, kad randomai butu vienodi (t.y. seed'a nustatyti)
    let mut rng = rand::thread_rng();

    let mut blobs = Vec::new();

    for _ in 0..3{
        let mut vec = Vec::new();
        let mut vec_sedes: ssz_rs::Vector<U256, BLOB_SIZE> = ssz_rs::Vector::default();
        for _ in 0 .. BLOB_SIZE{
            let mut bytes: [u8; 32] = [0; 32];
            for j in 0..32{
                bytes[j] = rng.gen::<u8>();
            }
            let mut fr = Fr::default();
            bytes_to_bls_field(&mut fr, bytes);
            vec.push(fr);
            vec_sedes.push(ssz_rs::U256(bytes));
        }
        blobs_sedes.push(vec_sedes);
        blobs.push(vec);
    }

    let ts = load_trusted_setup("tests/trusted_setup.txt");
    
    let mut kzg_commitments:Vec<TG1> = Vec::default();
    for blob in blobs.iter(){
        let mut g1 : TG1 = TG1::default();
        blob_to_kzg_commitment(& mut g1, blob, &ts);
        kzg_commitments.push(g1);
    }

    for comm in kzg_commitments.iter(){
        let mut r: [u8; 48usize] = [0; 48];
        bytes_from_g1(& mut r, comm);
        let mut vec: ssz_rs::Vector<u8, 48> = ssz_rs::Vector::default();
        for u in r.iter(){
            vec.push(*u);
        }
        kzg_commitments_sedes.push(vec);
    }

    // let mut s: [u8; 32usize] = default();
        // // bytes_from_bls_field(s, )
        // // kzg_commitments_sedes.push()

    // patikrinti ar yra Ok
    let encoded_blobs = match serialize(&blobs_sedes){
        Ok(v) => v,
        _ => panic!()
    };
    let encoded_commitments = match serialize(&kzg_commitments_sedes){
        Ok(v) => v,
        _ => panic!()
    };

    // let input = "hello";
    // let val = digest(&*encoded_blobs);
    // let k = &*([encoded_blobs, encoded_commitments].concat());
    // let hashed = digest(&*([encoded_blobs, encoded_commitments].concat())).result();

    let mut hasher = Sha256::new();
    let k = &*([encoded_blobs, encoded_commitments].concat());
    hasher.update(k);
    let finalized = hasher.finalize();
    let hashed = finalized.as_slice();

    let mut r: TFr = TFr::default();
    bytes_to_bls_field(& mut r, hashed.try_into().expect("slice with incorrect length"));

    let r_powers = compute_powers(&r, blobs.len());
    // let values = vector_lincomb(blobs, r_powers, ??, ??);
    //  Compute polynomial commitments for these blobs
    // We don't follow the spec exactly to get the hash, but it shouldn't matter since it's random data

    // = blobs.iter().map(|&x| blob_to_kzg_commitment()).collect::<Vec<_>>();

    println!("pabaiga testo");
}