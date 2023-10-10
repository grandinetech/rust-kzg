use crate::{Fr, G1, eip_4844::{bytes_of_uint64, CHALLENGE_INPUT_SIZE, FIAT_SHAMIR_PROTOCOL_DOMAIN, FIELD_ELEMENTS_PER_BLOB, BYTES_PER_FIELD_ELEMENT, BYTES_PER_BLOB, hash, BYTES_PER_COMMITMENT, BYTES_PER_PROOF, RANDOM_CHALLENGE_KZG_BATCH_DOMAIN, BYTES_PER_G1, BYTES_PER_G2, TRUSTED_SETUP_NUM_G2_POINTS}, cfg_into_iter, KZGSettings, Poly, FFTSettings, G2, PairingVerify};

pub fn reverse_bit_order<T>(vals: &mut [T]) -> Result<(), String>
where
    T: Clone,
{
    if vals.is_empty() {
        return Err(String::from("Values can not be empty"));
    }

    // required for tests
    if vals.len() == 1 {
        return Ok(());
    }

    if !vals.len().is_power_of_two() {
        return Err(String::from("Values length has to be a power of 2"));
    }

    let unused_bit_len = vals.len().leading_zeros() + 1;
    for i in 0..vals.len() - 1 {
        let r = i.reverse_bits() >> unused_bit_len;
        if r > i {
            let tmp = vals[r].clone();
            vals[r] = vals[i].clone();
            vals[i] = tmp;
        }
    }

    Ok(())
}

pub fn log_2_byte(b: u8) -> usize {
    let mut r = u8::from(b > 0xF) << 2;
    let mut b = b >> r;
    let shift = u8::from(b > 0x3) << 1;
    b >>= shift + 1;
    r |= shift | b;
    r.into()
}

pub fn log2_pow2(n: usize) -> usize {
    let bytes: [usize; 5] = [0xAAAAAAAA, 0xCCCCCCCC, 0xF0F0F0F0, 0xFF00FF00, 0xFFFF0000];
    let mut r: usize = usize::from((n & bytes[0]) != 0);
    r |= usize::from((n & bytes[1]) != 0) << 1;
    r |= usize::from((n & bytes[2]) != 0) << 2;
    r |= usize::from((n & bytes[3]) != 0) << 3;
    r |= usize::from((n & bytes[4]) != 0) << 4;
    r
}

pub fn log2_u64(n: usize) -> usize {
    let mut n2 = n;
    let mut r: usize = 0;
    while (n2 >> 1) != 0 {
        n2 >>= 1;
        r += 1;
    }
    r
}

pub const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

pub fn log_2(x: usize) -> usize {
    if x == 0 {
        return 0;
    }
    num_bits::<usize>() as usize - (x.leading_zeros() as usize) - 1
}

pub fn is_power_of_2(n: usize) -> bool {
    n & (n - 1) == 0
}

pub fn next_pow_of_2(x: usize) -> usize {
    if x == 0 {
        return 1;
    }
    if is_power_of_2(x) {
        return x;
    }
    1 << (log_2(x) + 1)
}

pub fn is_power_of_two(n: usize) -> bool {
    n & (n - 1) == 0
}

pub fn reverse_bits_limited(length: usize, value: usize) -> usize {
    let unused_bits = length.leading_zeros();
    value.reverse_bits() >> unused_bits
}

////////// Generic functions

#[allow(clippy::useless_conversion)]
pub fn bytes_to_blob<TFr: Fr>(bytes: &[u8]) -> Result<Vec<TFr>, String> {
    if bytes.len() != BYTES_PER_BLOB {
        return Err(format!(
            "Invalid byte length. Expected {} got {}",
            BYTES_PER_BLOB,
            bytes.len(),
        ));
    }

    bytes
        .chunks(BYTES_PER_FIELD_ELEMENT)
        .map(TFr::from_bytes)
        .collect()
}

pub fn fr_batch_inv<TFr: Fr + PartialEq + Copy>(out: &mut [TFr], a: &[TFr], len: usize) -> Result<(), String> {
    if len == 0 {
        return Err(String::from("Length is less than 0."));
    }

    if a == out {
        return Err(String::from("Destination is the same as source."));
    }

    let mut accumulator = TFr::one();

    for i in 0..len {
        out[i] = accumulator;
        accumulator = accumulator.mul(&a[i]);
    }

    if accumulator.is_zero() {
        return Err(String::from("Zero input"));
    }

    accumulator = accumulator.eucl_inverse();

    for i in (0..len).rev() {
        out[i] = out[i].mul(&accumulator);
        accumulator = accumulator.mul(&a[i]);
    }

    Ok(())
}

pub fn compute_powers<TFr: Fr>(base: &TFr, num_powers: usize) -> Vec<TFr> {
    let mut powers: Vec<TFr> = vec![TFr::default(); num_powers];
    if num_powers == 0 {
        return powers;
    }
    powers[0] = TFr::one();
    for i in 1..num_powers {
        powers[i] = powers[i - 1].mul(base);
    }
    powers
}

pub fn hash_to_bls_field<TFr: Fr>(x: &[u8; BYTES_PER_FIELD_ELEMENT]) -> TFr {
    TFr::from_bytes_unchecked(x).unwrap()
}

pub fn compute_challenge<TFr: Fr, TG1: G1>(blob: &[TFr], commitment: &TG1) -> Result<TFr, String> {
    let mut bytes: Vec<u8> = vec![0; CHALLENGE_INPUT_SIZE];

    // Copy domain separator
    bytes[..16].copy_from_slice(&FIAT_SHAMIR_PROTOCOL_DOMAIN);
    // Set all other bytes of this 16-byte (big-endian) field to zero
    bytes_of_uint64(&mut bytes[16..24], 0);
    bytes_of_uint64(&mut bytes[24..32], FIELD_ELEMENTS_PER_BLOB as u64);

    for (i, _) in blob.iter().enumerate() {
        let v = blob[i].to_bytes();
        let size = (32 + i * BYTES_PER_FIELD_ELEMENT)..(32 + (i + 1) * BYTES_PER_FIELD_ELEMENT);

        bytes[size].copy_from_slice(&v);
    }

    // Copy commitment
    let v = commitment.to_bytes();
    for i in 0..v.len() {
        bytes[32 + BYTES_PER_BLOB + i] = v[i];
    }

    // Now let's create the challenge!
    let eval_challenge = hash(&bytes);
    Ok(hash_to_bls_field(&eval_challenge))
}

pub fn compute_r_powers<TG1: G1, TFr: Fr>(
    commitments_g1: &[TG1],
    zs_fr: &[TFr],
    ys_fr: &[TFr],
    proofs_g1: &[TG1],
) -> Result<Vec<TFr>, String> {
    let n = commitments_g1.len();
    let input_size =
        32 + n * (BYTES_PER_COMMITMENT + 2 * BYTES_PER_FIELD_ELEMENT + BYTES_PER_PROOF);

    #[allow(unused_assignments)]
    let mut offset = 0;
    let mut bytes: Vec<u8> = vec![0; input_size];

    // Copy domain separator
    bytes[..16].copy_from_slice(&RANDOM_CHALLENGE_KZG_BATCH_DOMAIN);
    bytes_of_uint64(&mut bytes[16..24], FIELD_ELEMENTS_PER_BLOB as u64);
    bytes_of_uint64(&mut bytes[24..32], n as u64);
    offset = 32;

    for i in 0..n {
        // Copy commitment
        let v = commitments_g1[i].to_bytes();
        bytes[offset..(v.len() + offset)].copy_from_slice(&v[..]);
        offset += BYTES_PER_COMMITMENT;

        // Copy evaluation challenge
        let v = zs_fr[i].to_bytes();
        bytes[offset..(v.len() + offset)].copy_from_slice(&v[..]);
        offset += BYTES_PER_FIELD_ELEMENT;

        // Copy polynomial's evaluation value
        let v = ys_fr[i].to_bytes();
        bytes[offset..(v.len() + offset)].copy_from_slice(&v[..]);
        offset += BYTES_PER_FIELD_ELEMENT;

        // Copy proof
        let v = proofs_g1[i].to_bytes();
        bytes[offset..(v.len() + offset)].copy_from_slice(&v[..]);
        offset += BYTES_PER_PROOF;
    }

    // Make sure we wrote the entire buffer
    if offset != input_size {
        return Err(String::from("Error while copying commitments"));
    }

    // Now let's create the challenge!
    let eval_challenge = hash(&bytes);
    let r = hash_to_bls_field(&eval_challenge);

    Ok(compute_powers(&r, n))
}

pub fn validate_batched_input<TG1: G1>(commitments: &[TG1], proofs: &[TG1]) -> Result<(), String> {
    let invalid_commitment = cfg_into_iter!(commitments).any(|commitment| !commitment.is_valid());
    let invalid_proof = cfg_into_iter!(proofs).any(|proof| !proof.is_valid());

    if invalid_commitment {
        return Err("Invalid commitment".to_string());
    }
    if invalid_proof {
        return Err("Invalid proof".to_string());
    }

    Ok(())
}

pub fn blob_to_polynomial<TFr: Fr, TPoly: Poly<TFr>>(blob: &[TFr]) -> Result<TPoly, String> {
    if blob.len() != FIELD_ELEMENTS_PER_BLOB {
        return Err(String::from("Blob length must be FIELD_ELEMENTS_PER_BLOB"));
    }
    Ok(TPoly::from_coeffs(blob))
}

pub fn evaluate_polynomial_in_evaluation_form<TG1: G1, TG2: G2, TFr: Fr + Copy, TPoly: Poly<TFr>, TFFTSettings: FFTSettings<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(
    p: &TPoly,
    x: &TFr,
    s: &TKZGSettings,
) -> Result<TFr, String> {
    if p.len() != FIELD_ELEMENTS_PER_BLOB {
        return Err(String::from("Incorrect field elements count."));
    }

    let mut inverses_in: Vec<TFr> = vec![TFr::default(); FIELD_ELEMENTS_PER_BLOB];
    let mut inverses: Vec<TFr> = vec![TFr::default(); FIELD_ELEMENTS_PER_BLOB];

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        if x.equals(&s.get_roots_of_unity_at(i)) {
            return Ok(p.get_coeff_at(i));
        }
        inverses_in[i] = x.sub(&s.get_roots_of_unity_at(i));
    }

    fr_batch_inv(&mut inverses, &inverses_in, FIELD_ELEMENTS_PER_BLOB)?;

    let mut tmp: TFr;
    let mut out = TFr::zero();

    for i in 0..FIELD_ELEMENTS_PER_BLOB {
        tmp = inverses[i].mul(&s.get_roots_of_unity_at(i));
        tmp = tmp.mul(&p.get_coeff_at(i));
        out = out.add(&tmp);
    }

    tmp = TFr::from_u64(FIELD_ELEMENTS_PER_BLOB as u64);
    out = match out.div(&tmp) {
        Ok(value) => value,
        Err(err) => return Err(err),
    };
    tmp = x.pow(FIELD_ELEMENTS_PER_BLOB);
    tmp = tmp.sub(&TFr::one());
    out = out.mul(&tmp);
    Ok(out)
}

fn is_trusted_setup_in_lagrange_form<TG1: G1 + PairingVerify<TG1, TG2>, TG2: G2>(g1_values: &Vec<TG1>, g2_values: &Vec<TG2>) -> bool {
    if g1_values.len() < 2 || g2_values.len() < 2 {
        return false;
    }

    let is_monotomial_form = TG1::verify(&g1_values[1], &g2_values[0], &g1_values[0], &g2_values[1]);
    !is_monotomial_form
}

#[allow(clippy::useless_conversion)]
pub fn load_trusted_setup_rust<TFr: Fr, TG1: G1 + PairingVerify<TG1, TG2>, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(g1_bytes: &[u8], g2_bytes: &[u8]) -> Result<TKZGSettings, String> {
    let num_g1_points = g1_bytes.len() / BYTES_PER_G1;
    if num_g1_points != FIELD_ELEMENTS_PER_BLOB {
        return Err(String::from("Invalid number of G1 points"));
    }

    let num_g2_points = g2_bytes.len() / BYTES_PER_G2;
    if num_g2_points != TRUSTED_SETUP_NUM_G2_POINTS {
        return Err(String::from("Invalid number of G2 points"));
    }

    let mut g1_values = g1_bytes
        .chunks(BYTES_PER_G1)
        .map(TG1::from_bytes)
        .collect::<Result<Vec<TG1>, String>>()?;

    let g2_values = g2_bytes
        .chunks(BYTES_PER_G2)
        .map(TG2::from_bytes)
        .collect::<Result<Vec<TG2>, String>>()?;

    // Sanity check, that user is not trying to load old trusted setup file
    if !is_trusted_setup_in_lagrange_form::<TG1, TG2>(&g1_values, &g2_values) {
        return Err(String::from("Trusted setup is not in Lagrange form"));
    }

    let mut max_scale: usize = 0;
    while (1 << max_scale) < num_g1_points {
        max_scale += 1;
    }

    let fs = TFFTSettings::new(max_scale)?;
    reverse_bit_order(&mut g1_values)?;
    TKZGSettings::new(g1_values.as_slice(), &g2_values.as_slice(), max_scale, &fs)
}

pub fn verify_kzg_proof_rust<TFr: Fr, TG1: G1, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(
    commitment: &TG1,
    z: &TFr,
    y: &TFr,
    proof: &TG1,
    s: &TKZGSettings,
) -> Result<bool, String> {
    if !commitment.is_valid() {
        return Err("Invalid commitment".to_string());
    }
    if !proof.is_valid() {
        return Err("Invalid proof".to_string());
    }

    Ok(s.check_proof_single(commitment, proof, z, y)
        .unwrap_or(false))
}

pub fn verify_blob_kzg_proof_rust<TFr: Fr + Copy, TG1: G1, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(
    blob: &[TFr],
    commitment_g1: &TG1,
    proof_g1: &TG1,
    ts: &TKZGSettings,
) -> Result<bool, String> {
    if !commitment_g1.is_valid() {
        return Err("Invalid commitment".to_string());
    }
    if !proof_g1.is_valid() {
        return Err("Invalid proof".to_string());
    }

    let polynomial = blob_to_polynomial(blob)?;
    let evaluation_challenge_fr = compute_challenge(blob, commitment_g1)?;
    let y_fr =
        evaluate_polynomial_in_evaluation_form(&polynomial, &evaluation_challenge_fr, ts)?;
    verify_kzg_proof_rust(commitment_g1, &evaluation_challenge_fr, &y_fr, proof_g1, ts)
}

pub fn compute_challenges_and_evaluate_polynomial<TFr: Fr + Copy, TG1: G1, TG2: G2, TFFTSettings: FFTSettings<TFr>, TPoly: Poly<TFr>, TKZGSettings: KZGSettings<TFr, TG1, TG2, TFFTSettings, TPoly>>(
    blobs: &[Vec<TFr>],
    commitments_g1: &[TG1],
    ts: &TKZGSettings,
) -> Result<(Vec<TFr>, Vec<TFr>), String> {
    let mut evaluation_challenges_fr = Vec::with_capacity(blobs.len());
    let mut ys_fr = Vec::with_capacity(blobs.len());

    for i in 0..blobs.len() {
        let polynomial = blob_to_polynomial(&blobs[i])?;
        let evaluation_challenge_fr = compute_challenge(&blobs[i], &commitments_g1[i])?;
        let y_fr =
            evaluate_polynomial_in_evaluation_form(&polynomial, &evaluation_challenge_fr, ts)?;

        evaluation_challenges_fr.push(evaluation_challenge_fr);
        ys_fr.push(y_fr);
    }

    Ok((evaluation_challenges_fr, ys_fr))
}