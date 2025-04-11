use kzg::{
    common_utils::{is_power_of_two, log2_pow2, reverse_bit_order, reverse_bits_limited},
    EcBackend, FFTFr, FFTSettings, FK20MultiSettings, FK20SingleSettings, Fr, KZGSettings, Poly,
};

pub const SECRET: [u8; 32usize] = [
    0xa4, 0x73, 0x31, 0x95, 0x28, 0xc8, 0xb6, 0xea, 0x4d, 0x08, 0xcc, 0x53, 0x18, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[allow(clippy::type_complexity)]
pub fn fk_single<
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
        B::G1ProjAddAffine,
    >,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<B::G1>, Vec<B::G1>, Vec<B::G2>),
) {
    let coeffs = [1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
    let poly_len: usize = coeffs.len();
    let n: usize = 5;
    let n_len: usize = 1 << n;
    let secrets_len = n_len;

    assert!(n_len >= 2 * poly_len);
    let mut p = B::Poly::new(poly_len);
    for (i, &coeff) in coeffs.iter().enumerate() {
        p.set_coeff_at(i, &B::Fr::from_u64(coeff));
    }

    // Initialise the secrets and data structures
    let (s1, s2, s3) = generate_trusted_setup(secrets_len, SECRET);
    let fs = B::FFTSettings::new(n).unwrap();
    let ks = B::KZGSettings::new(&s1, &s2, &s3, &fs, 4).unwrap();
    let fk = TFK20SingleSettings::new(&ks, 2 * poly_len).unwrap();

    // Commit to the polynomial
    let commitment = ks.commit_to_poly(&p).unwrap();

    // 1. First with `da_using_fk20_single`

    // Generate the proofs
    let all_proofs = fk.data_availability(&p).unwrap();

    // Verify the proof at each root of unity
    for i in 0..(2 * poly_len) {
        let x = fs.get_roots_of_unity_at(i);
        let y = p.eval(&x);
        let proof = &all_proofs[reverse_bits_limited(2 * poly_len, i)];
        assert!(ks.check_proof_single(&commitment, proof, &x, &y).unwrap());
    }

    // 2. Exactly the same thing again with `fk20_single_da_opt`

    // Generate the proofs
    let all_proofs = fk.data_availability_optimized(&p).unwrap();

    // Verify the proof at each root of unity
    for (i, proof) in all_proofs.iter().enumerate().take(2 * poly_len) {
        let x = fs.get_roots_of_unity_at(i);
        let y = p.eval(&x);
        assert!(ks.check_proof_single(&commitment, proof, &x, &y).unwrap());
    }
}

#[allow(clippy::type_complexity)]
pub fn fk_single_strided<
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
        B::G1ProjAddAffine,
    >,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<B::G1>, Vec<B::G1>, Vec<B::G2>),
) {
    let coeffs = [1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
    let poly_len: usize = coeffs.len();
    let n: usize = 8;
    let n_len: usize = 1 << n;
    let stride: usize = n_len / (2 * poly_len);
    let secrets_len = n_len;

    assert!(n_len >= 2 * poly_len);
    let mut p = B::Poly::new(poly_len);
    for (i, &coeff) in coeffs.iter().enumerate() {
        p.set_coeff_at(i, &B::Fr::from_u64(coeff));
    }

    // Initialise the secrets and data structures
    let (s1, s2, s3) = generate_trusted_setup(secrets_len, SECRET);
    let fs = B::FFTSettings::new(n).unwrap();
    let ks = B::KZGSettings::new(&s1, &s2, &s3, &fs, 16).unwrap();
    let fk = TFK20SingleSettings::new(&ks, 2 * poly_len).unwrap();

    // Commit to the polynomial
    let commitment = ks.commit_to_poly(&p).unwrap();

    // Generate the proofs
    let all_proofs = fk.data_availability(&p).unwrap();

    // Verify the proof at each root of unity
    for i in 0..(2 * poly_len) {
        let x = fs.get_roots_of_unity_at(i * stride);
        let y = p.eval(&x);
        let proof = &all_proofs[reverse_bits_limited(2 * poly_len, i)];
        assert!(ks.check_proof_single(&commitment, proof, &x, &y).unwrap());
    }
}

#[allow(clippy::type_complexity)]
pub fn fk_multi_settings<
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
        B::G1ProjAddAffine,
    >,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<B::G1>, Vec<B::G1>, Vec<B::G2>),
) {
    let n: usize = 5;
    let secrets_len: usize = 32;

    // Initialise the secrets and data structures
    let (s1, s2, s3) = generate_trusted_setup(secrets_len, SECRET);
    let fs = B::FFTSettings::new(n).unwrap();
    let ks = B::KZGSettings::new(&s1, &s2, &s3, &fs, 4).unwrap();
    let _fk = TFK20MultiSettings::new(&ks, 32, 4).unwrap();
}

#[allow(clippy::type_complexity)]
fn fk_multi_case<
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
        B::G1ProjAddAffine,
    >,
>(
    chunk_len: usize,
    n: usize,
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<B::G1>, Vec<B::G1>, Vec<B::G2>),
) where
    B::FFTSettings: FFTFr<B::Fr>,
{
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
    let ks = B::KZGSettings::new(&s1, &s2, &s3, &fs, chunk_len).unwrap();
    let fk = TFK20MultiSettings::new(&ks, n * 2, chunk_len).unwrap();

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
    let commitment = ks.commit_to_poly(&p).unwrap();

    // Compute the multi proofs, assuming that the polynomial will be extended with zeros
    let all_proofs = fk.data_availability(&p).unwrap();

    // Now actually extend the polynomial with zeros
    let mut extended_coeffs = vec![B::Fr::zero(); 2 * n];
    for (i, extended_coeff) in extended_coeffs.iter_mut().enumerate().take(n) {
        *extended_coeff = p.get_coeff_at(i);
    }
    let mut extended_coeffs_fft = fs.fft_fr(&extended_coeffs, false).unwrap();
    reverse_bit_order(&mut extended_coeffs_fft).unwrap();

    // Verify the proofs
    let mut ys = vec![B::Fr::default(); chunk_len];
    let mut ys2 = vec![B::Fr::default(); chunk_len];
    let domain_stride = fs.get_max_width() / (2 * n);
    for pos in 0..(2 * chunk_count) {
        let domain_pos = reverse_bits_limited(2 * chunk_count, pos);
        let x = fs.get_roots_of_unity_at(domain_pos * domain_stride);

        // The ys from the extended coeffients
        for i in 0..chunk_len {
            ys[i] = extended_coeffs_fft[chunk_len * pos + i].clone();
        }
        reverse_bit_order(&mut ys).unwrap();

        // Now recreate the ys by evaluating the polynomial in the sub-domain range
        let stride = fs.get_max_width() / chunk_len;
        for (i, ys2) in ys2.iter_mut().enumerate() {
            let z = x.mul(&fs.get_roots_of_unity_at(i * stride));
            *ys2 = p.eval(&z);
        }

        // ys and ys2 should be equal
        for (ys, ys2) in ys.iter().zip(&ys2) {
            assert!(ys.equals(ys2));
        }

        // Verify this proof
        let result = ks
            .check_proof_multi(&commitment, &all_proofs[pos], &x, &ys, chunk_len)
            .unwrap();
        assert!(result);
    }
}

#[allow(clippy::type_complexity)]
pub fn fk_multi_chunk_len_1_512<
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
        B::G1ProjAddAffine,
    >,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<B::G1>, Vec<B::G1>, Vec<B::G2>),
) {
    fk_multi_case::<B, TFK20MultiSettings>(1, 512, &generate_trusted_setup);
}

#[allow(clippy::type_complexity)]
pub fn fk_multi_chunk_len_16_512<
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
        B::G1ProjAddAffine,
    >,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<B::G1>, Vec<B::G1>, Vec<B::G2>),
) {
    fk_multi_case::<B, TFK20MultiSettings>(16, 512, &generate_trusted_setup);
}

#[allow(clippy::type_complexity)]
pub fn fk_multi_chunk_len_16_16<
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
        B::G1ProjAddAffine,
    >,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<B::G1>, Vec<B::G1>, Vec<B::G2>),
) {
    fk_multi_case::<B, TFK20MultiSettings>(16, 16, &generate_trusted_setup);
}
