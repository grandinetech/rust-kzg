use crate::data_types::{fr::Fr, g1::G1, g2::G2};
use crate::fk20_fft::FFTSettings;
use crate::kzg10::Curve;
use crate::kzg10::Polynomial;
use crate::utilities::is_power_of_2;

#[derive(Debug, Clone, Default)]
pub struct KZGSettings {
    pub fft_settings: FFTSettings,
    pub curve: Curve,
}

impl KZGSettings {
    pub fn new_from_curve(curve: &Curve, fft_settings: &FFTSettings) -> Self {
        KZGSettings {
            fft_settings: fft_settings.clone(),
            curve: curve.clone(),
        }
    }

    pub fn new(
        secret_g1: &[G1],
        secret_g2: &[G2],
        length: usize,
        fft_settings: &FFTSettings,
    ) -> Result<Self, String> {
        if length < fft_settings.max_width {
            return Err(String::from(
                "length must be equal to or greater than fft settings max width",
            ));
        }
        if secret_g1.len() < fft_settings.max_width {
            return Err(String::from(
                "secret g1 must have a length equal to or greater than fft settings max width",
            ));
        }
        if secret_g2.len() < fft_settings.max_width {
            return Err(String::from(
                "secret g2 must have a length equal to or greater than fft settings max width",
            ));
        }

        let mut secret1: Vec<G1> = vec![];
        let mut secret2: Vec<G2> = vec![];
        for i in 0..length {
            secret1.push(secret_g1[i]);
            secret2.push(secret_g2[i]);
        }
        let curve = Curve::new2(&secret1, &secret2, length);

        Ok(KZGSettings {
            fft_settings: fft_settings.clone(),
            curve,
        })
    }

    pub fn check_proof_single(&self, commitment: &G1, proof: &G1, x: &Fr, y: &Fr) -> bool {
        self.curve.is_proof_valid(commitment, proof, x, y)
    }

    pub fn compute_proof_multi(&self, p: &Polynomial, x0: &Fr, n: usize) -> Result<G1, String> {
        if !is_power_of_2(n) {
            return Err(String::from("n must be a power of 2"));
        }

        let mut divisor = Polynomial::from_fr(vec![]);
        let x_pow_n = x0.pow(n);
        divisor.coeffs.push(x_pow_n.get_neg());

        for _ in 1..n {
            divisor.coeffs.push(Fr::zero());
        }

        divisor.coeffs.push(Fr::one());
        let temp_poly = p.clone();
        let q = temp_poly.div(&divisor.coeffs).unwrap();
        Ok(q.commit(&self.curve.g1_points).unwrap())
    }

    pub fn check_proof_multi(
        &self,
        commitment: &G1,
        proof: &G1,
        x: &Fr,
        ys: &[Fr],
        n: usize,
    ) -> Result<bool, String> {
        if !is_power_of_2(n) {
            return Err(String::from("n must be a power of 2"));
        }

        let mut interpolation_poly = Polynomial::new(n);
        interpolation_poly.coeffs = self.fft_settings.fft(ys, true).unwrap();

        let inv_x = x.inverse();
        let mut inv_x_pow = inv_x;
        for i in 1..n {
            interpolation_poly.coeffs[i] = interpolation_poly.coeffs[i] * inv_x_pow;
            inv_x_pow = inv_x_pow * inv_x;
        }

        let x_pow = inv_x_pow.inverse();
        let xn2 = &self.curve.g2_gen * &x_pow;
        let xn_minus_yn = self.curve.g2_points[n] - xn2;
        let is1 = interpolation_poly.commit(&self.curve.g1_points).unwrap();
        let commit_minus_interp = commitment - &is1;
        Ok(Curve::verify_pairing(
            &commit_minus_interp,
            &self.curve.g2_gen,
            proof,
            &xn_minus_yn,
        ))
    }

    pub fn generate_trusted_setup(n: usize, secret: [u8; 32usize]) -> (Vec<G1>, Vec<G2>) {
        let g1_gen = G1::gen();
        let g2_gen = G2::gen();

        let mut g1_points = vec![G1::default(); n];
        let mut g2_points = vec![G2::default(); n];
        let secretfr = Fr::from_bytes(&secret);
        let mut secret_to_power = Fr::one();
        for i in 0..n {
            g1_points[i] = &g1_gen * &secret_to_power;
            g2_points[i] = &g2_gen * &secret_to_power;
            secret_to_power *= secretfr.as_ref().unwrap();
        }

        (g1_points, g2_points)
    }
}
