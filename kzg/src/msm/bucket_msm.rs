use core::marker::PhantomData;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use crate::{
    cfg_into_iter,
    msm::{
        batch_adder::BatchAdder,
        bitmap::Bitmap,
        glv::endomorphism,
        types::{GROUP_SIZE, GROUP_SIZE_IN_BITS},
    },
    G1Affine, G1Fp, G1ProjAddAffine, G1,
};

pub struct BucketMSM<
    TG1: G1,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
> {
    pub num_windows: u32,
    pub window_bits: u32,
    pub bucket_bits: u32,
    pub max_batch_cnt: u32, // max slices allowed in a batch
    pub max_collision_cnt: u32,
    pub buckets: Vec<TG1Affine>, // size (num_windows << window_bits) * 2

    // current batch state
    pub bitmap: Bitmap,
    pub batch_buckets_and_points: Vec<(u32, u32)>,
    pub collision_buckets_and_points: Vec<(u32, TG1Affine)>,
    pub cur_points: Vec<TG1Affine>, // points of current batch, size batch_size

    // batch affine adder
    pub batch_adder: BatchAdder<TG1, TG1Fp, TG1Affine>,
    _p: PhantomData<TProjAddAffine>,
}

impl<
        TG1: G1,
        TG1Fp: G1Fp,
        TG1Affine: G1Affine<TG1, TG1Fp>,
        TProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
    > BucketMSM<TG1, TG1Fp, TG1Affine, TProjAddAffine>
{
    pub fn new(
        scalar_bits: u32,
        window_bits: u32,
        max_batch_cnt: u32,     // default: 4096
        max_collision_cnt: u32, // default: 128
    ) -> Self {
        // TODO: Check if these can be turned into consts
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
            buckets: vec![TG1Affine::ZERO; bucket_size as usize],

            bitmap: Bitmap::new(bucket_size as usize / 32),
            batch_buckets_and_points: Vec::with_capacity(batch_size as usize),
            collision_buckets_and_points: Vec::with_capacity(max_collision_cnt as usize),
            cur_points: vec![TG1Affine::ZERO; batch_size as usize],

            batch_adder: BatchAdder::new(batch_adder_size as usize),
            _p: PhantomData,
        }
    }

    pub fn process_point_and_slices_glv(
        &mut self,
        point: &TG1Affine,
        normal_slices: &[u32],
        phi_slices: &[u32],
        is_neg_scalar: bool,
        is_neg_normal: bool,
    ) {
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
            p.y_mut().neg_assign();
        };
        // TODO: Can be replaced with XOR?
        if is_neg_normal {
            p.y_mut().neg_assign();
        };

        self.cur_points.push(p);
        for (win, normal_slice) in normal_slices.iter().enumerate() {
            if (*normal_slice as i32) > 0 {
                let bucket_id = (win << self.bucket_bits) as u32 + normal_slice - 1;
                self._process_slices(bucket_id, self.cur_points.len() as u32 - 1);
            }
        }

        p.y_mut().neg_assign();

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
        p.y_mut().neg_assign();
        if is_neg_normal {
            p.y_mut().neg_assign();
        }

        // this isn't the cleanest of doing this, we'd better figure out a way to do this at compile time
        let p_g1: &mut TG1Affine = &mut p;
        endomorphism(p_g1);

        self.cur_points.push(p);
        for (win, phi_slice) in phi_slices.iter().enumerate() {
            if (*phi_slice as i32) > 0 {
                let bucket_id = (win << self.bucket_bits) as u32 + phi_slice - 1;
                self._process_slices(bucket_id, self.cur_points.len() as u32 - 1);
            }
        }

        p.y_mut().neg_assign();

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

    pub fn process_point_and_slices(&mut self, point: &TG1Affine, slices: &[u32]) {
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
        neg_p.y_mut().neg_assign();

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

    pub fn batch_reduce(&mut self) -> TG1 {
        let window_starts: Vec<_> = (0..self.num_windows as usize).collect();
        let num_groups =
            (self.num_windows as usize) << (self.bucket_bits as usize - GROUP_SIZE_IN_BITS);
        let mut running_sums: Vec<_> = vec![TG1Affine::ZERO; num_groups];
        let mut sum_of_sums: Vec<_> = vec![TG1Affine::ZERO; num_groups];

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

        let sum_by_window: Vec<TG1> = cfg_into_iter!(window_starts)
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

    fn inner_window_reduce(&self, running_sums: &[TG1Affine], sum_of_sums: &[TG1Affine]) -> TG1 {
        self.calc_sum_of_sum_total(sum_of_sums)
            .add_or_dbl(&self.calc_running_sum_total(running_sums))
    }

    fn calc_running_sum_total(&self, running_sums: &[TG1Affine]) -> TG1 {
        let mut running_sum_total = TG1::ZERO;
        for (i, running_sum) in running_sums.iter().enumerate().skip(1) {
            for _ in 0..i {
                TProjAddAffine::add_or_double_assign_affine(&mut running_sum_total, running_sum);
            }
        }

        for _ in 0..GROUP_SIZE_IN_BITS {
            running_sum_total.dbl_assign();
        }
        running_sum_total
    }

    fn calc_sum_of_sum_total(&self, sum_of_sums: &[TG1Affine]) -> TG1 {
        let mut sum = TG1::ZERO;
        sum_of_sums
            .iter()
            .for_each(|p| TProjAddAffine::add_or_double_assign_affine(&mut sum, p));
        sum
    }

    fn intra_window_reduce(&mut self, window_sums: &[TG1]) -> TG1 {
        // Traverse windows from high to low
        let lowest = window_sums.first().unwrap();
        lowest.add(
            &window_sums[1..]
                .iter()
                .rev()
                .fold(TG1::ZERO, |mut total, sum_i| {
                    total.add_assign(sum_i);
                    for _ in 0..self.window_bits {
                        total.dbl_assign();
                    }
                    total
                }),
        )
    }
}
