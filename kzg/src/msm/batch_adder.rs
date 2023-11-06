use crate::{G1Affine, G1Fp, G1};
use core::marker::PhantomData;

pub struct BatchAdder<TG1: G1, TFp: G1Fp, TG1Affine: G1Affine<TG1, TFp>> {
    pub inverse_state: TFp,
    pub inverses: Vec<TFp>,
    // Zero sized fields so that batch adder doesn't complain about unused types
    // TG1 & TG1Affine are needed for the BatchAdder impl
    phantom_g1: PhantomData<TG1>,
    phantom_affine: PhantomData<TG1Affine>,
}

impl<TG1: G1, TFp: G1Fp, TG1Affine: G1Affine<TG1, TFp>> BatchAdder<TG1, TFp, TG1Affine>
where
    TG1: G1,
{
    pub fn new(max_batch_cnt: usize) -> Self {
        BatchAdder {
            inverse_state: TFp::ONE,
            inverses: vec![TFp::ONE; max_batch_cnt],
            phantom_g1: PhantomData,
            phantom_affine: PhantomData,
        }
    }

    /// Batch add vector dest and src, the results will be stored in dest, i.e. dest[i] = dest[i] + src[i]
    pub fn batch_add(&mut self, dest: &mut [TG1Affine], src: &[TG1Affine]) {
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
        dest: &mut [TG1Affine],
        dest_step: usize,
        src: &[TG1Affine],
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
        dest: &mut [TG1Affine],
        dest_indexes: &[u32],
        src: &[TG1Affine],
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
    pub fn batch_add_phase_one(&mut self, p: &TG1Affine, q: &TG1Affine, idx: usize) {
        assert!(
            idx < self.inverses.len(),
            "index exceeds the max_batch_cnt, please increase max_batch_cnt during initialization!"
        );
        if p.is_zero() || q.is_zero() {
            return;
        }

        let mut delta_x = q.x().sub_fp(p.x());
        if delta_x.is_zero() {
            let delta_y = q.y().sub_fp(p.y());
            if !delta_y.is_zero() {
                // p = -q, return
                return;
            }

            // if P == Q
            // if delta_x is zero, we need to invert 2y
            delta_x = q.y().add_fp(q.y());
        }

        if self.inverse_state.is_zero() {
            self.inverses[idx].set_one();
            self.inverse_state = delta_x;
        } else {
            self.inverses[idx] = self.inverse_state;
            self.inverse_state.mul_assign_fp(&delta_x);
        }
    }

    /// should call inverse() between phase_one and phase_two
    pub fn batch_add_phase_two(&mut self, p: &mut TG1Affine, q: &TG1Affine, idx: usize) {
        assert!(idx < self.inverses.len(),
            "index exceeds the max_batch_cnt, please increase max_batch_cnt during initialization!"
        );
        if q.is_zero() {
            return;
        } else if p.is_zero() {
            *p = *q;
            return;
        }

        let mut _inverse = self.inverses[idx];
        _inverse.mul_assign_fp(&self.inverse_state);

        let mut delta_x = q.x().sub_fp(p.x());
        let mut delta_y = q.y().sub_fp(p.y());

        if delta_x.is_zero() {
            if !delta_y.is_zero() {
                // p = -q, result should be pt at infinity
                p.set_zero();
                return;
            }
            // Otherwise, p = q, and it's point doubling
            // Processing is almost the same, except s=3*affine.x^2 / 2*affine.y

            // set delta_y = 3*q.x^2
            delta_y = q.x().square();
            delta_y = delta_y.add_fp(&delta_y).add_fp(&delta_y);

            delta_x = q.y().double();
        }

        // get the state ready for the next iteration
        self.inverse_state.mul_assign_fp(&delta_x);

        let s = delta_y.mul_fp(&_inverse);
        let ss = s.mul_fp(&s);
        *p.x_mut() = ss.sub_fp(q.x()).sub_fp(p.x());
        delta_x = q.x().sub_fp(p.x());
        *p.y_mut() = s.mul_fp(&delta_x).sub_fp(q.y());
    }
}
