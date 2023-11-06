use ark_bls12_381::g1::Config as G1Parameters;
use ark_bls12_381::G1Affine;
use ark_ec::{
    models::short_weierstrass::SWCurveConfig as Parameters,
    short_weierstrass::{Affine, Projective},
    CurveGroup, Group,
};
use ark_ec::AffineRepr;
use ark_std::Zero;
use std::{any::TypeId, ops::AddAssign};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use crate::{
    arkmsm::batch_adder::BatchAdder,
    arkmsm::bitmap::Bitmap,
    arkmsm::glv::endomorphism,
    arkmsm::types::{GROUP_SIZE, GROUP_SIZE_IN_BITS},
};

pub struct BucketMSM<P: Parameters> {
    num_windows: u32,
    window_bits: u32,
    bucket_bits: u32,
    max_batch_cnt: u32, // max slices allowed in a batch
    max_collision_cnt: u32,
    buckets: Vec<Affine<P>>, // size (num_windows << window_bits) * 2

    // current batch state
    bitmap: Bitmap,
    batch_buckets_and_points: Vec<(u32, u32)>,
    collision_buckets_and_points: Vec<(u32, Affine<P>)>,
    cur_points: Vec<Affine<P>>, // points of current batch, size batch_size

    // batch affine adder
    batch_adder: BatchAdder<P>,
}

impl<P: Parameters> BucketMSM<P> {
    pub fn new(
        scalar_bits: u32,
        window_bits: u32,
        max_batch_cnt: u32,     // default: 4096
        max_collision_cnt: u32, // default: 128
    ) -> BucketMSM<P> {
        let num_windows = (scalar_bits + window_bits - 1) / window_bits;
        let batch_size = std::cmp::max(8192, max_batch_cnt);
        let bucket_bits = window_bits - 1; // half buckets needed because of signed-bucket-index
        let bucket_size = num_windows << bucket_bits;
        // size of batch_adder will be the max of batch_size and num_windows * groups per window
        let batch_adder_size = std::cmp::max(batch_size, bucket_size >> GROUP_SIZE_IN_BITS);

        BucketMSM {
            num_windows,
            window_bits,
            bucket_bits,
            max_batch_cnt,
            max_collision_cnt,
            buckets: vec![Affine::<P>::zero(); bucket_size as usize],

            bitmap: Bitmap::new(bucket_size as usize / 32),
            batch_buckets_and_points: Vec::with_capacity(batch_size as usize),
            collision_buckets_and_points: Vec::with_capacity(max_collision_cnt as usize),
            cur_points: vec![Affine::<P>::zero(); batch_size as usize],

            batch_adder: BatchAdder::new(batch_adder_size as usize),
        }
    }

    pub fn process_point_and_slices_glv(
        &mut self,
        point: &Affine<P>,
        normal_slices: &[u32],
        phi_slices: &[u32],
        is_neg_scalar: bool,
        is_neg_normal: bool,
    ) {
        assert_eq!(
            TypeId::of::<P>(),
            TypeId::of::<G1Parameters>(),
            "glv is only supported for ark_bls12_381"
        );
        assert!(
            self.num_windows as usize == normal_slices.len()
                && normal_slices.len() == phi_slices.len(),
            "slice len check failed: normal_slices {}, phi_slices {}, num_windows {}",
            normal_slices.len(),
            phi_slices.len(),
            self.num_windows
        );

        let mut p = *point; // copy

        if is_neg_scalar {
            p.y = -p.y
        };
        if is_neg_normal {
            p.y = -p.y
        };

        self.cur_points.push(p);
        for (win, normal_slice) in normal_slices.iter().enumerate() {
            if (*normal_slice as i32) > 0 {
                let bucket_id = (win << self.bucket_bits) as u32 + normal_slice - 1;
                self._process_slices(bucket_id, self.cur_points.len() as u32 - 1);
            }
        }

        p.y = -p.y;

        self.cur_points.push(p);
        for (win, normal_slice) in normal_slices.iter().enumerate() {
            if (*normal_slice as i32) < 0 {
                let slice = normal_slice & 0x7FFFFFFF;
                if slice > 0 {
                    let bucket_id = (win << self.bucket_bits) as u32 + slice - 1;
                    self._process_slices(bucket_id, self.cur_points.len() as u32 - 1);
                }
            }
        }

        // process phi slices
        p.y = -p.y;
        if is_neg_normal {
            p.y = -p.y;
        }

        // this isn't the cleanest of doing this, we'd better figure out a way to do this at compile time
        let p_g1: &mut G1Affine = unsafe { &mut *(std::ptr::addr_of_mut!(p) as *mut G1Affine) };
        endomorphism(p_g1);

        self.cur_points.push(p);
        for (win, phi_slice) in phi_slices.iter().enumerate() {
            if (*phi_slice as i32) > 0 {
                let bucket_id = (win << self.bucket_bits) as u32 + phi_slice - 1;
                self._process_slices(bucket_id, self.cur_points.len() as u32 - 1);
            }
        }

        p.y = -p.y;

        self.cur_points.push(p);
        for (win, phi_slice) in phi_slices.iter().enumerate() {
            if (*phi_slice as i32) < 0 {
                let slice = phi_slice & 0x7FFFFFFF;
                if slice > 0 {
                    let bucket_id = (win << self.bucket_bits) as u32 + slice - 1;
                    self._process_slices(bucket_id, self.cur_points.len() as u32 - 1);
                }
            }
        }
    }

    pub fn process_point_and_slices(&mut self, point: &Affine<P>, slices: &[u32]) {
        assert!(
            self.num_windows as usize == slices.len(),
            "slices.len() {} should equal num_windows {}",
            slices.len(),
            self.num_windows
        );

        self.cur_points.push(*point);
        for (win, slice) in slices.iter().enumerate() {
            if (*slice as i32) > 0 {
                let bucket_id = (win << self.bucket_bits) as u32 + slice - 1; // skip slice == 0
                self._process_slices(bucket_id, self.cur_points.len() as u32 - 1);
            }
        }

        let mut neg_p = *point;
        neg_p.y = -neg_p.y;

        self.cur_points.push(neg_p);
        for (win, slice) in slices.iter().enumerate() {
            if (*slice as i32) < 0 {
                let slice = slice & 0x7FFFFFFF;
                if slice > 0 {
                    let bucket_id = (win << self.bucket_bits) as u32 + slice - 1; // skip slice == 0
                    self._process_slices(bucket_id, self.cur_points.len() as u32 - 1);
                }
            }
        }
    }

    pub fn process_complete(&mut self) {
        self._process_batch();
        while !(self.collision_buckets_and_points.is_empty()
            && self.batch_buckets_and_points.is_empty())
        {
            self._process_batch();
        }
    }

    fn _process_slices(&mut self, bucket_id: u32, point_idx: u32) {
        if !self.bitmap.test_and_set(bucket_id) {
            // if no collision found, add point to current batch
            self.batch_buckets_and_points.push((bucket_id, point_idx));
        } else {
            self.collision_buckets_and_points
                .push((bucket_id, self.cur_points[point_idx as usize]));
        }

        if self.collision_buckets_and_points.len() as u32 >= self.max_collision_cnt
            || self.batch_buckets_and_points.len() as u32 >= self.max_batch_cnt
        {
            self._process_batch();
        }
    }

    fn _process_batch(&mut self) {
        if self.batch_buckets_and_points.is_empty() {
            return;
        }
        // batch addition
        let (bucket_ids, point_idxs): (Vec<u32>, Vec<u32>) = self
            .batch_buckets_and_points
            .iter()
            .map(|(b, p)| (*b, *p))
            .unzip();
        self.batch_adder.batch_add_indexed(
            &mut self.buckets,
            &bucket_ids,
            &self.cur_points,
            &point_idxs,
        );
        // clean up current batch
        self.bitmap.clear();
        self.batch_buckets_and_points.clear();
        // memorize the last point which is the current processing point and we need to
        // push it back to the cur_points list since we're processing slices in a for loop
        let slicing_point = self.cur_points.pop();
        self.cur_points.clear();

        let mut next_pos = 0;
        for i in 0..self.collision_buckets_and_points.len() {
            let (bucket_id, point) = self.collision_buckets_and_points[i];
            if self.bitmap.test_and_set(bucket_id) {
                // collision found
                self.collision_buckets_and_points.swap(next_pos, i);
                next_pos += 1;
            } else {
                self.batch_buckets_and_points
                    .push((bucket_id, self.cur_points.len() as u32));
                self.cur_points.push(point);
            }
        }
        self.collision_buckets_and_points.truncate(next_pos);
        self.cur_points.push(slicing_point.unwrap());
    }

    pub fn batch_reduce(&mut self) -> Projective<P> {
        let window_starts: Vec<_> = (0..self.num_windows as usize).collect();
        let num_groups =
            (self.num_windows as usize) << (self.bucket_bits as usize - GROUP_SIZE_IN_BITS);
        let mut running_sums: Vec<_> = vec![Affine::<P>::zero(); num_groups];
        let mut sum_of_sums: Vec<_> = vec![Affine::<P>::zero(); num_groups];

        // calculate running sum and sum of sum for each group
        for i in (0..GROUP_SIZE).rev() {
            // running sum
            self.batch_adder.batch_add_step_n(
                &mut running_sums,
                1,
                &self.buckets[i..],
                GROUP_SIZE,
                num_groups,
            );
            // sum of sum
            self.batch_adder.batch_add(&mut sum_of_sums, &running_sums);
        }

        let sum_by_window: Vec<Projective<P>> = ark_std::cfg_into_iter!(window_starts)
            .map(|w_start| {
                let group_start = w_start << (self.bucket_bits as usize - GROUP_SIZE_IN_BITS);
                let group_end = (w_start + 1) << (self.bucket_bits as usize - GROUP_SIZE_IN_BITS);
                self.inner_window_reduce(
                    &running_sums[group_start..group_end],
                    &sum_of_sums[group_start..group_end],
                )
            })
            .collect();

        self.intra_window_reduce(&sum_by_window)
    }

    fn inner_window_reduce(
        &self,
        running_sums: &[Affine<P>],
        sum_of_sums: &[Affine<P>],
    ) -> Projective<P> {
        self.calc_sum_of_sum_total(sum_of_sums) + self.calc_running_sum_total(running_sums)
    }

    fn calc_running_sum_total(&self, running_sums: &[Affine<P>]) -> Projective<P> {
        let mut running_sum_total = Projective::<P>::zero();
        for (i, running_sum) in running_sums.iter().enumerate().skip(1) {
            for _ in 0..i {
                running_sum_total.add_assign(running_sum);
            }
        }

        for _ in 0..GROUP_SIZE_IN_BITS {
            running_sum_total.double_in_place();
        }
        running_sum_total
    }

    fn calc_sum_of_sum_total(&self, sum_of_sums: &[Affine<P>]) -> Projective<P> {
        let mut sum = Projective::<P>::zero();
        sum_of_sums.iter().for_each(|p| sum.add_assign(p));
        sum
    }

    fn intra_window_reduce(&self, window_sums: &[Projective<P>]) -> Projective<P> {
        // We store the sum for the lowest window.
        let lowest = *window_sums.first().unwrap();

        // We're traversing windows from high to low.
        lowest
            + window_sums.iter().skip(1).rev().fold(
                Projective::<P>::zero(),
                |mut total, sum_i| {
                    total += sum_i;
                    for _ in 0..self.window_bits {
                        total.double_in_place();
                    }
                    total
                },
            )
    }
}

#[cfg(test)]
mod bucket_msm_tests {
    use super::*;
    use ark_bls12_381::{G1Affine, G1Projective};
    use ark_std::UniformRand;

    #[test]
    fn test_process_point_and_slices_deal_two_points() {
        let window_bits = 15u32;
        let mut bucket_msm = BucketMSM::new(30u32, window_bits, 128u32, 4096u32);
        let mut rng = ark_std::test_rng();
        let p_prj = G1Projective::rand(&mut rng);
        let q_prj = G1Projective::rand(&mut rng);
        let p = G1Affine::from(p_prj);
        let q = G1Affine::from(q_prj);

        bucket_msm.process_point_and_slices(&p, &[1u32, 3u32]);
        bucket_msm.process_point_and_slices(&q, &[2u32, 3u32]);
        bucket_msm.process_complete();
        assert_eq!(bucket_msm.buckets[0], p);
        assert_eq!(bucket_msm.buckets[1], q);
        assert_eq!(bucket_msm.buckets[2 + (1 << bucket_msm.bucket_bits)], p + q);
    }

    #[test]
    fn test_process_point_and_slices_deal_three_points() {
        let window_bits = 15u32;
        let mut bucket_msm = BucketMSM::new(45u32, window_bits, 128u32, 4096u32);
        let mut rng = ark_std::test_rng();
        let p_prj = G1Projective::rand(&mut rng);
        let q_prj = G1Projective::rand(&mut rng);
        let r_prj = G1Projective::rand(&mut rng);
        let p = G1Affine::from(p_prj);
        let q = G1Affine::from(q_prj);
        let r = G1Affine::from(r_prj);

        bucket_msm.process_point_and_slices(&p, &[1u32, 3u32, 4u32]);
        bucket_msm.process_point_and_slices(&q, &[2u32, 3u32, 4u32]);
        bucket_msm.process_point_and_slices(&r, &[2u32, 3u32, 5u32]);
        bucket_msm.process_complete();
        assert_eq!(bucket_msm.buckets[0], p);
        assert_eq!(bucket_msm.buckets[1], q + r);
        assert_eq!(
            bucket_msm.buckets[2 + (1 << bucket_msm.bucket_bits)],
            p + q + r
        );
        assert_eq!(bucket_msm.buckets[3 + (2 << bucket_msm.bucket_bits)], p + q);
        assert_eq!(bucket_msm.buckets[4 + (2 << bucket_msm.bucket_bits)], r);
    }

    #[test]
    fn test_process_point_and_slices_glv_deal_two_points() {
        let window_bits = 15u32;
        let mut bucket_msm = BucketMSM::new(30u32, window_bits, 128u32, 4096u32);
        let mut rng = ark_std::test_rng();
        let p_prj = G1Projective::rand(&mut rng);
        let q_prj = G1Projective::rand(&mut rng);
        let mut p = G1Affine::from(p_prj);
        let mut q = G1Affine::from(q_prj);

        bucket_msm.process_point_and_slices_glv(&p, &[1u32, 3u32], &[4u32, 6u32], false, false);
        bucket_msm.process_point_and_slices_glv(&q, &[2u32, 3u32], &[5u32, 6u32], false, false);
        bucket_msm.process_complete();
        assert_eq!(bucket_msm.buckets[0], p);
        assert_eq!(bucket_msm.buckets[1], q);
        assert_eq!(bucket_msm.buckets[2 + (1 << bucket_msm.bucket_bits)], p + q);

        endomorphism(&mut p);
        endomorphism(&mut q);
        assert_eq!(bucket_msm.buckets[3], p);
        assert_eq!(bucket_msm.buckets[4], q);
        assert_eq!(bucket_msm.buckets[5 + (1 << bucket_msm.bucket_bits)], p + q);
    }
}
