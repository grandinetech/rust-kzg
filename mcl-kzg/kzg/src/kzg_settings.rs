use crate::data_types::{fr::Fr, g1::G1, g2::G2};
use crate::fk20_fft::FFTSettings;
use crate::kzg10::Curve;
use crate::kzg10::Polynomial;

#[derive(Clone)]
pub struct KZGSettings {
    pub fs: FFTSettings,
    pub secret1: Vec<G1>,
    pub secret2: Vec<G2>
}

impl KZGSettings {
    pub fn new(secret_g1: &Vec<G1>, secret_g2: &Vec<G2>, length: usize, fs: &FFTSettings) -> Self {
        let mut sec1: Vec<G1> = vec![];
        let mut sec2: Vec<G2> = vec![];
        for i in 0..length {
            sec1.push(secret_g1[i].clone());
            sec2.push(secret_g2[i].clone());
        }
        
        KZGSettings{
            fs: fs.clone(),
            secret1: sec1,
            secret2: sec2
        }
    }

    pub fn check_proof_single(&self, commitment: &G1, proof: &G1, x: &Fr, y: &Fr) -> bool {
        let g1_gen = G1::gen();
        let g2_gen = G2::gen();
        let secret_minus_x = &self.secret2[1] - &(&g2_gen * x); // g2 * x to get x on g2
        let commitment_minus_y = commitment - &(&g1_gen * y);

        Curve::verify_pairing(&commitment_minus_y, &g2_gen, proof, &secret_minus_x)
    }

    pub fn compute_proof_multi(&self, p: &Polynomial, x0: &Fr, n: usize) -> G1 {

        let mut divisor: Polynomial = Polynomial { coeffs: Vec::new() };
        let x_pow_n = x0.pow(n);
        divisor.coeffs.push(x_pow_n.get_neg());

        for _ in 1..n {
            divisor.coeffs.push(Fr::zero());
        }

        divisor.coeffs.push(Fr::one());
        let temp_poly = p.clone();
        let q = temp_poly.div(&divisor.coeffs).unwrap();
        let ret = q.commit(&self.secret1);

        ret
    }

    pub fn check_proof_multi(&self, commitment: &G1, proof: &G1, x: &Fr, ys: &Vec<Fr>, n: usize) -> bool {
        let mut interp = Polynomial::new(n);
        interp.coeffs = self.fs.fft_from_slice(ys, true);

        let inv_x = x.inverse();
        let mut inv_x_pow = inv_x.clone();
        for i in 1..n {
            let mut temp_fr = Fr::zero();
            Fr::mul(&mut temp_fr, &interp.coeffs[i], &inv_x_pow);
            interp.coeffs[i] = temp_fr;

            let mut temp_fr2 = Fr::zero();
            Fr::mul(&mut temp_fr2, &inv_x_pow, &inv_x);
            inv_x_pow = temp_fr2;
        }

        let x_pow = inv_x_pow.inverse();
        let g2_gen = G2::gen(); 

        let mut xn2 = G2::zero();
        G2::mul(&mut xn2, &g2_gen, &x_pow);

        let mut xn_minus_yn = G2::zero();
        G2::sub(&mut xn_minus_yn, &self.secret2[n], &xn2);

        let is1 = interp.commit(&self.secret1);

        let mut commit_minus_interp = G1::zero();
        G1::sub(&mut commit_minus_interp, commitment, &is1);

        Curve::verify_pairing(&commit_minus_interp, &G2::gen(), proof, &xn_minus_yn)

    }

    pub fn generate_trusted_setup(n: usize, secret: [u8; 32usize]) -> (Vec<G1>, Vec<G2>) {
        let g1_gen = G1::gen();
        let g2_gen = G2::gen(); 

        let mut g1_points = vec!(G1::default(); n);
        let mut g2_points = vec!(G2::default(); n);
        let secretfr = Fr::from_scalar(&secret);
        let mut secret_to_power = Fr::one();
        for i in 0..n {
            G1::mul(&mut (g1_points[i]), &g1_gen, &secret_to_power);
            G2::mul(&mut (g2_points[i]), &g2_gen, &secret_to_power);

            secret_to_power *= &secretfr;
        }

        (g1_points, g2_points)
    }
}


