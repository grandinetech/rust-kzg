use crate::data_types::{fr::*, g1::*, g2::*, gt::*};
use crate::fk20_fft::{FFTSettings, G1_GENERATOR};
use crate::mcl_methods::{final_exp, mclBn_FrEvaluatePolynomial, pairing};
use crate::utilities::{log_2, next_pow_of_2};
use std::{cmp::min, iter, ops};

const G1_GEN_X: &str = "3685416753713387016781088315183077757961620795782546409894578378688607592378376318836054947676345821548104185464507";
const G1_GEN_Y: &str = "1339506544944476473020471379941921221584933875938349620426543736416511423956333506472724655353366534992391756441569";
const G2_GEN_X_D0: &str = "352701069587466618187139116011060144890029952792775240219908644239793785735715026873347600343865175952761926303160";
const G2_GEN_X_D1: &str = "3059144344244213709971259814753781636986470325476647558659373206291635324768958432433509563104347017837885763365758";
const G2_GEN_Y_D0: &str = "1985150602287291935568054521177171638300868978215655730859378665066344726373823718423869104263333984641494340347905";
const G2_GEN_Y_D1: &str = "927553665492332455747201965776037880757740193453592970025027978793976877002675564980949289727957565575433344219582";

impl G1 {
    pub fn gen() -> G1 {
        let mut g1 = G1::default();
        g1.x.set_str(G1_GEN_X, 10);
        g1.y.set_str(G1_GEN_Y, 10);
        g1.z.set_int(1);
        g1
    }

    pub fn pair(&self, rhs: &G2) -> GT {
        let mut gt = GT::default();

        pairing(&mut gt, self, rhs);

        gt
    }

    pub fn random() -> G1 {
        let fr = Fr::random();
        &G1_GENERATOR * &fr
    }
}

impl ops::Mul<&Fr> for G1 {
    type Output = G1;
    fn mul(self, rhs: &Fr) -> Self::Output {
        let mut g1 = G1::default();
        G1::mul(&mut g1, &self, rhs);

        g1
    }
}

impl ops::Mul<&Fr> for &G1 {
    type Output = G1;
    fn mul(self, rhs: &Fr) -> Self::Output {
        let mut g1 = G1::default();
        G1::mul(&mut g1, self, rhs);

        g1
    }
}

impl ops::Mul<&Fr> for &mut G1 {
    type Output = G1;
    fn mul(self, rhs: &Fr) -> Self::Output {
        let mut g1 = G1::default();
        G1::mul(&mut g1, self, rhs);

        g1
    }
}

impl ops::Sub<G1> for G1 {
    type Output = G1;
    fn sub(self, rhs: G1) -> Self::Output {
        let mut g1 = G1::default();
        G1::sub(&mut g1, &self, &rhs);

        g1
    }
}

impl GT {
    pub fn get_final_exp(&self) -> GT {
        let mut gt = GT::default();
        final_exp(&mut gt, self);

        gt
    }

    pub fn get_inv(&self) -> GT {
        let mut gt = GT::default();
        GT::inv(&mut gt, self);

        gt
    }
}

impl ops::Mul<GT> for GT {
    type Output = GT;
    fn mul(self, rhs: GT) -> Self::Output {
        let mut gt = GT::default();
        GT::mul(&mut gt, &self, &rhs);

        gt
    }
}

impl G2 {
    pub fn gen() -> G2 {
        let mut g2 = G2::default();
        g2.x.d[0].set_str(G2_GEN_X_D0, 10);
        g2.x.d[1].set_str(G2_GEN_X_D1, 10);
        g2.y.d[0].set_str(G2_GEN_Y_D0, 10);
        g2.y.d[1].set_str(G2_GEN_Y_D1, 10);
        g2.z.d[0].set_int(1);
        g2.z.d[1].clear();

        g2
    }
}

impl ops::Mul<&Fr> for &G2 {
    type Output = G2;
    fn mul(self, rhs: &Fr) -> Self::Output {
        let mut g2 = G2::default();
        G2::mul(&mut g2, self, rhs);

        g2
    }
}

impl ops::Sub<G2> for G2 {
    type Output = G2;
    fn sub(self, rhs: G2) -> Self::Output {
        let mut g2 = G2::default();
        G2::sub(&mut g2, &self, &rhs);

        g2
    }
}

impl Fr {
    pub fn one() -> Fr {
        Fr::from_int(1)
    }

    pub fn get_neg(&self) -> Fr {
        let mut fr = Fr::default();
        Fr::neg(&mut fr, self);

        fr
    }

    pub fn get_inv(&self) -> Fr {
        let mut fr = Fr::default();
        Fr::inv(&mut fr, self);

        fr
    }

    pub fn random() -> Fr {
        let mut fr = Fr::default();
        Fr::set_by_csprng(&mut fr);

        fr
    }
}

impl ops::Mul<Fr> for Fr {
    type Output = Fr;
    fn mul(self, rhs: Fr) -> Self::Output {
        let mut result = Fr::default();
        Fr::mul(&mut result, &self, &rhs);

        result
    }
}

impl ops::Div<Fr> for Fr {
    type Output = Fr;
    fn div(self, rhs: Fr) -> Self::Output {
        let mut result = Fr::default();
        Fr::div(&mut result, &self, &rhs);

        result
    }
}

impl ops::Sub<Fr> for Fr {
    type Output = Fr;
    fn sub(self, rhs: Fr) -> Self::Output {
        let mut result = Fr::default();
        Fr::sub(&mut result, &self, &rhs);

        result
    }
}

impl ops::Add<Fr> for Fr {
    type Output = Fr;
    fn add(self, rhs: Fr) -> Self::Output {
        let mut result = Fr::default();
        Fr::add(&mut result, &self, &rhs);

        result
    }
}

// KZG 10 Impl

#[derive(Debug, Clone)]
pub struct Polynomial {
    pub coeffs: Vec<Fr>,
}

impl Polynomial {
    pub fn default() -> Self {
        Self { coeffs: vec![] }
    }

    pub fn new(size: usize) -> Self {
        Polynomial {
            coeffs: vec![Fr::default(); size],
        }
    }

    pub fn from_fr(data: Vec<Fr>) -> Self {
        Self { coeffs: data }
    }

    pub fn from_i32(data: &[i32]) -> Self {
        Self {
            coeffs: data.iter().map(|x| Fr::from_int(*x)).collect(),
        }
    }

    pub fn order(&self) -> usize {
        self.coeffs.len()
    }

    pub fn eval_at(&self, point: &Fr) -> Fr {
        let mut result = Fr::default();
        unsafe {
            mclBn_FrEvaluatePolynomial(&mut result, self.coeffs.as_ptr(), self.order(), point)
        };
        result
    }

    pub fn gen_proof_at(&self, g1_points: &[G1], point: &Fr) -> Result<G1, String> {
        let divisor = vec![point.get_neg(), Fr::one()];
        let quotient_poly = self.long_division(&divisor).unwrap();

        let mut result = G1::default();
        unsafe {
            mclBnG1_mulVec(
                &mut result,
                g1_points.as_ptr(),
                quotient_poly.coeffs.as_ptr(),
                min(g1_points.len(), quotient_poly.order()),
            )
        };
        Ok(result)
    }

    pub fn poly_quotient_length(dividend: &[Fr], divisor: &[Fr]) -> usize {
        if dividend.len() >= divisor.len() {
            dividend.len() - divisor.len() + 1
        } else {
            0
        }
    }

    pub fn long_division(&self, divisor: &[Fr]) -> Result<Polynomial, String> {
        if divisor.is_empty() {
            return Err(String::from("Dividing by zero is undefined"));
        }
        if divisor.last().unwrap().is_zero() {
            return Err(String::from(
                "The divisor's highest coefficient must be non-zero",
            ));
        }
        let out_length = Polynomial::poly_quotient_length(&self.coeffs, divisor);
        if out_length == 0 {
            return Ok(Polynomial::default());
        }

        let mut a_pos = self.order() - 1;
        let b_pos = divisor.len() - 1;
        let mut diff = a_pos - b_pos;

        let mut a = self.coeffs.clone();
        let mut out_coeffs = vec![Fr::default(); out_length];

        while diff > 0 {
            out_coeffs[diff] = a[a_pos] / divisor[b_pos];

            for i in 0..b_pos {
                // a[diff + i] -= b[i] * quot
                let tmp = out_coeffs[diff] * divisor[i];
                a[diff + i] = a[diff + i] - tmp;
            }

            a_pos -= 1;
            diff -= 1;
        }
        out_coeffs[0] = a[a_pos] / divisor[b_pos];
        Ok(Polynomial::from_fr(out_coeffs))
    }

    pub fn fast_div(&self, divisor: &[Fr]) -> Result<Polynomial, String> {
        if divisor.is_empty() {
            return Err(String::from("Dividing by zero is undefined"));
        }
        if divisor.last().unwrap().is_zero() {
            return Err(String::from(
                "The divisor's highest coefficient must be non-zero",
            ));
        }
        let mut out_length = Polynomial::poly_quotient_length(&self.coeffs, divisor);
        if out_length == 0 {
            return Ok(Polynomial::default());
        }

        // Special case for divisor.length == 1 (it's a constant)
        if divisor.len() == 1 {
            let mut out_coeffs: Vec<Fr> = vec![];
            out_length = self.order();
            for i in 0..out_length {
                out_coeffs.push(self.coeffs[i] / divisor[0]);
            }
            return Ok(Polynomial::from_fr(out_coeffs));
        }

        let a_flip = Polynomial::from_fr(Polynomial::flip_coeffs(&self.coeffs));
        let b_flip = Polynomial::from_fr(Polynomial::flip_coeffs(divisor));
        let inv_b_flip = b_flip.inverse(out_length).unwrap();
        let q_flip = a_flip.mul(&inv_b_flip, out_length).unwrap();

        Ok(q_flip.flip())
    }

    fn normalise_coeffs(coeffs: &[Fr]) -> Vec<Fr> {
        let mut ret_length = coeffs.len();
        while ret_length > 0 && coeffs[ret_length - 1].is_zero() {
            ret_length -= 1;
        }
        coeffs[0..ret_length].to_vec()
    }

    fn normalise(&self) -> Polynomial {
        Polynomial::from_fr(Polynomial::normalise_coeffs(&self.coeffs))
    }

    fn flip_coeffs(coeffs: &[Fr]) -> Vec<Fr> {
        let mut result: Vec<Fr> = vec![];
        for i in (0..coeffs.len()).rev() {
            result.push(coeffs[i]);
        }
        result
    }

    fn flip(&self) -> Polynomial {
        Polynomial::from_fr(Polynomial::flip_coeffs(&self.coeffs))
    }

    pub fn div(&self, _divisor: &[Fr]) -> Result<Polynomial, String> {
        let dividend = self.normalise();
        let divisor = Polynomial::normalise_coeffs(_divisor);

        if divisor.len() >= dividend.order() || divisor.len() < 128 {
            // Tunable paramter
            self.long_division(&divisor)
        } else {
            self.fast_div(&divisor)
        }
    }

    pub fn commit(&self, g1_points: &[G1]) -> Result<G1, String> {
        if self.order() > g1_points.len() {
            return Err(String::from("Provided polynomial is longer than G1!"));
        }

        let mut result = G1::default();
        unsafe {
            mclBnG1_mulVec(
                &mut result,
                g1_points.as_ptr(),
                self.coeffs.as_ptr(),
                min(g1_points.len(), self.order()),
            )
        };
        Ok(result)
    }

    pub fn random(order: usize) -> Polynomial {
        let coeffs = iter::repeat(0).take(order).map(|_| Fr::random()).collect();

        Polynomial { coeffs }
    }

    pub fn mul_(
        &self,
        b: &Self,
        ft: Option<&FFTSettings>,
        len: usize,
    ) -> Result<Polynomial, String> {
        if self.order() < 64 || b.order() < 64 || len < 128 {
            // Tunable parameter
            Polynomial::mul_direct(self, b, len)
        } else {
            Polynomial::mul_fft(self, b, ft, len)
        }
    }

    pub fn mul(&self, b: &Self, len: usize) -> Result<Polynomial, String> {
        Polynomial::mul_(self, b, None, len)
    }

    /// @param[in]  n_in  The number of elements of @p in to take
    /// @param[in]  n_out The length of @p out
    pub fn pad_coeffs(coeffs: &[Fr], n_in: usize, n_out: usize) -> Vec<Fr> {
        let num = min(n_in, n_out);
        let mut ret_coeffs: Vec<Fr> = vec![];
        for item in coeffs.iter().take(num) {
            ret_coeffs.push(*item);
        }
        for _ in num..n_out {
            ret_coeffs.push(Fr::zero());
        }
        ret_coeffs
    }

    pub fn pad_coeffs_mut(&mut self, n_in: usize, n_out: usize) {
        let num = min(n_in, n_out);
        self.coeffs = self.coeffs[..num].to_vec();
        for _ in num..n_out {
            self.coeffs.push(Fr::zero());
        }
    }

    /// @param[in]  n_in  The number of elements of @p in to take
    /// @param[in]  n_out The length of @p out
    fn pad(&self, n_in: usize, n_out: usize) -> Polynomial {
        Polynomial::from_fr(Polynomial::pad_coeffs(&self.coeffs, n_in, n_out))
    }

    // #[cfg(feature = "parallel")] 
    pub fn mul_fft(
        &self,
        b: &Self,
        ft: Option<&FFTSettings>,
        len: usize,
    ) -> Result<Polynomial, String> {
        // Truncate a and b so as not to do excess work for the number of coefficients required.
        let a_len = min(self.order(), len);
        let b_len = min(b.order(), len);
        let length = next_pow_of_2(a_len + b_len - 1);

        //TO DO remove temp_fft, can't find a nice way to declare fft and only use it as ref
        let temp_fft = FFTSettings::new(log_2(length) as u8);
        let fft_settings = match ft {
            Some(x) => x,
            None => &temp_fft,
        };
        let ft = fft_settings;
        if length > ft.max_width {
            return Err(String::from("Mul fft only good up to length < 32 bits"));
        }

        let a_pad = self.pad(a_len, length);
        let b_pad = b.pad(b_len, length);
        let a_fft: Vec<Fr>;
        let b_fft: Vec<Fr>;

        #[cfg(feature = "parallel")] 
        {
            if length > 1024 {
                let mut a_fft_temp = vec![];
                let mut b_fft_temp = vec![];

                rayon::join(
                    || a_fft_temp = ft.fft(&a_pad.coeffs, false).unwrap(),
                    || b_fft_temp = ft.fft(&b_pad.coeffs, false).unwrap(),
                );
                
                a_fft = a_fft_temp;
                b_fft = b_fft_temp;
            } else {
                a_fft = ft.fft(&a_pad.coeffs, false).unwrap();
                b_fft = ft.fft(&b_pad.coeffs, false).unwrap();
            }

        }
        #[cfg(not(feature="parallel"))]
        {
            a_fft = ft.fft(&a_pad.coeffs, false).unwrap();
            b_fft = ft.fft(&b_pad.coeffs, false).unwrap(); 
        }
        
        let mut ab_fft = a_fft;
        for i in 0..length {
            ab_fft[i] = ab_fft[i] * b_fft[i];
        }
        let ab = ft.fft(&ab_fft, true).unwrap();

        let mut ret_coeffs: Vec<Fr> = ab;

        //pad if too short, else take first len if too long
        if len > length {
            for _ in length..len {
                ret_coeffs.push(Fr::zero());
            }
        } else {
            unsafe {
                ret_coeffs.set_len(len);
            }
        }
        Ok(Polynomial::from_fr(ret_coeffs))
    }

    pub fn mul_direct(&self, b: &Self, len: usize) -> Result<Polynomial, String> {
        let mut coeffs: Vec<Fr> = vec![];
        for _ in 0..len {
            coeffs.push(Fr::zero());
        }

        for i in 0..self.order() {
            let mut j = 0;
            while j < b.order() && i + j < len {
                let temp = self.coeffs[i] * b.coeffs[j];
                coeffs[i + j] = coeffs[i + j] + temp;
                j += 1;
            }
        }
        Ok(Polynomial::from_fr(coeffs))
    }

    pub fn inverse(&self, new_length: usize) -> Result<Polynomial, String> {
        let self_length = self.order();
        if self_length == 0 || new_length == 0 {
            return Ok(Polynomial::default());
        }
        if self.coeffs[0].is_zero() {
            return Err(String::from("The constant term of self must be nonzero."));
        }

        // If the input polynomial is constant, the remainder of the series is zero
        if self_length == 1 {
            let mut coeffs = vec![self.coeffs[0].inverse()];
            for _ in 1..new_length {
                coeffs.push(Fr::zero());
            }
            return Ok(Polynomial::from_fr(coeffs));
        }

        let maxd = new_length - 1;
        let mut d = 0;

        // Max space for multiplications is (2 * length - 1)
        //use a more efficent log_2?
        let scale = log_2(next_pow_of_2(2 * new_length - 1));
        //check if scale actually always fits in u8
        //fftsettings to be used, if multiplacation is done with fft
        let fs = FFTSettings::new(scale as u8);
        let coeffs = vec![self.coeffs[0].inverse()];
        let mut out = Polynomial::from_fr(coeffs);

        //if new length is 1, max d is 0
        let mut mask = 1 << log_2(maxd);
        while mask != 0 {
            d = 2 * d + ((maxd & mask) != 0) as usize;
            mask >>= 1;

            // b.c -> tmp0 (we're using out for c)
            let temp_0_len = min(d + 1, self.order() + out.order() - 1);
            let mut poly_temp_0: Polynomial = self.mul_(&out, Some(&fs), temp_0_len).unwrap();

            // 2 - b.c -> tmp0
            for i in 0..temp_0_len {
                poly_temp_0.coeffs[i] = poly_temp_0.coeffs[i].get_neg();
            }

            let fr_two = Fr::from_int(2);
            poly_temp_0.coeffs[0] = poly_temp_0.coeffs[0] + fr_two;

            // c.(2 - b.c) -> tmp1;
            let temp_1_len = d + 1;
            let poly_temp_1: Polynomial = out.mul_(&poly_temp_0, Some(&fs), temp_1_len).unwrap();

            out = Polynomial::from_fr(poly_temp_1.coeffs);
        }

        if d + 1 != new_length {
            return Err(String::from("d + 1 != new_length"));
        }

        Ok(out)
    }
}

#[derive(Debug, Clone)]
pub struct Curve {
    pub g1_gen: G1,
    pub g2_gen: G2,
    pub g1_points: Vec<G1>,
    pub g2_points: Vec<G2>,
    pub order: usize,
}

impl Curve {
    pub fn default() -> Self {
        let g1_gen = G1::gen();
        let g2_gen = G2::gen();
        let g1_points: Vec<G1> = vec![];
        let g2_points: Vec<G2> = vec![];
        let order = 0;

        Self {
            g1_gen,
            g2_gen,
            g1_points,
            g2_points,
            order
        }
    }

    pub fn new(secret: &Fr, order: usize) -> Self {
        let g1_gen = G1::gen();
        let g2_gen = G2::gen();

        let mut g1_points = vec![G1::default(); order];
        let mut g2_points = vec![G2::default(); order];

        let mut secret_to_power = Fr::one();
        for i in 0..order {
            G1::mul(&mut (g1_points[i]), &g1_gen, &secret_to_power);
            G2::mul(&mut (g2_points[i]), &g2_gen, &secret_to_power);

            secret_to_power *= secret;
        }

        Self {
            g1_gen,
            g2_gen,
            g1_points,
            g2_points,
            order
        }
    }

    pub fn new2(secret_g1: &[G1], secret_g2: &[G2], order: usize) -> Self {
        let g1_gen = G1::gen();
        let g2_gen = G2::gen();

        let mut g1_points: Vec<G1> = vec![];
        let mut g2_points: Vec<G2> = vec![];
        for i in 0..order {
            g1_points.push(secret_g1[i]);
            g2_points.push(secret_g2[i].clone());
        }

        Self {
            g1_gen,
            g2_gen,
            g1_points,
            g2_points,
            order,
        }
    }

    pub fn is_proof_valid(&self, commitment: &G1, proof: &G1, x: &Fr, y: &Fr) -> bool {
        let secret_minus_x = &self.g2_points[1] - &(&self.g2_gen * x); // g2 * x to get x on g2
        let commitment_minus_y = commitment - &(self.g1_gen * y);

        Curve::verify_pairing(&commitment_minus_y, &self.g2_gen, proof, &secret_minus_x)
    }

    pub fn verify_pairing(a1: &G1, a2: &G2, b1: &G1, b2: &G2) -> bool {
        let pairing1 = a1.pair(a2).get_inv();
        let pairing2 = b1.pair(b2);

        let result = (pairing1 * pairing2).get_final_exp();

        result.is_one()
    }
}
