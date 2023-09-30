use kzg::{FFTSettings, Fr, KZGSettings, Poly, G1, G2};

pub const SECRET: [u8; 32usize] = [
    0xa4, 0x73, 0x31, 0x95, 0x28, 0xc8, 0xb6, 0xea, 0x4d, 0x08, 0xcc, 0x53, 0x18, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Check that both FFT implementations produce the same results
pub fn proof_single<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    // Our polynomial: degree 15, 16 coefficients
    let coeffs = [1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
    let poly_len = coeffs.len();
    let secrets_len = poly_len + 1;

    // Create the polynomial
    let mut p = TPoly::new(poly_len);
    for (x, &coeff) in coeffs.iter().enumerate() {
        p.set_coeff_at(x, &TFr::from_u64(coeff));
    }

    // Initialise the secrets and data structures
    let (s1, s2) = generate_trusted_setup(secrets_len, SECRET);
    let fs = TFFTSettings::new(4).unwrap();
    let ks = TKZGSettings::new(&s1, &s2, secrets_len, &fs).unwrap();

    // Compute the proof for x = 25
    let x = TFr::from_u64(25);
    let commitment = ks.commit_to_poly(&p).unwrap();
    let proof = ks.compute_proof_single(&p, &x).unwrap();
    let mut value = p.eval(&x);

    // Verify the proof that the (unknown) polynomial has y = value at x = 25
    assert!(ks
        .check_proof_single(&commitment, &proof, &x, &value)
        .unwrap());

    // Change the value and check that the proof fails
    value = value.add(&TFr::one());
    assert!(!ks
        .check_proof_single(&commitment, &proof, &x, &value)
        .unwrap());
}

pub fn commit_to_nil_poly<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    {
        let secrets_len = 16;

        // Initialise the (arbitrary) secrets and data structures
        let (s1, s2) = generate_trusted_setup(secrets_len, SECRET);
        let fs = TFFTSettings::new(4).unwrap();
        let ks = TKZGSettings::new(&s1, &s2, secrets_len, &fs).unwrap();

        let a = TPoly::new(0);
        let result = ks.commit_to_poly(&a).unwrap();
        assert!(result.equals(&TG1::default()));
    }
}

//Test is made to panic so put this under #[test]
//#[should_panic(expected = "Poly given is too long")]
pub fn commit_to_too_long_poly<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    {
        let secrets_len = 16;
        let poly_len = 32; // poly is longer than secrets!

        // Initialise the (arbitrary) secrets and data structures
        let (s1, s2) = generate_trusted_setup(secrets_len, SECRET);
        let fs = TFFTSettings::new(4).unwrap();
        let ks = TKZGSettings::new(&s1, &s2, secrets_len, &fs).unwrap();

        let a = TPoly::new(poly_len);
        let _result = ks.commit_to_poly(&a);
    }
}

// Instead of panicking, commit should return an err
pub fn commit_to_too_long_poly_returns_err<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    let secrets_len = 16;
    let poly_len = 32; // poly is longer than secrets!

    // Initialise the (arbitrary) secrets and data structures
    let (s1, s2) = generate_trusted_setup(secrets_len, SECRET);
    let fs = TFFTSettings::new(4).unwrap();
    let ks = TKZGSettings::new(&s1, &s2, secrets_len, &fs).unwrap();

    let a = TPoly::new(poly_len);
    let _result = ks.commit_to_poly(&a);
    assert!(_result.is_err());
}

//It was not verified that this test works, use with caution
pub fn proof_multi<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    // Our polynomial: degree 15, 16 coefficients
    let coeffs = [1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
    let poly_len = coeffs.len();

    // Compute proof at 2^coset_scale points
    let coset_scale = 3;
    let coset_len = 1 << coset_scale;
    let mut y: Vec<TFr> = Vec::new();

    let secrets_len = if poly_len > coset_len {
        poly_len + 1
    } else {
        coset_len + 1
    };

    // Create the polynomial
    let mut p = TPoly::new(poly_len);
    for (x, &coeff) in coeffs.iter().enumerate() {
        p.set_coeff_at(x, &TFr::from_u64(coeff));
    }

    // Initialise the secrets and data structures
    let (s1, s2) = generate_trusted_setup(secrets_len, SECRET);
    let fs1 = TFFTSettings::new(4).unwrap();
    let ks1 = TKZGSettings::new(&s1, &s2, secrets_len, &fs1).unwrap();

    // Commit to the polynomial
    let commitment = ks1.commit_to_poly(&p).unwrap();

    let fs2 = TFFTSettings::new(coset_scale).unwrap();
    let ks2 = TKZGSettings::new(&s1, &s2, secrets_len, &fs2).unwrap();

    // Compute proof at the points [x * root_i] 0 <= i < coset_len
    let x = TFr::from_u64(5431);
    let proof = ks2.compute_proof_multi(&p, &x, coset_len).unwrap();

    // y_i is the value of the polynomial at each x_i
    for i in 0..coset_len {
        let tmp = TFr::mul(&x, &ks2.get_expanded_roots_of_unity_at(i));
        y.push(p.eval(&tmp));
    }

    // Verify the proof that the (unknown) polynomial has value y_i at x_i
    let result = ks2
        .check_proof_multi(&commitment, &proof, &x, &y, coset_len)
        .unwrap();
    assert!(result);

    // Change a value and check that the proof fails
    let temp = TFr::add(&y[4], &TFr::one());
    let _temp = std::mem::replace(&mut y[4], temp);
    let result = ks2
        .check_proof_multi(&commitment, &proof, &x, &y, coset_len)
        .unwrap();
    assert!(!result);
}
