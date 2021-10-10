use super::P1 as P1;
use super::Fr;

pub enum STATUS {
    SUCCESS,
    BAD_ARGS,
    ERROR
}

pub struct Poly {
    pub coeffs: Fr,
    pub length: u64,
}

/*
typedef struct {
    fr_t *coeffs;    //< `coeffs[i]` is the coefficient of the `x^i` term of the polynomial.
    uint64_t length; //< One more than the polynomial's degree
} poly;*/

// typedef blst_p1 g1_t;
// G1_T -> P1
//type G1_T = blst::blst_p1;
//type G2_T = blst::blst_p2;

pub struct FFTSettings {
    max_width: u64,
    root_of_unity: Fr,
    expanded_roots_of_unity: Fr,
    reverse_roots_of_unity: Fr
}
/*
typedef struct {
    uint64_t max_width;            //< The maximum size of FFT these settings support, a power of 2. 
    fr_t root_of_unity;            //< The root of unity used to generate the lists in the structure. 
    fr_t *expanded_roots_of_unity; //< Ascending powers of the root of unity, size `width + 1`. 
    fr_t *reverse_roots_of_unity;  //< Descending powers of the root of unity, size `width + 1`. 
} FFTSettings;*/

pub struct KZGSettings {
    pub fs: FFTSettings,
    pub secret_g1: P1,
    pub secret_g2: P1,
    pub length: u64
}
/*
typedef struct {
    const FFTSettings *fs; //< The corresponding settings for performing FFTs 
    g1_t *secret_g1;       //< G1 group elements from the trusted setup 
    g2_t *secret_g2;       //< G2 group elements from the trusted setup 
    uint64_t length;       //< The number of elements in secret_g1 and secret_g2
} KZGSettings;*/

pub fn commit_to_poly(_out: P1, p: Poly, ks: KZGSettings) -> STATUS {
    if p.length > ks.length {
        return STATUS::BAD_ARGS;
    }
    
    // g1_linear_combination()
    STATUS::SUCCESS
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