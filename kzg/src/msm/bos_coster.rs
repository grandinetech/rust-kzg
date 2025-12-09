use core::marker::PhantomData;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::ops::Sub;

use crate::{Fr, G1Affine, G1Fp, G1GetFp, G1Mul, G1ProjAddAffine, Scalar256, G1};

#[derive(Debug, Clone)]
pub struct BosCosterTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
where
    TFr: Fr,
    TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
{
    points: Vec<TG1>,
    numpoints: usize,

    batch_numpoints: usize,
    batch_points: Vec<Vec<TG1>>,

    g1_affine_marker: PhantomData<TG1Affine>,
    g1_fp_marker: PhantomData<TG1Fp>,
    fr_marker: PhantomData<TFr>,
    g1_affine_add_marker: PhantomData<TG1ProjAddAffine>,
}

impl Ord for Scalar256 {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        for i in (0..4).rev() {
            if self.data[i] < other.data[i] {
                return Ordering::Less;
            } else if self.data[i] > other.data[i] {
                return Ordering::Greater;
            }
        }
        Ordering::Equal
    }
}

impl Scalar256 {
    #[inline(always)]
    fn bitlen_256(&self) -> u32 {
        for i in (0..4).rev() {
            if self.data[i] != 0 {
                return (i as u32) * 64 + (64 - self.data[i].leading_zeros());
            }
        }
        0
    }

    #[inline(always)]
    fn shl_256(&self, n: u32) -> Scalar256 {
        if n == 0 {
            return *self;
        }
        let limb_shift = (n / 64) as usize;
        let bit_shift = n % 64;
        let mut res = [0u64; 4];

        for i in (0..4).rev() {
            if i < limb_shift {
                continue;
            }
            let mut val = self.data[i - limb_shift] << bit_shift;
            if bit_shift != 0 && i - limb_shift > 0 {
                val |= self.data[i - limb_shift - 1] >> (64 - bit_shift);
            }
            res[i] = val;
        }
        Scalar256::from_u64(res)
    }

    #[inline(always)]
    pub fn shr1_assign(&mut self) {
        let mut carry = 0u64;
        for limb in self.data.iter_mut().rev() {
            let new_carry = (*limb & 1) << 63;
            *limb = (*limb >> 1) | carry;
            carry = new_carry;
        }
    }
}

impl PartialOrd for Scalar256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Sub for Scalar256 {
    type Output = Scalar256;

    #[inline(always)]
    fn sub(self, rhs: Scalar256) -> Scalar256 {
        let mut result = Scalar256::ZERO;
        let mut borrow = 0u64;
        for i in 0..4 {
            let (rhs_with_borrow, overflow_add) = rhs.data[i].overflowing_add(borrow);
            let (res, overflow_sub) = self.data[i].overflowing_sub(rhs_with_borrow);
            result.data[i] = res;
            borrow = if overflow_add || overflow_sub { 1 } else { 0 };
        }
        result
    }
}

struct Pair<T> {
    scalar: Scalar256,
    point: T,
}

impl<T> PartialEq for Pair<T> {
    fn eq(&self, other: &Self) -> bool {
        self.scalar == other.scalar
    }
}

impl<T> Eq for Pair<T> {}

impl<T> Ord for Pair<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.scalar.cmp(&other.scalar)
    }
}

impl<T> PartialOrd for Pair<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<
        TFr: Fr,
        TG1Fp: G1Fp,
        TG1: G1 + G1Mul<TFr> + G1GetFp<TG1Fp>,
        TG1Affine: G1Affine<TG1, TG1Fp>,
        TG1ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
    > BosCosterTable<TFr, TG1, TG1Fp, TG1Affine, TG1ProjAddAffine>
{
    pub fn new(points: &[TG1], matrix: &[Vec<TG1>]) -> Result<Option<Self>, String> {
        if matrix.is_empty() {
            Ok(Some(Self {
                numpoints: points.len(),
                points: Vec::from(points),

                batch_numpoints: 0,
                batch_points: Vec::new(),

                fr_marker: PhantomData,
                g1_fp_marker: PhantomData,
                g1_affine_marker: PhantomData,
                g1_affine_add_marker: PhantomData,
            }))
        } else {
            Ok(Some(Self {
                numpoints: points.len(),
                points: Vec::from(points),

                batch_numpoints: matrix[0].len(),
                batch_points: Vec::from(matrix),

                fr_marker: PhantomData,
                g1_fp_marker: PhantomData,
                g1_affine_marker: PhantomData,
                g1_affine_add_marker: PhantomData,
            }))
        }
    }

    #[cfg(feature = "parallel")]
    pub fn multiply_parallel(&self, scalars: &[TFr]) -> TG1 {
        self.multiply_sequential(scalars)
    }

    pub fn multiply_sequential(&self, scalars: &[TFr]) -> TG1 {
        Self::multiply_sequential_raw(&self.points, scalars)
    }

    // https://github.com/btclib-org/btclib was used as reference for this implementation
    fn multiply_sequential_raw(bases: &[TG1], scalars: &[TFr]) -> TG1 {
        let valid_pairs: Vec<Pair<TG1>> = scalars
            .iter()
            .zip(bases.iter())
            .filter(|(s, p)| !s.is_zero() && !p.is_inf())
            .map(|(s, p)| Pair {
                scalar: TFr::to_scalar(&s),
                point: p.clone(),
            })
            .collect();

        let mut heap: BinaryHeap<Pair<TG1>> = BinaryHeap::from(valid_pairs);

        while heap.len() > 1 {
            let pair1: Pair<TG1> = heap.pop().unwrap();
            let pair2_scalar = {
                let mut pair2 = heap.peek_mut().unwrap();
                let (k, pair2_scalar_2powk) = find_k(&pair1.scalar, &pair2.scalar);
                let mut new_point = pair1.point.clone();
                for _ in 0..k {
                    new_point.dbl_assign();
                }
                pair2.point.add_assign(&new_point);
                pair2_scalar_2powk
            };

            let scalar = pair1.scalar.sub(pair2_scalar);
            if !scalar.is_zero() {
                heap.push(Pair {
                    scalar,
                    point: pair1.point,
                });
            }
        }

        if heap.is_empty() {
            return TG1::zero();
        }

        let pair = heap.pop().unwrap();
        return pair.point.mul(&TFr::from_u64_arr(&pair.scalar.data));
    }

    pub fn multiply_batch(&self, scalars: &[Vec<TFr>]) -> Vec<TG1> {
        assert!(scalars.len() == self.batch_points.len());

        #[cfg(not(feature = "parallel"))]
        {
            self.batch_points
                .iter()
                .zip(scalars)
                .map(|(points, scalars)| Self::multiply_sequential_raw(points, scalars))
                .collect::<Vec<_>>()
        }

        #[cfg(feature = "parallel")]
        {
            use super::{
                cell::Cell,
                thread_pool::{da_pool, ThreadPoolExt},
            };
            use core::sync::atomic::{AtomicUsize, Ordering};
            use std::sync::Arc;

            let pool = da_pool();
            let ncpus = pool.max_count();
            let counter = Arc::new(AtomicUsize::new(0));
            let mut results: Vec<Cell<TG1>> = Vec::with_capacity(scalars.len());
            #[allow(clippy::uninit_vec)]
            unsafe {
                results.set_len(results.capacity())
            };
            let results = &results[..];

            for _ in 0..ncpus {
                let counter = counter.clone();
                pool.joined_execute(move || loop {
                    let work = counter.fetch_add(1, Ordering::Relaxed);

                    if work >= scalars.len() {
                        break;
                    }

                    let result =
                        Self::multiply_sequential_raw(&self.batch_points[work], &scalars[work]);
                    unsafe { *results[work].as_ptr().as_mut().unwrap() = result };
                });
            }

            pool.join();

            results.iter().map(|it| it.as_mut().clone()).collect()
        }
    }
}

// x >= y
#[inline(always)]
fn find_k(x: &Scalar256, y: &Scalar256) -> (u32, Scalar256) {
    let bitlen_x = Scalar256::bitlen_256(&x);
    let bitlen_y = Scalar256::bitlen_256(&y);
    if bitlen_x == bitlen_y {
        return (0, *y);
    }
    let mut d = bitlen_x - bitlen_y;
    let mut shifted_y = y.shl_256(d);
    if shifted_y.cmp(&x) == std::cmp::Ordering::Greater {
        d -= 1;
        shifted_y.shr1_assign();
    }
    (d, shifted_y)
}
