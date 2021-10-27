use std::{cmp::min, ops, iter};
use crate::data_types::{fr::*, g1::*, g2::*, gt::*};
use crate::{BlstFr};
use crate::data_converter::fr_converter::*;
use crate::mcl_methods::{pairing, final_exp, mclBn_FrEvaluatePolynomial};
use crate::utilities::{next_pow_of_2, log_2};
use crate::fk20_fft::{FFTSettings, G1_GENERATOR};

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
        return g1;
    }

    pub fn pair(&self, rhs: &G2) -> GT {
        let mut gt = GT::default();

        pairing(&mut gt, &self, &rhs);

        return gt;
    }

    pub fn random() -> G1 {
        let fr = Fr::random();
        let g1 = &G1_GENERATOR * &fr;
        g1
    }
}

impl ops::Mul<&Fr> for &G1 {
    type Output = G1;
    fn mul(self, rhs: &Fr) -> Self::Output {
        let mut g1 = G1::default();
        G1::mul(&mut g1, &self, &rhs);

        return g1;
    }
}

impl ops::Sub<G1> for G1 {
    type Output = G1;
    fn sub(self, rhs: G1) -> Self::Output {
        let mut g1 = G1::default();
        G1::sub(&mut g1, &self, &rhs);

        return g1;
    }
}

impl GT {
    pub fn get_final_exp(&self) -> GT { 
        let mut gt = GT::default();
        final_exp(&mut gt, &self);

        return gt;
    }

    pub fn get_inv(&self) -> GT {
        let mut gt = GT::default();
        GT::inv(&mut gt, self);

        return gt;
    }
}

impl ops::Mul<GT> for GT {
    type Output = GT;
    fn mul(self, rhs: GT) -> Self::Output {
        let mut gt = GT::default();
        GT::mul(&mut gt, &self, &rhs);

        return gt;
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

        return g2;
    }
}

impl ops::Mul<&Fr> for &G2 {
    type Output = G2;
    fn mul(self, rhs: &Fr) -> Self::Output {
        let mut g2 = G2::default();
        G2::mul(&mut g2, &self, &rhs);

        return g2;
    }
}

impl ops::Sub<G2> for G2 {
    type Output = G2;
    fn sub(self, rhs: G2) -> Self::Output {
        let mut g2 = G2::default();
        G2::sub(&mut g2, &self, &rhs);

        return g2;
    }
}

impl Fr {
    pub fn one() -> Fr {
        Fr::from_int(1)
    }

    pub fn get_neg(&self) -> Fr {
        let mut fr = Fr::default();
        Fr::neg(&mut fr, self);

        return fr;
    }

    pub fn get_inv(&self) -> Fr {
        let mut fr = Fr::default();
        Fr::inv(&mut fr, self);

        return fr;
    }

    pub fn random() -> Fr {
        let mut fr = Fr::default();
        Fr::set_by_csprng(&mut fr);

        return fr;
    }
}

impl ops::Mul<Fr> for Fr {
    type Output = Fr;
    fn mul(self, rhs: Fr) -> Self::Output {
        let mut result = Fr::default();
        Fr::mul(&mut result, &self, &rhs);

        return result;
    }
}

impl ops::Div<Fr> for Fr {
    type Output = Fr;
    fn div(self, rhs: Fr) -> Self::Output {
        let mut result = Fr::default();
        Fr::div(&mut result, &self, &rhs);

        return result;
    }
}

impl ops::Sub<Fr> for Fr {
    type Output = Fr;
    fn sub(self, rhs: Fr) -> Self::Output {
        let mut result = Fr::default();
        Fr::sub(&mut result, &self, &rhs);

        return result;
    }
}

impl ops::Add<Fr> for Fr {
    type Output = Fr;
    fn add(self, rhs: Fr) -> Self::Output {
        let mut result = Fr::default();
        Fr::add(&mut result, &self, &rhs);

        return result;
    }
}

// KZG 10 Impl

#[derive(Debug, Clone)]
pub struct Polynomial {
    pub coeffs: Vec<Fr>
}

#[derive(Debug, Clone)]
pub struct Curve {
    pub g1_gen: G1,
    pub g2_gen: G2,
    pub g1_points: Vec<G1>,
    pub g2_points: Vec<G2>,
    pub order: usize
}

impl Polynomial {
    pub fn default() -> Self {
        Self {
            coeffs: vec![]
        }
    }
    pub fn new(size: usize) -> Self {
        Polynomial {
            coeffs: vec![Fr::default(); size],
        }
    }

    pub fn from_fr(data: Vec<Fr>) -> Self {
        Self {
            coeffs: data
        }
    }
    
    pub fn from_i32(data: &Vec<i32>) -> Self {
        Self {
            coeffs: data.iter().map(|x| Fr::from_int(*x)).collect(),
        }
    }

    pub fn order(&self) -> usize {
        self.coeffs.len()
    }

    pub fn eval_at_blst(&self, point: &BlstFr) -> BlstFr {
        let point_from_blst = fr_from_blst(*point);
        return fr_to_blst(self.eval_at(&point_from_blst));
    }

    pub fn eval_at(&self, point: &Fr) -> Fr {
        let mut result = Fr::default();
        unsafe { 
            mclBn_FrEvaluatePolynomial(&mut result, self.coeffs.as_ptr(), self.order(), point)
        };
        return result;
    }

    pub fn gen_proof_at(&self, g1_points: &Vec<G1>, point: &Fr) -> G1 {
        let divisor = vec![point.get_neg(), Fr::one()];
        let quotient_poly = self.long_division(&divisor);

        let mut result = G1::default();
        unsafe {
            mclBnG1_mulVec(&mut result, g1_points.as_ptr(), quotient_poly.coeffs.as_ptr(), min(g1_points.len(), quotient_poly.order()))
        };
        return result;
    }

    pub fn long_division(&self, divisor: &Vec<Fr>) -> Polynomial {
        let mut poly_copy = self.clone();
        let mut copy_pos = poly_copy.order() - 1;

        let mut result = vec![Fr::default(); poly_copy.order() - divisor.len() + 1];//here
        
        for r_i in (0 .. result.len()).rev() {
            result[r_i] = &poly_copy.coeffs[copy_pos] / &divisor.last().unwrap();

            for d_i in (0 .. divisor.len()).rev() {
                poly_copy.coeffs[r_i + d_i] -= &(&result[r_i] * &divisor[d_i]);
            }

            copy_pos -= 1;//here
        }

        return Polynomial {
            coeffs: result
        };
    }

    pub fn commit(&self, g1_points: &Vec<G1>) -> G1 {
        let mut result = G1::default();
        unsafe {
            mclBnG1_mulVec(&mut result, g1_points.as_ptr(), self.coeffs.as_ptr(), min(g1_points.len(), self.order()))
        };
        return result;
    }

    pub fn random(order: usize) -> Polynomial {
        let coeffs = iter::repeat(0)
            .take(order)
            .map(|_| Fr::random())
            .collect();

        return Polynomial {
            coeffs
        };
    }

    pub fn mul(&self, b: &Self, _ft: &FFTSettings, len: usize) -> Result<Polynomial, String> {
        //if the polynomials are large, we should use fft multiplication, for now it's not implemented
        Polynomial::mul_direct(self, b, len)
    }

    pub fn mul_direct(&self, b: &Self, len: usize) -> Result<Polynomial, String> {
        let mut coeffs: Vec<Fr> = vec![];
        for _ in 0..len {
            coeffs.push(Fr::zero());
        }

        for i in 0..self.order() {
            for j in 0..b.order() {

                let temp = self.coeffs[i] * b.coeffs[j];
                coeffs[i + j] = coeffs[i + j] + temp;
            }
        }
        return Ok(Polynomial::from_fr(coeffs));
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
        let coeffs = vec![self.coeffs[0].clone().inverse()];
        let mut out = Polynomial::from_fr(coeffs);

        let mut mask = 1 << log_2(maxd);
        
        let mut poly_temp_0: Polynomial;
        let mut poly_temp_1: Polynomial;
        while mask != 0 {
            d = 2 * d + ((maxd & mask) != 0) as usize;
            mask >>= 1;

            // b.c -> tmp0 (we're using out for c)
            let temp_0_len = min(d + 1, self.order() + &out.order() - 1);
            poly_temp_0 = self.mul(&out, &fs, temp_0_len).unwrap();

             // 2 - b.c -> tmp0
            for i in 0..temp_0_len {
                poly_temp_0.coeffs[i] = poly_temp_0.coeffs[i].get_neg();
            }

            let fr_two = Fr::from_int(2);
            poly_temp_0.coeffs[0] = poly_temp_0.coeffs[0] + fr_two;

            // c.(2 - b.c) -> tmp1;
            let temp_1_len = d + 1;
            poly_temp_1 = out.mul(&poly_temp_0, &fs, temp_1_len).unwrap();

            out = Polynomial::from_fr(poly_temp_1.coeffs.clone());
        }

        if d + 1 != new_length {
            return Err(String::from("d + 1 != new_length"));
        }

        return Ok(out);
    }
}

impl Curve {
    pub fn new(secret: &Fr, order: usize) -> Self {
        let g1_gen = G1::gen();
        let g2_gen = G2::gen(); 

        let mut g1_points = vec!(G1::default(); order);
        let mut g2_points = vec!(G2::default(); order);

        let mut secret_to_power = Fr::one();
        for i in 0..order {
            G1::mul(&mut (g1_points[i]), &g1_gen, &secret_to_power);
            G2::mul(&mut (g2_points[i]), &g2_gen, &secret_to_power);

            secret_to_power *= &secret;
        }

        Self {
            g1_gen,
            g2_gen,
            g1_points,
            g2_points,
            order
        }
    }

    pub fn is_proof_valid(&self, commitment: &G1, proof: &G1, x: &Fr, y: &Fr) -> bool {
        let secret_minus_x = &self.g2_points[1] - &(&self.g2_gen * x); // g2 * x to get x on g2
        let commitment_minus_y = commitment - &(&self.g1_gen * y);

        return self.verify_pairing(&commitment_minus_y, &self.g2_gen, proof, &secret_minus_x);
    }

    pub fn verify_pairing(&self, a1: &G1, a2: &G2, b1: &G1, b2: &G2) -> bool {
        let pairing1 = a1.pair(&a2).get_inv();
        let pairing2 = b1.pair(&b2);

        let result = (pairing1 * pairing2).get_final_exp();

        return result.is_one();
    }
}