use kzg::{
    EcBackend, FFTSettings, Fr, G1Affine, G1Fp, G1GetFp, G1Mul, KZGSettings, Poly, Preset, G1, G2,
};

pub const SECRET: [u8; 32usize] = [
    0xa4, 0x73, 0x31, 0x95, 0x28, 0xc8, 0xb6, 0xea, 0x4d, 0x08, 0xcc, 0x53, 0x18, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

struct TestPreset1;

impl Preset for TestPreset1 {
    const FIELD_ELEMENTS_PER_BLOB: usize = 17;
    const FIELD_ELEMENTS_PER_EXT_BLOB: usize = 34;
    const CELLS_PER_EXT_BLOB: usize = 1;
}

/// Check that both FFT implementations produce the same results
#[allow(clippy::type_complexity)]
pub fn proof_single<B: EcBackend>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<B::G1>, Vec<B::G1>, Vec<B::G2>),
) {
    // Our polynomial: degree 15, 16 coefficients
    let coeffs = [1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
    let poly_len = coeffs.len();
    let secrets_len = poly_len + 1;

    // Create the polynomial
    let mut p = B::Poly::new(poly_len);
    for (x, &coeff) in coeffs.iter().enumerate() {
        p.set_coeff_at(x, &B::Fr::from_u64(coeff));
    }

    // Initialise the secrets and data structures
    let (s1, s2, s3) = generate_trusted_setup(secrets_len, SECRET);
    let fs = B::FFTSettings::new(4).unwrap();
    let ks = B::KZGSettings::new_for_preset::<7, TestPreset1>(&s1, &s2, &s3, &fs).unwrap();

    // Compute the proof for x = 25
    let x = B::Fr::from_u64(25);
    let commitment = ks.commit_to_poly(&p).unwrap();
    let proof = ks.compute_proof_single(&p, &x).unwrap();
    let mut value = p.eval(&x);

    // Verify the proof that the (unknown) polynomial has y = value at x = 25
    assert!(ks
        .check_proof_single(&commitment, &proof, &x, &value)
        .unwrap());

    // Change the value and check that the proof fails
    value = value.add(&B::Fr::one());
    assert!(!ks
        .check_proof_single(&commitment, &proof, &x, &value)
        .unwrap());
}

struct TestPreset2;

impl Preset for TestPreset2 {
    const FIELD_ELEMENTS_PER_BLOB: usize = 16;
    const FIELD_ELEMENTS_PER_EXT_BLOB: usize = 32;
    const CELLS_PER_EXT_BLOB: usize = 4;
}

#[allow(clippy::type_complexity)]
pub fn commit_to_nil_poly<B: EcBackend>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<B::G1>, Vec<B::G1>, Vec<B::G2>),
) {
    {
        let secrets_len = 16;

        // Initialise the (arbitrary) secrets and data structures
        let (s1, s2, s3) = generate_trusted_setup(secrets_len, SECRET);
        let fs = B::FFTSettings::new(4).unwrap();
        let ks = B::KZGSettings::new_for_preset::<8, TestPreset2>(&s1, &s2, &s3, &fs).unwrap();

        let a = B::Poly::new(0);
        let result = ks.commit_to_poly(&a).unwrap();
        assert!(result.equals(&B::G1::default()));
    }
}

//Test is made to panic so put this under #[test]
//#[should_panic(expected = "Poly given is too long")]
#[allow(clippy::type_complexity)]
pub fn commit_to_too_long_poly<
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TG1Fp, TG1Affine>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG1>, Vec<TG2>),
) {
    {
        let secrets_len = 16;
        let poly_len = 32; // poly is longer than secrets!

        // Initialise the (arbitrary) secrets and data structures
        let (s1, s2, s3) = generate_trusted_setup(secrets_len, SECRET);
        let fs = TFFTSettings::new(4).unwrap();
        let ks = TKZGSettings::new(&s1, &s2, &s3, &fs).unwrap();

        let a = TPoly::new(poly_len);
        let _result = ks.commit_to_poly(&a);
    }
}

// Instead of panicking, commit should return an err
#[allow(clippy::type_complexity)]
pub fn commit_to_too_long_poly_returns_err<B: EcBackend>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<B::G1>, Vec<B::G1>, Vec<B::G2>),
) {
    let secrets_len = 16;
    let poly_len = 32; // poly is longer than secrets!

    // Initialise the (arbitrary) secrets and data structures
    let (s1, s2, s3) = generate_trusted_setup(secrets_len, SECRET);
    let fs = B::FFTSettings::new(4).unwrap();
    let ks = B::KZGSettings::new_for_preset::<8, TestPreset2>(&s1, &s2, &s3, &fs).unwrap();

    let a = B::Poly::new(poly_len);
    let _result = ks.commit_to_poly(&a);
    assert!(_result.is_err());
}

//It was not verified that this test works, use with caution
#[allow(clippy::type_complexity)]
pub fn proof_multi<B: EcBackend>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<B::G1>, Vec<B::G1>, Vec<B::G2>),
) {
    // Our polynomial: degree 15, 16 coefficients
    let coeffs = [1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
    let poly_len = coeffs.len();

    // Compute proof at 2^coset_scale points
    let coset_scale = 3;
    let coset_len = 1 << coset_scale;
    let mut y: Vec<B::Fr> = Vec::new();

    let secrets_len = if poly_len > coset_len {
        poly_len + 1
    } else {
        coset_len + 1
    };

    // Create the polynomial
    let mut p = B::Poly::new(poly_len);
    for (x, &coeff) in coeffs.iter().enumerate() {
        p.set_coeff_at(x, &B::Fr::from_u64(coeff));
    }

    // Initialise the secrets and data structures
    let (s1, s2, s3) = generate_trusted_setup(secrets_len, SECRET);
    let fs1 = B::FFTSettings::new(4).unwrap();
    let ks1 = B::KZGSettings::new_for_preset::<7, TestPreset1>(&s1, &s2, &s3, &fs1).unwrap();

    // Commit to the polynomial
    let commitment = ks1.commit_to_poly(&p).unwrap();

    let fs2 = B::FFTSettings::new(coset_scale).unwrap();
    let ks2 = B::KZGSettings::new_for_preset::<7, TestPreset1>(&s1, &s2, &s3, &fs2).unwrap();

    // Compute proof at the points [x * root_i] 0 <= i < coset_len
    let x = B::Fr::from_u64(5431);
    let proof = ks2.compute_proof_multi(&p, &x, coset_len).unwrap();

    // y_i is the value of the polynomial at each x_i
    for i in 0..coset_len {
        let tmp = B::Fr::mul(&x, &ks2.get_roots_of_unity_at(i));
        y.push(p.eval(&tmp));
    }

    // Verify the proof that the (unknown) polynomial has value y_i at x_i
    let result = ks2
        .check_proof_multi(&commitment, &proof, &x, &y, coset_len)
        .unwrap();
    assert!(result);

    // Change a value and check that the proof fails
    let temp = B::Fr::add(&y[4], &B::Fr::one());
    let _temp = std::mem::replace(&mut y[4], temp);
    let result = ks2
        .check_proof_multi(&commitment, &proof, &x, &y, coset_len)
        .unwrap();
    assert!(!result);
}
