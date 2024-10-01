use crate::{eip_4844::FIELD_ELEMENTS_PER_CELL, msm::precompute, FFTSettings, Fr, G1Affine, G1Fp, G1GetFp, G1Mul, KZGSettings, Poly, G1, G2};
use blst::blst_scalar;
use blst::{blst_fr, blst_p1, blst_p2};

pub fn toeplitz_coeffs_stride<PolyData: Poly<Coeff>, Coeff: Fr>(
    poly: &PolyData,
    offset: usize,
    stride: usize,
    outlen: usize,
) -> Result<PolyData, String> {
    let n = poly.len();

    if stride == 0 {
        return Err(String::from("stride must be greater than 0"));
    }

    let k = n / stride;
    let k2 = k * 2;

    if outlen < k2 {
        return Err(String::from("outlen must be equal or greater than k2"));
    }

    let mut out = PolyData::new(outlen);
    out.set_coeff_at(0, &poly.get_coeffs()[n - 1 - offset]);
    let mut i = 1;
    while i <= (k + 1) && i < k2 {
        out.set_coeff_at(i, &Fr::zero());
        i += 1;
    }
    let mut j = 2 * stride - offset - 1;
    for i in (k + 2)..k2 {
        out.set_coeff_at(i, &poly.get_coeffs()[j]);
        j += stride;
    }
    Ok(out)
}


pub fn compute_fk20_proofs<
    Coeff1: Fr,
    Coeff2: G1 + G1Mul<Coeff1> + G1GetFp<TG1Fp>,
    Coeff3: G2,
    Fs: FFTSettings<Coeff1>,
    Polynomial: Poly<Coeff1>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<Coeff2, TG1Fp>,
    KZG: KZGSettings<Coeff1, Coeff2, Coeff3, Fs, Polynomial, TG1Fp, TG1Affine>,
    TG1: G1 + Default,
>(
    p: &[Coeff1],
    n: usize,
    s: &KZG
) -> Result<Vec<Coeff2>, String> {
    let k = n / FIELD_ELEMENTS_PER_CELL;
    let k2 = k * 2;
    let mut toeplitz_coeffs: Vec<_> = vec![TG1::default(); k2];
    let mut toeplitz_coeffs_fft: Vec<_> = vec![TG1::default(); k2];
    let mut h_ext_fft: Vec<_> = vec![G1::default(); k2];
    let mut h: Vec<_> = vec![G1::default(); k2];

    let precompute = s.wbit != 0;
    let mut scratch: Option<Vec<u8>> = None;
    let mut scalars: Option<Vec<blst_scalar>> = None; 

    if precompute {
        scratch = Some(vec![0u8; s.scratch_size]);
        scalars = Some(vec![blst_scalar::default(); FIELD_ELEMENTS_PER_CELL]);
    }

    let mut coeffs: Vec<Vec<blst_fr>> = vec![vec![blst_fr::default(); k]; k2];

    // Compute toeplitz coeffiecients and organize by column
    for i in 0..FIELD_ELEMENTS_PER_CELL {
        toeplitz_coeffs_stride(&PolyData::new(p.to_vec()), i, FIELD_ELEMENTS_PER_CELL, k2)?;
        fr_fft(&mut toeplitz_coeffs_fft, &toeplitz_coeffs, k2, s)?;
        for j in 0..k2 {
            coeffs[j][i] = toeplitz_coeffs_fft[j];
        }
    }

    // Compute h_ext_fft_via MSM
    for i in 0..k2 {
        if precompute {
            let scalars = scalars.as_mut().unwrap();
            for j in 0..FIELD_ELEMENTS_PER_CELL {
                blst_scalar::from_fr(&mut scalars[j], &coeffs[i][j]);
            }
            let scalars_arg: [&[u8]; 2] = [
                unsafe { std::slice::from_raw_parts(scalars.as_ptr() as *const u8, scalars.len() * std::mem::size_of::<blst_scalar>())},
                &[]
            ];

            unsafe {
                blst::blst_p1s_mult_wbits(&mut h_ext_fft[i], s.tables[i].as_ptr(), s.wbits, FIELD_ELEMENTS_PER_CELL, scalars_arg.as_ptr(), Fr::BITS, scratch.as_mut().unwrap().as_mut_ptr());
            }
        } else {
            g1_lincomb_fast(&mut h_ext_fft[i], &s.x_ext_fft_columns[i], &coeffs[i], FIELD_ELEMENTS_PER_CELL)?;
        }
    }
    g1_ifft(&mut h, &h_ext_fft, k2, s)?;

    // Zero the second half of h
    for i in k..k2 {
        h[i] = G1::identity();
    }

    let mut out = vec![G1::default(); k2];
    g1_fft(&mut out, &h, k2, s)?;

    Ok(out)
}

