use ark_ec::{
    models::short_weierstrass::SWCurveConfig as Parameters, short_weierstrass::Affine, AffineRepr,
};
use ark_ff::Field;
use ark_std::{One, Zero};

pub struct BatchAdder<P: Parameters> {
    inverse_state: P::BaseField,
    inverses: Vec<P::BaseField>,
}

impl<P: Parameters> BatchAdder<P> {
    pub fn new(max_batch_cnt: usize) -> Self {
        BatchAdder {
            inverse_state: P::BaseField::one(),
            inverses: vec![P::BaseField::one(); max_batch_cnt],
        }
    }

    /// Batch add vector dest and src, the results will be stored in dest, i.e. dest[i] = dest[i] + src[i]
    pub fn batch_add(&mut self, dest: &mut [Affine<P>], src: &[Affine<P>]) {
        assert!(
            dest.len() == src.len(),
            "length of dest and src don't match!"
        );
        assert!(dest.len() <= self.inverses.len(),
                "input length exceeds the max_batch_cnt, please increase max_batch_cnt during initialization!");

        self.reset();
        for i in 0..dest.len() {
            self.batch_add_phase_one(&dest[i], &src[i], i);
        }
        self.inverse();
        for i in (0..dest.len()).rev() {
            self.batch_add_phase_two(&mut dest[i], &src[i], i);
        }
    }

    /// Batch add vector dest and src of len entries, skipping dest_step and src_step entries each
    /// the results will be stored in dest, i.e. dest[i] = dest[i] + src[i]
    pub fn batch_add_step_n(
        &mut self,
        dest: &mut [Affine<P>],
        dest_step: usize,
        src: &[Affine<P>],
        src_step: usize,
        len: usize,
    ) {
        assert!(
            dest.len() > (len - 1) * dest_step,
            "insufficient entries in dest array"
        );
        assert!(
            src.len() > (len - 1) * src_step,
            "insufficient entries in src array"
        );
        assert!(len <= self.inverses.len(),
                "input length exceeds the max_batch_cnt, please increase max_batch_cnt during initialization!");

        self.reset();
        for i in 0..len {
            self.batch_add_phase_one(&dest[i * dest_step], &src[i * src_step], i);
        }
        self.inverse();
        for i in (0..len).rev() {
            self.batch_add_phase_two(&mut dest[i * dest_step], &src[i * src_step], i);
        }
    }

    /// Batch add vector dest[dest_index] and src[src_index] using the specified indexes in input
    /// the results will be stored in dest, i.e. dest[i] = dest[i] + src[i]
    pub fn batch_add_indexed(
        &mut self,
        dest: &mut [Affine<P>],
        dest_indexes: &[u32],
        src: &[Affine<P>],
        src_indexes: &[u32],
    ) {
        assert!(
            dest.len() >= dest_indexes.len(),
            "insufficient entries in dest array"
        );
        assert!(dest_indexes.len() <= self.inverses.len(),
                "input length exceeds the max_batch_cnt, please increase max_batch_cnt during initialization!");
        assert_eq!(
            dest_indexes.len(),
            src_indexes.len(),
            "length of dest_indexes and src_indexes don't match!"
        );

        self.reset();
        for i in 0..dest_indexes.len() {
            self.batch_add_phase_one(
                &dest[dest_indexes[i] as usize],
                &src[src_indexes[i] as usize],
                i,
            );
        }
        self.inverse();
        for i in (0..dest_indexes.len()).rev() {
            self.batch_add_phase_two(
                &mut dest[dest_indexes[i] as usize],
                &src[src_indexes[i] as usize],
                i,
            );
        }
    }

    pub fn inverse(&mut self) {
        self.inverse_state = self.inverse_state.inverse().unwrap();
    }

    pub fn reset(&mut self) {
        self.inverse_state.set_one();
    }

    /// Two-pass batch affine addition
    ///   - 1st pass calculates from left to right
    ///      - inverse_state: accumulated product of deltaX
    ///      - inverses[]: accumulated product left to a point
    ///   - call inverse()
    ///   - 2nd pass calculates from right to left
    ///      - slope s and ss from state
    ///      - inverse_state = inverse_state * deltaX
    ///      - addition result acc
    pub fn batch_add_phase_one(&mut self, p: &Affine<P>, q: &Affine<P>, idx: usize) {
        assert!(
            idx < self.inverses.len(),
            "index exceeds the max_batch_cnt, please increase max_batch_cnt during initialization!"
        );
        if p.is_zero() | q.is_zero() {
            return;
        }

        let mut delta_x = q.x - p.x;
        if delta_x.is_zero() {
            let delta_y = q.y - p.y;
            if !delta_y.is_zero() {
                // p = -q, return
                return;
            }

            // if P == Q
            // if delta_x is zero, we need to invert 2y
            delta_x = q.y + q.y;
        }

        if self.inverse_state.is_zero() {
            self.inverses[idx].set_one();
            self.inverse_state = delta_x;
        } else {
            self.inverses[idx] = self.inverse_state;
            self.inverse_state *= delta_x
        }
    }

    /// should call inverse() between phase_one and phase_two
    pub fn batch_add_phase_two(&mut self, p: &mut Affine<P>, q: &Affine<P>, idx: usize) {
        // assert_lt!(
        //     idx,
        //     self.inverses.len(),
        //     "index exceeds the max_batch_cnt, please increase max_batch_cnt during initialization!"
        // );
        if p.is_zero() | q.is_zero() {
            if !q.is_zero() {
                *p = *q;
            }
            return;
        }

        let mut _inverse = self.inverses[idx];
        _inverse *= self.inverse_state;

        let mut delta_x = q.x - p.x;
        let mut delta_y = q.y - p.y;

        if delta_x.is_zero() {
            if !delta_y.is_zero() {
                // p = -q, result should be pt at infinity
                p.x.set_zero();
                p.y.set_zero();
                p.infinity = true;
                return;
            }
            // Otherwise, p = q, and it's point doubling
            // Processing is almost the same, except s=3*affine.x^2 / 2*affine.y

            // set delta_y = 3*q.x^2
            delta_y = q.x.square();
            delta_y = delta_y + delta_y + delta_y;

            delta_x = q.y.double();
        }

        // get the state ready for the next iteration
        self.inverse_state *= delta_x;

        let s = delta_y * _inverse;
        let ss = s * s;
        p.x = ss - q.x - p.x;
        delta_x = q.x - p.x;
        p.y = s * delta_x;
        p.y -= q.y;
    }
}

// #[cfg(test)]
// mod batch_add_tests {
//     use super::*;
//     use ark_bls12_381::G1Affine;
//     use ark_ec::{AffineRepr};
//     use ark_std::UniformRand;
//     use std::ops::Add;

//     #[test]
//     fn test_phase_one_zero_or_neg() {
//         let mut batch_adder = BatchAdder::new(4);
//         batch_adder.batch_add_phase_one(&G1Affine::zero(), &G1Affine::zero(), 0);

//         let mut rng = ark_std::test_rng();
//         let p = <G1Affine as AffineRepr>::Group::rand(&mut rng);
//         let p_affine = G1Affine::from(p);
//         let mut neg_p_affine = p_affine;
//         neg_p_affine.y = -neg_p_affine.y;

//         batch_adder.batch_add_phase_one(&p_affine, &neg_p_affine, 0);
//     }

//     #[test]
//     fn test_phase_one_p_add_p() {
//         let mut batch_adder = BatchAdder::new(4);
//         let mut rng = ark_std::test_rng();
//         let prj = <G1Affine as AffineRepr>::Group::rand(&mut rng);
//         let p = G1Affine::from(prj);
//         let acc = p;

//         batch_adder.batch_add_phase_one(&acc, &p, 0);
//         assert!(batch_adder.inverses[0].is_one());
//         assert_eq!(batch_adder.inverse_state, p.y + p.y);
//     }

//     #[test]
//     fn test_phase_one_p_add_q() {
//         let mut batch_adder = BatchAdder::new(4);
//         let mut rng = ark_std::test_rng();
//         let p_prj = <G1Affine as AffineRepr>::Group::rand(&mut rng);
//         let q_prj = <G1Affine as AffineRepr>::Group::rand(&mut rng);
//         let p = G1Affine::from(p_prj);
//         let q = G1Affine::from(q_prj);

//         batch_adder.batch_add_phase_one(&p, &q, 0);
//         assert!(batch_adder.inverses[0].is_one());
//         assert_eq!(batch_adder.inverse_state, q.x - p.x);
//     }

//     #[test]
//     fn test_phase_one_p_add_q_twice() {
//         let mut batch_adder = BatchAdder::new(4);
//         let mut rng = ark_std::test_rng();
//         let p_prj = <G1Affine as AffineRepr>::Group::rand(&mut rng);
//         let q_prj = <G1Affine as AffineRepr>::Group::rand(&mut rng);
//         let p = G1Affine::from(p_prj);
//         let q = G1Affine::from(q_prj);

//         batch_adder.batch_add_phase_one(&p, &q, 0);
//         batch_adder.batch_add_phase_one(&p, &q, 0);
//         assert_eq!(batch_adder.inverses[0], q.x - p.x);
//         assert_eq!(batch_adder.inverse_state, (q.x - p.x) * (q.x - p.x));
//     }

//     #[test]
//     fn test_phase_two_zero_add_p() {
//         let mut batch_adder = BatchAdder::new(4);
//         let mut rng = ark_std::test_rng();
//         let p_prj = <G1Affine as AffineRepr>::Group::rand(&mut rng);
//         let p = G1Affine::from(p_prj);
//         let mut acc = G1Affine::zero();
//         batch_adder.batch_add_phase_two(&mut acc, &p, 0);
//         assert_eq!(acc, p);
//     }

//     #[test]
//     fn test_phase_two_p_add_neg() {
//         let mut batch_adder = BatchAdder::new(4);

//         let mut rng = ark_std::test_rng();
//         let p_prj = <G1Affine as AffineRepr>::Group::rand(&mut rng);
//         let mut acc = G1Affine::from(p_prj);
//         let mut p = acc;
//         p.y = -p.y;

//         batch_adder.batch_add_phase_two(&mut acc, &p, 0);
//         assert_eq!(acc, G1Affine::zero());
//     }

//     #[test]
//     fn test_phase_two_p_add_q() {
//         let mut batch_adder = BatchAdder::new(4);

//         let mut rng = ark_std::test_rng();
//         let acc_prj = <G1Affine as AffineRepr>::Group::rand(&mut rng);
//         let mut acc = G1Affine::from(acc_prj);
//         let mut p = acc;
//         p.x = p.x + p.x;

//         batch_adder.inverses[0] = (p.x - acc.x).inverse().unwrap();
//         batch_adder.batch_add_phase_two(&mut acc, &p, 0);
//         assert_eq!(acc, G1Affine::from(acc_prj.add(&p)));
//     }

//     #[test]
//     fn test_phase_two_p_add_p() {
//         let mut batch_adder = BatchAdder::new(4);

//         let mut rng = ark_std::test_rng();
//         let acc_prj = <G1Affine as AffineRepr>::Group::rand(&mut rng);
//         let mut acc = G1Affine::from(acc_prj);
//         let p = acc;

//         batch_adder.inverses[0] = (p.y + p.y).inverse().unwrap();
//         batch_adder.batch_add_phase_two(&mut acc, &p, 0);
//         assert_eq!(acc, G1Affine::from(acc_prj).add(p));
//     }

//     #[test]
//     fn test_batch_add() {
//         let mut batch_adder = BatchAdder::new(10);

//         let mut rng = ark_std::test_rng();
//         let mut buckets: Vec<G1Affine> = (0..10)
//             .map(|_| G1Affine::from(<G1Affine as AffineRepr>::Group::rand(&mut rng)))
//             .collect();
//         let points: Vec<G1Affine> = (0..10)
//             .map(|_| G1Affine::from(<G1Affine as AffineRepr>::Group::rand(&mut rng)))
//             .collect();

//         let tmp = buckets.clone();
//         batch_adder.batch_add(&mut buckets, &points);

//         for i in 0..10 {
//             assert_eq!(buckets[i], tmp[i].add(points[i]));
//         }
//     }

//     #[test]
//     fn test_batch_add_step_n() {
//         let mut batch_adder = BatchAdder::new(10);

//         let mut rng = ark_std::test_rng();
//         let mut buckets: Vec<G1Affine> = (0..10)
//             .map(|_| G1Affine::from(<G1Affine as AffineRepr>::Group::rand(&mut rng)))
//             .collect();
//         let points: Vec<G1Affine> = (0..10)
//             .map(|_| G1Affine::from(<G1Affine as AffineRepr>::Group::rand(&mut rng)))
//             .collect();

//         let tmp = buckets.clone();
//         batch_adder.batch_add_step_n(&mut buckets, 1, &points, 2, 3);

//         for i in 0..3 {
//             assert_eq!(buckets[i], tmp[i].add(points[i * 2]));
//         }
//     }

//     #[test]
//     fn test_batch_add_indexed() {
//         let mut batch_adder = BatchAdder::new(10);
//         let mut rng = ark_std::test_rng();

//         let mut buckets: Vec<G1Affine> = (0..10)
//             .map(|_| G1Affine::from(<G1Affine as AffineRepr>::Group::rand(&mut rng)))
//             .collect();
//         let points: Vec<G1Affine> = (0..10)
//             .map(|_| G1Affine::from(<G1Affine as AffineRepr>::Group::rand(&mut rng)))
//             .collect();

//         let tmp = buckets.clone();
//         batch_adder.batch_add_indexed(&mut buckets, &[0, 2, 4], &points, &[0, 2, 4]);

//         for i in (0..5).step_by(2) {
//             assert_eq!(buckets[i], tmp[i].add(points[i]));
//         }
//     }

//     #[test]
//     fn test_batch_add_indexed_single_bucket() {
//         let mut batch_adder = BatchAdder::new(1);
//         let mut rng = ark_std::test_rng();

//         let mut buckets: Vec<G1Affine> = vec![G1Affine::from(
//             <G1Affine as AffineRepr>::Group::rand(&mut rng),
//         )];
//         let points: Vec<G1Affine> = vec![G1Affine::from(
//             <G1Affine as AffineRepr>::Group::rand(&mut rng),
//         )];

//         let tmp = buckets.clone();
//         batch_adder.batch_add_indexed(&mut buckets, &[0], &points, &[0]);

//         assert_eq!(buckets[0], tmp[0].add(points[0]));
//     }
// }
