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

fn reverse_bits_limited(length: usize, value: usize) -> usize {
    let unused_bits = length.leading_zeros();
    value.reverse_bits() >> unused_bits
}

pub fn reverse_bit_order<T>(vals: &mut [T])
where
    T: Clone,
{
    let unused_bit_len = vals.len().leading_zeros() + 1;
    for i in 0..vals.len() - 1 {
        let r = i.reverse_bits() >> unused_bit_len;
        if r > i {
            let tmp = vals[r].clone();
            vals[r] = vals[i].clone();
            vals[i] = tmp;
        }
    }
}

pub fn fk_single<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
    TFK20SingleSettings: FK20SingleSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TKZGSettings>,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    let coeffs: Vec<u64> = vec![1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
    let poly_len: usize = coeffs.len();
    let n: usize = 5;
    let n_len: usize = 1 << n;
    let secrets_len = n_len + 1;

    assert!(n_len >= 2 * poly_len);
    let mut p = TPoly::new(poly_len);
    for (i, &coeff) in coeffs.iter().enumerate() {
        p.set_coeff_at(i, &TFr::from_u64(coeff));
    }

    // Initialise the secrets and data structures
    let (s1, s2) = generate_trusted_setup(secrets_len, SECRET);
    let fs = TFFTSettings::new(n).unwrap();
    let ks = TKZGSettings::new(&s1, &s2, secrets_len, &fs).unwrap();
    let fk = TFK20SingleSettings::new(&ks, 2 * poly_len).unwrap();

    // Commit to the polynomial
    let commitment = ks.commit_to_poly(&p).unwrap();

    // 1. First with `da_using_fk20_single`

    // Generate the proofs
    let all_proofs = fk.data_availability(&p).unwrap();

    // Verify the proof at each root of unity
    for i in 0..(2 * poly_len) {
        let x = fs.get_expanded_roots_of_unity_at(i);
        let y = p.eval(&x);
        let proof = &all_proofs[reverse_bits_limited(2 * poly_len - 1, i)];
        assert!(ks.check_proof_single(&commitment, proof, &x, &y).unwrap());
    }

    // 2. Exactly the same thing again with `fk20_single_da_opt`

    // Generate the proofs
    let all_proofs = fk.data_availability_optimized(&p).unwrap();

    // Verify the proof at each root of unity
    for (i, proof) in all_proofs.iter().enumerate().take(2 * poly_len) {
        let x = fs.get_expanded_roots_of_unity_at(i);
        let y = p.eval(&x);
        assert!(ks.check_proof_single(&commitment, proof, &x, &y).unwrap());
    }
}

pub fn fk_single_strided<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
    TFK20SingleSettings: FK20SingleSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TKZGSettings>,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    let coeffs: Vec<u64> = vec![1, 2, 3, 4, 7, 7, 7, 7, 13, 13, 13, 13, 13, 13, 13, 13];
    let poly_len: usize = coeffs.len();
    let n: usize = 8;
    let n_len: usize = 1 << n;
    let stride: usize = n_len / (2 * poly_len);
    let secrets_len = n_len + 1;

    assert!(n_len >= 2 * poly_len);
    let mut p = TPoly::new(poly_len);
    for (i, &coeff) in coeffs.iter().enumerate() {
        p.set_coeff_at(i, &TFr::from_u64(coeff));
    }

    // Initialise the secrets and data structures
    let (s1, s2) = generate_trusted_setup(secrets_len, SECRET);
    let fs = TFFTSettings::new(n).unwrap();
    let ks = TKZGSettings::new(&s1, &s2, secrets_len, &fs).unwrap();
    let fk = TFK20SingleSettings::new(&ks, 2 * poly_len).unwrap();

    // Commit to the polynomial
    let commitment = ks.commit_to_poly(&p).unwrap();

    // Generate the proofs
    let all_proofs = fk.data_availability(&p).unwrap();

    // Verify the proof at each root of unity
    for i in 0..(2 * poly_len) {
        let x = fs.get_expanded_roots_of_unity_at(i * stride);
        let y = p.eval(&x);
        let proof = &all_proofs[reverse_bits_limited(2 * poly_len - 1, i)];
        assert!(ks.check_proof_single(&commitment, proof, &x, &y).unwrap());
    }
}

pub fn fk_multi_settings<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
    TFK20MultiSettings: FK20MultiSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TKZGSettings>,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    let n: usize = 5;
    let secrets_len: usize = 33;

    // Initialise the secrets and data structures
    let (s1, s2) = generate_trusted_setup(secrets_len, SECRET);
    let fs = TFFTSettings::new(n).unwrap();
    let ks = TKZGSettings::new(&s1, &s2, secrets_len, &fs).unwrap();
    let _fk = TFK20MultiSettings::new(&ks, 32, 4).unwrap();
}

fn fk_multi_case<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
    TFK20MultiSettings: FK20MultiSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TKZGSettings>,
>(
    chunk_len: usize,
    n: usize,
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
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
    let fk = TFK20MultiSettings::new(&ks, n * 2, chunk_len).unwrap();

    // Create a test polynomial of size n that's independent of chunk_len
    let mut p = TPoly::new(n);
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
    let commitment = ks.commit_to_poly(&p).unwrap();

    // Compute the multi proofs, assuming that the polynomial will be extended with zeros
    let all_proofs = fk.data_availability(&p).unwrap();

    // Now actually extend the polynomial with zeros
    let mut extended_coeffs = vec![TFr::zero(); 2 * n];
    for (i, extended_coeff) in extended_coeffs.iter_mut().enumerate().take(n) {
        *extended_coeff = p.get_coeff_at(i);
    }
    let mut extended_coeffs_fft = fs.fft_fr(&extended_coeffs, false).unwrap();
    reverse_bit_order(&mut extended_coeffs_fft);

    // Verify the proofs
    let mut ys = vec![TFr::default(); chunk_len];
    let mut ys2 = vec![TFr::default(); chunk_len];
    let domain_stride = fs.get_max_width() / (2 * n);
    for pos in 0..(2 * chunk_count) {
        let domain_pos = reverse_bits_limited(chunk_count, pos);
        let x = fs.get_expanded_roots_of_unity_at(domain_pos * domain_stride);

        // The ys from the extended coeffients
        for i in 0..chunk_len {
            ys[i] = extended_coeffs_fft[chunk_len * pos + i].clone();
        }
        reverse_bit_order(&mut ys);

        // Now recreate the ys by evaluating the polynomial in the sub-domain range
        let stride = fs.get_max_width() / chunk_len;
        for (i, ys2) in ys2.iter_mut().enumerate() {
            let z = x.mul(&fs.get_expanded_roots_of_unity_at(i * stride));
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

pub fn fk_multi_chunk_len_1_512<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
    TFK20MultiSettings: FK20MultiSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TKZGSettings>,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    fk_multi_case::<TFr, TG1, TG2, TPoly, TFFTSettings, TKZGSettings, TFK20MultiSettings>(
        1,
        512,
        &generate_trusted_setup,
    );
}

pub fn fk_multi_chunk_len_16_512<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
    TFK20MultiSettings: FK20MultiSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TKZGSettings>,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    fk_multi_case::<TFr, TG1, TG2, TPoly, TFFTSettings, TKZGSettings, TFK20MultiSettings>(
        16,
        512,
        &generate_trusted_setup,
    );
}

pub fn fk_multi_chunk_len_16_16<
    TFr: Fr,
    TG1: G1,
    TG2: G2,
    TPoly: Poly<TFr>,
    TFFTSettings: FFTSettings<TFr> + FFTFr<TFr>,
    TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>,
    TFK20MultiSettings: FK20MultiSettings<TFr, TG1, TG2, TFFTSettings, TPoly, TKZGSettings>,
>(
    generate_trusted_setup: &dyn Fn(usize, [u8; 32usize]) -> (Vec<TG1>, Vec<TG2>),
) {
    fk_multi_case::<TFr, TG1, TG2, TPoly, TFFTSettings, TKZGSettings, TFK20MultiSettings>(
        16,
        16,
        &generate_trusted_setup,
    );
}
