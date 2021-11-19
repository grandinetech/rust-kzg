use criterion::Criterion;
use kzg::{FFTSettings, FK20SingleSettings, Fr, KZGSettings, Poly, G1, G2};

pub const SECRET: [u8; 32usize] = [0xa4, 0x73, 0x31, 0x95, 0x28, 0xc8, 0xb6, 0xea, 0x4d, 0x08, 0xcc,
    0x53, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

pub fn fk_single_da<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
    TFK20SingleSettings: FK20SingleSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TKZGSettings>
>(
    c: &mut Criterion,
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    for scale in 5..16 {
        let coeffs: Vec<u64> = vec![1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
        let poly_len: usize = coeffs.len();
        let n_len: usize = 1 << scale;
        let secrets_len = n_len + 1;

        assert!(n_len >= 2 * poly_len);
        let mut p = TPoly::new(poly_len).unwrap();
        for i in 0..poly_len {
            p.set_coeff_at(i, &TFr::from_u64(coeffs[i]));
        }

        // Initialise the secrets and data structures
        let (s1, s2) = generate_trusted_setup(secrets_len, SECRET);
        let fs = TFFTSettings::new(scale).unwrap();
        let ks = TKZGSettings::new(&s1, &s2, secrets_len, &fs).unwrap();
        let fk = TFK20SingleSettings::new(&ks, 2 * poly_len).unwrap();

        // Commit to the polynomial
        ks.commit_to_poly(&p).unwrap();

        // Generate the proofs
        let id = format!("bench_fk_single_commit_da scale: '{}'", scale);
        c.bench_function(&id, |b| b.iter(|| fk.data_availability(&p).unwrap()));
    }
}

pub fn fk_single_da_optimized<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
    TFK20SingleSettings: FK20SingleSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TKZGSettings>
>(
    c: &mut Criterion,
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    for scale in 5..16 {
        let coeffs: Vec<u64> = vec![1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
        let poly_len: usize = coeffs.len();
        let n_len: usize = 1 << scale;
        let secrets_len = n_len + 1;

        assert!(n_len >= 2 * poly_len);
        let mut p = TPoly::new(poly_len).unwrap();
        for i in 0..poly_len {
            p.set_coeff_at(i, &TFr::from_u64(coeffs[i]));
        }

        // Initialise the secrets and data structures
        let (s1, s2) = generate_trusted_setup(secrets_len, SECRET);
        let fs = TFFTSettings::new(scale).unwrap();
        let ks = TKZGSettings::new(&s1, &s2, secrets_len, &fs).unwrap();
        let fk = TFK20SingleSettings::new(&ks, 2 * poly_len).unwrap();

        // Commit to the polynomial
        ks.commit_to_poly(&p).unwrap();

        // Generate the proofs
        let id = format!("bench_fk_single_commit_da_optimized scale: '{}'", scale);
        c.bench_function(&id, |b| b.iter(|| fk.data_availability_optimized(&p).unwrap()));
    }
}
