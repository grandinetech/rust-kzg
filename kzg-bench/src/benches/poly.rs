use criterion::Criterion;
use kzg::{Fr, Poly};

pub fn bench_new_poly_div<TFr: Fr, TPoly: Poly<TFr>>(c: &mut Criterion) {
    for scale in 6..16 {
        let dividend_length = 1 << scale;
        let divisor_length = dividend_length / 2;

        let mut dividend = TPoly::new(dividend_length).unwrap();
        let mut divisor = TPoly::new(divisor_length).unwrap();

        // Randomize the polynomials' coefficients
        for i in 0..dividend.len() {
            dividend.set_coeff_at(i, &TFr::rand());
        }
        for i in 0..divisor.len() {
            divisor.set_coeff_at(i, &TFr::rand());
        }

        // Ensure that the polynomials' orders correspond to their lengths
        if dividend.get_coeff_at(dividend.len() - 1).is_zero() {
            dividend.set_coeff_at(dividend.len() - 1, &TFr::one());
        }
        if divisor.get_coeff_at(divisor.len() - 1).is_zero() {
            divisor.set_coeff_at(divisor.len() - 1, &TFr::one());
        }

        let id = format!("bench_new_poly_div scale: '{}'", scale);
        c.bench_function(&id, |b| b.iter(|| {
            let mut divided_poly = dividend.div(&mut divisor).unwrap();
            divided_poly.destroy();
        }));

        dividend.destroy();
        divisor.destroy();
    }
}