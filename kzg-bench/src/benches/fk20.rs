use criterion::Criterion;
use kzg::{
    FFTFr, FFTSettings, FK20MultiSettings, FK20SingleSettings, Fr, KZGSettings, Poly, G1, G2,
};

pub const SECRET: [u8; 32usize] = [
    0xa4, 0x73, 0x31, 0x95, 0x28, 0xc8, 0xb6, 0xea, 0x4d, 0x08, 0xcc, 0x53, 0x18, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

fn is_power_of_two(n: usize) -> bool {
    n & (n - 1) == 0
}

fn log2_pow2(n: u32) -> usize {
    let b: [u32; 5] = [0xaaaaaaaa, 0xcccccccc, 0xf0f0f0f0, 0xff00ff00, 0xffff0000];
    let mut r: u32 = u32::from((n & b[0]) != 0);
    r |= u32::from((n & b[1]) != 0) << 1;
    r |= u32::from((n & b[2]) != 0) << 2;
    r |= u32::from((n & b[3]) != 0) << 3;
    r |= u32::from((n & b[4]) != 0) << 4;
    r as usize
}

pub fn fk_single_da<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
    TFK20SingleSettings: FK20SingleSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TKZGSettings>,
>(
    c: &mut Criterion,
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    for scale in 5..12 {
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
    TFK20SingleSettings: FK20SingleSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TKZGSettings>,
>(
    c: &mut Criterion,
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    for scale in 5..12 {
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
        c.bench_function(&id, |b| {
            b.iter(|| fk.data_availability_optimized(&p).unwrap())
        });
    }
}

fn fk_multi_da<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
    TFK20MultiSettings: FK20MultiSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TKZGSettings>,
>(
    chunk_len: usize,
    test_name: String,
    c: &mut Criterion,
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
    optimized: bool,
) {
    for i in 5..8 {
        let n = 1 << i;
        let vv: Vec<u64> = vec![1, 2, 3, 4, 7, 8, 9, 10, 13, 14, 1, 15, 1, 1000, 134, 33];

        assert!(is_power_of_two(n));
        assert!(is_power_of_two(chunk_len));
        assert_eq!(n % 16, 0);
        assert!(n >= chunk_len);

        let chunk_count: usize = n / chunk_len;
        let secrets_len: usize = 2 * n;
        let width: usize = log2_pow2(secrets_len as u32);

        // Initialise the secrets and data structures
        let (s1, s2) = generate_trusted_setup(secrets_len, SECRET);
        let fs = TFFTSettings::new(width).unwrap();
        let ks = TKZGSettings::new(&s1, &s2, secrets_len, &fs).unwrap();
        let fk = TFK20MultiSettings::new(&ks, secrets_len, chunk_len).unwrap();

        // Create a test polynomial of size n that's independent of chunk_len
        let mut p = TPoly::new(n).unwrap();
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
                p.set_coeff_at(p_index, &TFr::from_u64(v));
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

        // Compute the multi proofs, assuming that the polynomial will be extended with zeros
        let id = format!("{} scale: '{}'", test_name, width);

        if optimized == true {
            c.bench_function(&id, |b| {
                b.iter(|| fk.data_availability_optimized(&p).unwrap())
            });
        } else {
            c.bench_function(&id, |b| b.iter(|| fk.data_availability(&p).unwrap()));
        }
    }
}

pub fn fk_multi_da_chunk_32<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
    TFK20MultiSettings: FK20MultiSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TKZGSettings>,
>(
    c: &mut Criterion,
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    fk_multi_da::<TFr, TG1, TG2, TPoly, TFFTSettings, TKZGSettings, TFK20MultiSettings>(
        32,
        "bench_fk_multi_commit_da_chunk_32".to_string(),
        c,
        generate_trusted_setup,
        false,
    );
}

pub fn fk_multi_da_chunk_32_optimized<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
    TFK20MultiSettings: FK20MultiSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TKZGSettings>,
>(
    c: &mut Criterion,
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    fk_multi_da::<TFr, TG1, TG2, TPoly, TFFTSettings, TKZGSettings, TFK20MultiSettings>(
        32,
        "bench_fk_multi_commit_da_chunk_32_optimized".to_string(),
        c,
        generate_trusted_setup,
        false,
    );
}
