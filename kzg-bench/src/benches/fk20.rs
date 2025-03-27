use criterion::Criterion;
use kzg::{
    common_utils::{is_power_of_two, log2_pow2},
    EcBackend, FFTSettings, FK20MultiSettings, FK20SingleSettings, Fr, KZGSettings, Poly,
};
use rand::{thread_rng, RngCore};

pub const SECRET: [u8; 32usize] = [
    0xa4, 0x73, 0x31, 0x95, 0x28, 0xc8, 0xb6, 0xea, 0x4d, 0x08, 0xcc, 0x53, 0x18, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

const BENCH_SCALE: usize = 14;

#[allow(clippy::type_complexity)]
pub fn bench_fk_single_da<
    B: EcBackend,
    TFK20SingleSettings: FK20SingleSettings<
        B::Fr,
        B::G1,
        B::G2,
        B::FFTSettings,
        B::Poly,
        B::KZGSettings,
        B::G1Fp,
        B::G1Affine,
    >,
>(
    c: &mut Criterion,
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<B::G1>, Vec<B::G1>, Vec<B::G2>),
) {
    let mut rng = thread_rng();
    let coeffs: Vec<u64> = vec![rng.next_u64(); 1 << (BENCH_SCALE - 1)];
    let poly_len: usize = coeffs.len();
    let n_len: usize = 1 << BENCH_SCALE;
    let secrets_len = n_len;

    assert!(n_len >= 2 * poly_len);
    let mut p = B::Poly::new(poly_len);
    for (i, &coeff) in coeffs.iter().enumerate() {
        p.set_coeff_at(i, &B::Fr::from_u64(coeff));
    }

    // Initialise the secrets and data structures
    let (s1, s2, s3) = generate_trusted_setup(secrets_len, SECRET);
    let fs = B::FFTSettings::new(BENCH_SCALE).unwrap();
    let ks = B::KZGSettings::new(&s1, &s2, &s3, &fs, 16).unwrap();
    let fk = TFK20SingleSettings::new(&ks, 2 * poly_len).unwrap();

    // Commit to the polynomial
    ks.commit_to_poly(&p).unwrap();

    // Generate the proofs
    let id = format!("bench_fk_single_da scale: '{}'", BENCH_SCALE);
    c.bench_function(&id, |b| b.iter(|| fk.data_availability(&p).unwrap()));
}

#[allow(clippy::type_complexity)]
pub fn bench_fk_multi_da<
    B: EcBackend,
    TFK20MultiSettings: FK20MultiSettings<
        B::Fr,
        B::G1,
        B::G2,
        B::FFTSettings,
        B::Poly,
        B::KZGSettings,
        B::G1Fp,
        B::G1Affine,
    >,
>(
    c: &mut Criterion,
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<B::G1>, Vec<B::G1>, Vec<B::G2>),
) {
    let n = 1 << BENCH_SCALE;
    let chunk_len = 16;
    let vv: Vec<u64> = vec![1, 2, 3, 4, 7, 8, 9, 10, 13, 14, 1, 15, 1, 1000, 134, 33];

    assert!(is_power_of_two(n));
    assert!(is_power_of_two(chunk_len));
    assert_eq!(n % 16, 0);
    assert!(n >= chunk_len);

    let chunk_count: usize = n / chunk_len;
    let secrets_len: usize = 2 * n;
    let width: usize = log2_pow2(secrets_len);

    // Initialise the secrets and data structures
    let (s1, s2, s3) = generate_trusted_setup(secrets_len, SECRET);
    let fs = B::FFTSettings::new(width).unwrap();
    let ks = B::KZGSettings::new(&s1, &s2, &s3, &fs, 16).unwrap();
    let fk = TFK20MultiSettings::new(&ks, secrets_len, chunk_len).unwrap();

    // Create a test polynomial of size n that's independent of chunk_len
    let mut p = B::Poly::new(n);
    for i in 0..chunk_count {
        for j in 0..chunk_len {
            let p_index = i * chunk_len + j;
            let v_index = p_index % 16;
            let mut v = vv[v_index];
            let tmp: u64 = (i * chunk_len / 16) as u64;
            if v_index == 3 {
                v += tmp;
            }
            if v_index == 5 {
                v += tmp * tmp;
            }
            p.set_coeff_at(p_index, &B::Fr::from_u64(v));
            if v_index == 12 {
                p.set_coeff_at(p_index, &p.get_coeff_at(p_index).negate());
            }
            if v_index == 14 {
                p.set_coeff_at(p_index, &p.get_coeff_at(p_index).negate());
            }
        }
    }

    // Commit to the polynomial
    ks.commit_to_poly(&p).unwrap();

    let id = format!("bench_fk_multi_da scale: '{}'", BENCH_SCALE);
    c.bench_function(&id, |b| b.iter(|| fk.data_availability(&p).unwrap()));
}
