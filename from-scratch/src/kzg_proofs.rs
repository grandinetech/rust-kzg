use kzg::{Fr, G1, Fp, P1Affine};
use blst::{blst_p1_add_or_double,
           blst_p1s_to_affine,
           blst_scalar,
           blst_scalar_from_fr,
           blst_p1_mult,
           blst_p1s_mult_pippenger,
};

use crate::poly_utils::{new_poly_div};
use crate::kzg_types::{KZGSettings, Poly, create_fr_one, create_fr_zero, fr_pow, negate_fr};
use crate::utils::{is_power_of_two, log_2_byte};

pub fn commit_to_poly(out: &mut G1, poly: &Poly, kzg_settings: &KZGSettings) -> Result<(), String> {
    if poly.coeffs.len() > kzg_settings.secret_g1.len() {
        return Err(String::from("Polynomial is longer than secret g1"));
    }

    g1_linear_combination(out, &kzg_settings.secret_g1, &poly.coeffs, poly.coeffs.len());

    return Ok(());
}

fn compute_proof_multi(p: &Poly, x0: &Fr, n: usize, kzg_settings: &KZGSettings) -> Result<G1, String>{
    //CHECK(is_power_of_two(n));
    assert!(is_power_of_two(n));
    //poly divisor, q;
    //let mut divisor = Poly::default();
    let mut divisor: Poly = Poly { coeffs: Vec::default()};
    let mut q: Poly = Poly { coeffs: Vec::default()};

    
    // Construct x^n - x0^n = (x - x0.w^0)(x - x0.w^1)...(x - x0.w^(n-1))
    // TRY(new_poly(&divisor, n + 1));
    
    // -(x0^n)
    let result = fr_pow(&x0, n);
    assert!(result.is_ok()); 
    //fr_t x_pow_n;
    let x_pow_n = result.unwrap();
    
    // fr_negate(&divisor.coeffs[0], &x_pow_n);
    negate_fr(&mut divisor.coeffs[0], &x_pow_n);

    // Zeros
    for _ in 1..n {
        // divisor.coeffs[i] = create_fr_zero();
        divisor.coeffs.push(create_fr_zero());
    }

    // x^n
    divisor.coeffs.push(create_fr_one());
    // divisor.coeffs[n] = create_fr_one();

    // Calculate q = p / (x^n - x0^n)
    //TRY(new_poly_div(&q, p, &divisor));


    let result = new_poly_div(&p, &divisor);
    assert!(result.is_ok()); 
    q = result.unwrap();

    //TRY(commit_to_poly(out, &q, ks));
    let mut out = G1::default(); 
    commit_to_poly(&mut out, &q, &kzg_settings);

    return Ok(out);
}

fn g1_mul(out: &mut G1, a: &G1, b: &Fr) {
    let scalar: &mut blst_scalar = &mut blst_scalar::default();
    unsafe {
        blst_scalar_from_fr(&mut *scalar, b);
    }

    // Count the number of bytes to be multiplied.
    let mut i = scalar.b.len();// std::mem::size_of::<blst_scalar>();
    while i != 0 && scalar.b[i - 1] == 0 {
        i -= 1;
    }

    if i == 0 {
        let g1_identity: G1 = G1 {
            x: Fp { l: [0u64; 6] },
            y: Fp { l: [0u64; 6] },
            z: Fp { l: [0u64; 6] },
        };
        *out = g1_identity;
    } else if i == 1 && scalar.b[0] == 1 {
        *out = *a;
    } else {
        // Count the number of bits to be multiplied.
        unsafe {
            blst_p1_mult(out, a, &(scalar.b[0]), 8 * i - 7 + log_2_byte(scalar.b[i - 1]));
        }
    }
}

fn g1_linear_combination(out: &mut G1, p: &Vec<G1>, coeffs: &Vec<Fr>, len: usize) {
    if len < 8 { // Tunable parameter: must be at least 2 since Blst fails for 0 or 1
        // Direct approach
        let mut tmp: G1 = G1::default();

        let g1_identity: G1 = G1 {
            x: Fp { l: [0u64; 6] },
            y: Fp { l: [0u64; 6] },
            z: Fp { l: [0u64; 6] },
        };

        *out = g1_identity;
        for i in 0..len {
            unsafe {
                g1_mul(&mut tmp, &p[i], &coeffs[i]);
                blst_p1_add_or_double(out, out, &tmp);
            }
        }
    } else {

        // Blst's implementation of the Pippenger method
        //blst_p1_affine *p_affine = malloc(len * sizeof(blst_p1_affine));
        let mut p_affine = vec![P1Affine::default(); len];
        //blst_scalar *scalars = malloc(len * sizeof(blst_scalar));
        let mut scalars = vec![blst_scalar::default(); len];

        // Transform the points to affine representation
        //const blst_p1 *p_arg[2] = {p, NULL};
        // let p_arg: const* = {p, null}
        let p_arg: [*const G1; 2] = [&p[0], &G1::default()];
        //p_arg[0] = &p;

        unsafe {
            blst_p1s_to_affine(p_affine.as_mut_ptr(), p_arg.as_ptr(), len);
        }

        // Transform the field elements to 256-bit scalars
        for i in 0..len {
            unsafe {
                blst_scalar_from_fr(&mut scalars[i], &coeffs[i]);
            }
        }

        // Call the Pippenger implementation
        //const byte *scalars_arg[2] = {(byte *)scalars, NULL};
        let scalars_arg: [*const blst_scalar; 2] = [scalars.as_ptr(), &blst_scalar::default()];
        //scalars_arg[0] = &scalars;

        //const blst_p1_affine *points_arg[2] = {p_affine, NULL};
        let points_arg: [*const P1Affine; 2] = [p_affine.as_ptr(), &P1Affine::default()];
        //points_arg[0] = &p_affine;

        //void *scratch = malloc(blst_p1s_mult_pippenger_scratch_sizeof(len));
        let mut scratch: u64 = u64::default();
        //blst_p1s_mult_pippenger(out, points_arg, len, scalars_arg, 256, scratch);
        unsafe {
            blst_p1s_mult_pippenger(out, points_arg.as_ptr(), len, scalars_arg.as_ptr() as *const *const u8, 256, &mut scratch);
        }
    }
}
