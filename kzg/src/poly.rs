//! This module provides an implementation of polinomials over bls12_381::blsScalar

pub type blsScalar = bls12_381::Scalar;

/// A polinomial with bl12_381::blsScalar factors
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Poly(pub(crate) Vec<blsScalar>);

impl Poly {
    /// Creates a new Poly from its `coeffs`icients, first element the coefficient for x^0
    /// for safetly, input value is normalized (trailing zeroes are removed)
    pub fn new(coeffs: Vec<blsScalar>) -> Self {
        let mut poly = Poly(coeffs);
        poly.normalize();
        poly
    }

    /// Creates a new polinomial where the `coeffs` fits in u64 values
    pub fn from(coeffs: &[u64]) -> Self {
        Poly::new(
            coeffs
                .iter()
                .map(|n| blsScalar::from(*n))
                .collect::<Vec<blsScalar>>(),
        )
    }
    /// Returns p(x)=0
    pub fn zero() -> Self {
        Poly(vec![blsScalar::zero()])
    }

    /// Returns p(x)=1
    pub fn one() -> Self {
        Poly(vec![blsScalar::one()])
    }

    /// Creates a polinomial that contains a set of `p` points, by using lagrange
    /// see https://en.wikipedia.org/wiki/Lagrange_polynomial
    /// # Examples
    /// ```
    ///    use crate::kzg::{Poly, BlsScalar};
    ///    // f(x)=x is a polinomial that fits in (1,1), (2,2) points
    ///    assert_eq!(
    ///      Poly::lagrange(&vec![
    ///          (BlsScalar::from(1), BlsScalar::from(1)),
    ///          (BlsScalar::from(2), BlsScalar::from(2))
    ///      ]),
    ///      Poly::from(&[0, 1]) // f(x) = x
    ///    );
    /// ```
    pub fn lagrange(p: &[(blsScalar, blsScalar)]) -> Self {
        let k = p.len();
        let mut l = Poly::zero();
        for j in 0..k {
            let mut l_j = Poly::one();
            for i in 0..k {
                if i != j {
                    let c = (p[j].0 - p[i].0).invert().unwrap();
                    l_j = &l_j * &Poly::new(vec![-(c * p[i].0), c]);
                }
            }
            l += &(&l_j * &p[j].1);
        }
        l
    }

    /// Evals the polinomial at the desired point
    /// # Examples
    /// ```
    ///    use crate::kzg::{Poly, BlsScalar};
    ///    // check that (x^2+2x+1)(2) = 9
    ///    assert_eq!(
    ///      Poly::from(&[1, 2, 1]).eval(&BlsScalar::from(2)),
    ///      BlsScalar::from(9));
    /// ```
    pub fn eval(&self, x: &blsScalar) -> blsScalar {
        let mut x_pow = blsScalar::one();
        let mut y = self.0[0];
        for (i, _) in self.0.iter().enumerate().skip(1) {
            x_pow *= x;
            y += &(x_pow * self.0[i]);
        }
        y
    }

    /// Evals the polinomial suplying the `x_pows` x^0, x^1, x^2
    pub fn eval_with_pows(&self, x_pow: &[blsScalar]) -> blsScalar {
        let mut y = self.0[0];
        for (i, _) in self.0.iter().enumerate() {
            y += &(x_pow[i] * self.0[i]);
        }
        y
    }

    /// Returns the degree of the polinominal, degree(x+1) = 1
    pub fn degree(&self) -> usize {
        self.0.len() - 1
    }

    /// Normalizes the coefficients, removing ending zeroes
    /// # Examples
    /// ```
    ///    use crate::kzg::Poly;
    ///    let mut p1 = Poly::from(&[1, 0, 0, 0]);
    ///    p1.normalize();
    ///    assert_eq!(p1, Poly::from(&[1]));
    /// ```
    pub fn normalize(&mut self) {
        if self.0.len() > 1 && self.0[self.0.len() - 1] == blsScalar::zero() {
            let zero = blsScalar::zero();
            let first_non_zero = self.0.iter().rev().position(|p| p != &zero);
            if let Some(first_non_zero) = first_non_zero {
                self.0.resize(self.0.len() - first_non_zero, blsScalar::zero());
            } else {
                self.0.resize(1, blsScalar::zero());
            }
        }
    }

    /// Returns if p(x)=0
    /// # Examples
    /// ```
    ///    use kzg::Poly;
    ///    assert!(Poly::zero().is_zero());
    ///    assert!(!Poly::one().is_zero());
    /// ```
    pub fn is_zero(&self) -> bool {
        self.0.len() == 1 && self.0[0] == blsScalar::zero()
    }

    /// Sets the `i`-th coefficient to the selected `p` value
    /// # Examples
    /// ``
    ///   use crate::kzg::{Poly, BlsScalar};
    ///   let mut p007 = Poly::zero();
    ///   p007.set(2, blsScalar::from(7));
    ///   assert_eq!(p007, Poly::from(&[0, 0, 7]));
    ///  ```
    pub fn set(&mut self, i: usize, p: blsScalar) {
        if self.0.len() < i + 1 {
            self.0.resize(i + 1, blsScalar::zero());
        }
        self.0[i] = p;
        self.normalize();
    }

    /// Returns the `i`-th coefficient
    /// # Examples
    /// ```
    ///   use crate::kzg::{Poly, BlsScalar};
    ///   let mut p007 = Poly::zero();
    ///   p007.set(2, BlsScalar::from(7));
    ///   assert_eq!(p007.get(2), Some(&BlsScalar::from(7)));
    ///   assert_eq!(p007.get(3), None);
    ///  ```
    pub fn get(&mut self, i: usize) -> Option<&blsScalar> {
        self.0.get(i)
    }
}

impl std::fmt::Display for Poly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first: bool = true;
        for i in (0..=self.degree()).rev() {
            let bi_n = num_bigint::BigUint::from_bytes_le(&self.0[i].to_bytes()).to_str_radix(10);
            let bi_inv =
                num_bigint::BigUint::from_bytes_le(&(-self.0[i]).to_bytes()).to_str_radix(10);

            if bi_n == "0" {
                continue;
            }

            if bi_inv.len() < 20 && bi_n.len() > 20 {
                if bi_inv == "1" && i != 0 {
                    write!(f, "-")?;
                } else {
                    write!(f, "-{}", bi_inv)?;
                }
            } else {
                if !first {
                    write!(f, "+")?;
                }
                if i == 0 || bi_n != "1" {
                    write!(f, "{}", bi_n)?;
                }
            }
            if i >= 1 {
                write!(f, "x")?;
            }
            if i >= 2 {
                write!(f, "^{}", i)?;
            }
            first = false;
        }
        Ok(())
    }
}
impl std::ops::AddAssign<&Poly> for Poly {
    fn add_assign(&mut self, rhs: &Poly) {
        for n in 0..std::cmp::max(self.0.len(), rhs.0.len()) {
            if n >= self.0.len() {
                self.0.push(rhs.0[n]);
            } else if n < self.0.len() && n < rhs.0.len() {
                self.0[n] += rhs.0[n];
            }
        }
        self.normalize();
    }
}

impl std::ops::AddAssign<&blsScalar> for Poly {
    fn add_assign(&mut self, rhs: &blsScalar) {
        self.0[0] += rhs;
    }
}

impl std::ops::SubAssign<&Poly> for Poly {
    fn sub_assign(&mut self, rhs: &Poly) {
        for n in 0..std::cmp::max(self.0.len(), rhs.0.len()) {
            if n >= self.0.len() {
                self.0.push(rhs.0[n]);
            } else if n < self.0.len() && n < rhs.0.len() {
                self.0[n] -= rhs.0[n];
            }
        }
        self.normalize();
    }
}

impl std::ops::Mul<&Poly> for &Poly {
    type Output = Poly;
    fn mul(self, rhs: &Poly) -> Self::Output {
        let mut mul: Vec<blsScalar> = std::iter::repeat(blsScalar::zero())
            .take(self.0.len() + rhs.0.len() - 1)
            .collect();
        for n in 0..self.0.len() {
            for m in 0..rhs.0.len() {
                mul[n + m] += self.0[n] * rhs.0[m];
            }
        }
        Poly(mul)
    }
}

impl std::ops::Mul<&blsScalar> for &Poly {
    type Output = Poly;
    fn mul(self, rhs: &blsScalar) -> Self::Output {
        if rhs == &blsScalar::zero() {
            Poly::zero()
        } else {
            Poly(self.0.iter().map(|v| v * rhs).collect::<Vec<_>>())
        }
    }
}

impl std::ops::Div for Poly {
    type Output = (Poly, Poly);

    fn div(self, rhs: Poly) -> Self::Output {
        let (mut q, mut r) = (Poly::zero(), self);
        while !r.is_zero() && r.degree() >= rhs.degree() {
            let lead_r = r.0[r.0.len() - 1];
            let lead_d = rhs.0[rhs.0.len() - 1];
            let mut t = Poly::zero();
            t.set(r.0.len() - rhs.0.len(), lead_r * lead_d.invert().unwrap());
            q += &t;
            r -= &(&rhs * &t);
        }
        (q, r)
    }
}

#[test]
fn test_poly_add() {
    let mut p246 = Poly::from(&[1, 2, 3]);
    p246 += &Poly::from(&[1, 2, 3]);
    assert_eq!(p246, Poly::from(&[2, 4, 6]));

    let mut p24645 = Poly::from(&[1, 2, 3]);
    p24645 += &Poly::from(&[1, 2, 3, 4, 5]);
    assert_eq!(p24645, Poly::from(&[2, 4, 6, 4, 5]));

    let mut p24646 = Poly::from(&[1, 2, 3, 4, 6]);
    p24646 += &Poly::from(&[1, 2, 3]);
    assert_eq!(p24646, Poly::from(&[2, 4, 6, 4, 6]));
}

#[test]
fn test_poly_sub() {
    let mut p0 = Poly::from(&[1, 2, 3]);
    p0 -= &Poly::from(&[1, 2, 3]);
    assert_eq!(p0, Poly::from(&[0]));

    let mut p003 = Poly::from(&[1, 2, 3]);
    p003 -= &Poly::from(&[1, 2]);
    assert_eq!(p003, Poly::from(&[0, 0, 3]));
}

#[test]
fn test_poly_mul() {
    assert_eq!(
        &Poly::from(&[5, 0, 10, 6]) * &Poly::from(&[1, 2, 4]),
        Poly::from(&[5, 10, 30, 26, 52, 24])
    );
}

#[test]
fn test_div() {
    fn do_test(n: Poly, d: Poly) {
        let (q, r) = n.clone() / d.clone();
        let mut n2 = &q * &d;
        n2 += &r;
        assert_eq!(n, n2);
    }

    do_test(Poly::from(&[1]), Poly::from(&[1, 1]));
    do_test(Poly::from(&[1, 1]), Poly::from(&[1, 1]));
    do_test(Poly::from(&[1, 2, 1]), Poly::from(&[1, 1]));
    do_test(
        Poly::from(&[1, 2, 1, 2, 5, 8, 1, 9]),
        Poly::from(&[1, 1, 5, 4]),
    );
}

#[test]
fn test_print() {
    assert_eq!("x^2+2x+1", format!("{}", Poly::from(&[1, 2, 1])));
    assert_eq!("x^2+1", format!("{}", Poly::from(&[1, 0, 1])));
    assert_eq!("x^2", format!("{}", Poly::from(&[0, 0, 1])));
    assert_eq!("2x^2", format!("{}", Poly::from(&[0, 0, 2])));
    assert_eq!("-4", format!("{}", Poly::new(vec![-blsScalar::from(4)])));
    assert_eq!(
        "-4x",
        format!("{}", Poly::new(vec![blsScalar::zero(), -blsScalar::from(4)]))
    );
    assert_eq!(
        "-x-2",
        format!("{}", Poly::new(vec![-blsScalar::from(2), -blsScalar::from(1)]))
    );
    assert_eq!(
        "x-2",
        format!("{}", Poly::new(vec![-blsScalar::from(2), blsScalar::from(1)]))
    );
}

#[test]
fn test_lagrange_multi() {
    let points = vec![
        (blsScalar::from(12342), blsScalar::from(22342)),
        (blsScalar::from(2234), blsScalar::from(22222)),
        (blsScalar::from(3982394), blsScalar::from(111114)),
        (blsScalar::from(483838), blsScalar::from(444444)),
    ];
    let l = Poly::lagrange(&points);
    points.iter().for_each(|p| assert_eq!(l.eval(&p.0), p.1));
}