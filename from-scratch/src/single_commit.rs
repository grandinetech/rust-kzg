use crate::kzg_types;
use kzg::{Fr, P1, G1, Fp, P1Affine};
use blst::{ blst_p1_add_or_double,
            blst_p1s_to_affine,
            blst_p1s_mult_pippenger_scratch_sizeof,
            blst_scalar,
            blst_scalar_from_fr,
            blst_p1_mult,
            blst_p1s_mult_pippenger
        };

pub fn commit_to_poly(out: &mut G1, poly: kzg_types::Poly, kzg_settings: kzg_types::KZGSettings) {
    if poly.coeffs.len() > kzg_settings.secret_g1.len() {
        println!("BULLSHIT");
    }
    
    g1_linear_combination(out, kzg_settings.secret_g1, &poly.coeffs, poly.coeffs.len());
}

fn log_2_byte(b: u8) -> usize {

    let mut b_copy = b;
    let mut r: u8;
    let shift: u8;

    r = if b_copy > 0xF {1} else {0} << 2;
    b_copy >>= r;
    shift =  if b_copy > 0x3 {1} else {0} << 1;
    b_copy >>= (shift + 1);
    r |= shift | b_copy;
    r.into()
}

fn g1_mul(out: &mut G1, a: G1, b: & Fr) {
    let scalar: &mut blst_scalar = &mut blst_scalar::default();
    unsafe {
        blst_scalar_from_fr(&mut *scalar, b);
    }

    // Count the number of bytes to be multiplied.
    let mut i = std::mem::size_of::<blst_scalar>();
    while i != 0 && scalar.b[i - 1] == 0 {
        i-=1;
    }

    if i == 0 {
        let g1_identity: G1 = G1 {
            x: Fp { l: [0u64; 6] },
            y: Fp { l: [0u64; 6] },
            z: Fp { l: [0u64; 6] }
        };
        *out = g1_identity;
    } else if i == 1 && scalar.b[0] == 1 {
        *out = a;
    } else {
        // Count the number of bits to be multiplied.
        unsafe {
            blst_p1_mult(out, &a, &(scalar.b[0]), 8 * i - 7 + log_2_byte(scalar.b[i - 1]));
        }
    }
}

fn g1_linear_combination(out: &mut G1, p: Vec<G1>, coeffs: & Vec<Fr>, len: usize) {

    if len < 8 { // Tunable parameter: must be at least 2 since Blst fails for 0 or 1
        // Direct approach
        let mut tmp: G1 = G1::default();

        let g1_identity: G1 = G1 {
            x: Fp { l: [0u64; 6] },
            y: Fp { l: [0u64; 6] },
            z: Fp { l: [0u64; 6] }
        };

        *out = g1_identity;
        for i in 0..len {

            unsafe {
                g1_mul(&mut tmp, p[i], &coeffs[i]);
                blst_p1_add_or_double(out, out, &tmp);
            }
        }
    } else {
        
        // Blst's implementation of the Pippenger method
        //blst_p1_affine *p_affine = malloc(len * sizeof(blst_p1_affine));
        let mut p_affine: P1Affine;
        //blst_scalar *scalars = malloc(len * sizeof(blst_scalar));
        let mut scalars: [blst_scalar; len] = vec![blst_scalar::default(); len];
        
        // Transform the points to affine representation
        //const blst_p1 *p_arg[2] = {p, NULL};
        // let p_arg: const* = {p, null}
        let mut p_arg: [*const G1; 2] = [&p[0], &G1::default()];
        //p_arg[0] = &p;
        
        blst_p1s_to_affine(&mut p_affine, p_arg.as_ptr(), len);
        
        // Transform the field elements to 256-bit scalars
        for i in 0..len {
            blst_scalar_from_fr(&mut scalars[i], &coeffs[i]);
        }
        
        // Call the Pippenger implementation
        //const byte *scalars_arg[2] = {(byte *)scalars, NULL};
        let mut scalars_arg: [*const u8; 2] =  [&scalars, &u8::default()];
        //scalars_arg[0] = &scalars;
        
        //const blst_p1_affine *points_arg[2] = {p_affine, NULL};
        let points_arg: [*const P1Affine; 2] = [&p_affine, &P1Affine::default()];
        //points_arg[0] = &p_affine;
        
        //void *scratch = malloc(blst_p1s_mult_pippenger_scratch_sizeof(len));
        let mut scratch: u64;
        //blst_p1s_mult_pippenger(out, points_arg, len, scalars_arg, 256, scratch);
        blst_p1s_mult_pippenger(out,  points_arg.as_ptr(), len, scalars_arg.as_ptr(), 256, &mut scratch);
        
    }
}


/*
C_KZG_RET commit_to_poly(g1_t *out, const poly *p, const KZGSettings *ks) {
    CHECK(p->length <= ks->length);
    g1_linear_combination(out, ks->secret_g1, p->coeffs, p->length);
    return C_KZG_OK;
}
*/



/*
 * Multiply a G1 group element by a field element.
 *
 * This "undoes" the Blst constant-timedness. FFTs do a lot of multiplication by one, so constant time is rather slow.
 *
 * @param[out] out [@p b]@p a
 * @param[in]  a   The G1 group element
 * @param[in]  b   The multiplier
 */
/*
void g1_mul(g1_t *out, const g1_t *a, const fr_t *b) {
    blst_scalar s;
    blst_scalar_from_fr(&s, b); // blst::blst_scalar_from_fr(&s, b)

    // Count the number of bytes to be multiplied.
    int i = sizeof(blst_scalar);
    while (i && !s.b[i - 1]) --i;
    if (i == 0) {
        *out = g1_identity;
    } else if (i == 1 && s.b[0] == 1) {
        *out = *a;
    } else {
        // Count the number of bits to be multiplied.
        blst_p1_mult(out, a, s.b, 8 * i - 7 + log_2_byte(s.b[i - 1])); //blst::blst_p1_mult();
    }
}*/
/*
void g1_linear_combination(g1_t *out, const g1_t *p, const fr_t *coeffs, const uint64_t len) {

    if (len < 8) { // Tunable parameter: must be at least 2 since Blst fails for 0 or 1
        // Direct approach
        g1_t tmp;
        *out = g1_identity;
        for (uint64_t i = 0; i < len; i++) {
            g1_mul(&tmp, &p[i], &coeffs[i]); //blst::g1_mul(&tmp, &p[i], &coeffs[i]);
            blst_p1_add_or_double(out, out, &tmp); //blst::blst_p1_add_or_double();
        }
    } else {
        // Blst's implementation of the Pippenger method
        void *scratch = malloc(blst_p1s_mult_pippenger_scratch_sizeof(len));
        blst_p1_affine *p_affine = malloc(len * sizeof(blst_p1_affine));
        blst_scalar *scalars = malloc(len * sizeof(blst_scalar));

        // Transform the points to affine representation
        const blst_p1 *p_arg[2] = {p, NULL};
        // let p_arg: const* = {p, null}
        
        blst_p1s_to_affine(p_affine, p_arg, len);

        // Transform the field elements to 256-bit scalars
        for (int i = 0; i < len; i++) {
            blst_scalar_from_fr(&scalars[i], &coeffs[i]);
        }

        // Call the Pippenger implementation
        const byte *scalars_arg[2] = {(byte *)scalars, NULL};
        const blst_p1_affine *points_arg[2] = {p_affine, NULL};
        blst_p1s_mult_pippenger(out, points_arg, len, scalars_arg, 256, scratch);

        // Tidy up
        free(scratch);
        free(p_affine);
        free(scalars);
    }
}

C_KZG_RET commit_to_poly(g1_t *out, const poly *p, const KZGSettings *ks) {
    CHECK(p->length <= ks->length);
    g1_linear_combination(out, ks->secret_g1, p->coeffs, p->length);
    return C_KZG_OK;
}
*/