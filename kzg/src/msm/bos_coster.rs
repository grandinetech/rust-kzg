use core::marker::PhantomData;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

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
    points: Vec<TG1Affine>,
    numpoints: usize,

    batch_numpoints: usize,
    batch_points: Vec<Vec<TG1Affine>>,

    g1_marker: PhantomData<TG1>,
    g1_fp_marker: PhantomData<TG1Fp>,
    fr_marker: PhantomData<TFr>,
    g1_affine_add_marker: PhantomData<TG1ProjAddAffine>,
}

impl Ord for Scalar256 {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare as big-endian integer: highest limb first
        for (a, b) in self.data.iter().zip(other.data.iter()).rev() {
            match a.cmp(b) {
                Ordering::Equal => continue,
                non_eq => return non_eq,
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for Scalar256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
        Ok(Some(Self {
            numpoints: points.len(),
            points: Vec::from(points)
                .iter()
                .map(|g1| TG1Affine::into_affine(g1))
                .collect(),

            // TODO:
            batch_numpoints: 0,
            batch_points: Vec::new(),

            fr_marker: PhantomData,
            g1_fp_marker: PhantomData,
            g1_marker: PhantomData,
            g1_affine_add_marker: PhantomData,
        }))
    }

    #[cfg(feature = "parallel")]
    pub fn multiply_parallel(&self, scalars: &[TFr]) -> TG1 {
        self.multiply_sequential(scalars)
    }

    pub fn multiply_sequential(&self, scalars: &[TFr]) -> TG1 {
        Self::multiply_sequential_raw(&self.points, scalars)
    }

    fn multiply_sequential_raw(bases: &[TG1Affine], scalars: &[TFr]) -> TG1 {
        let scalars: Vec<Scalar256> = scalars.iter().map(TFr::to_scalar).collect::<Vec<_>>();

        let value: Vec<Pair<TG1>> = scalars
            .iter()
            .zip(bases.iter().map(|p| p.to_proj()))
            .map(|(&s, p)| Pair {
                scalar: s,
                point: p,
            })
            .collect();

        let mut heap: BinaryHeap<Pair<TG1>> = BinaryHeap::from(value);

        while heap.len() > 1 {
            let pair1: Pair<TG1> = heap.pop().unwrap();
            let pair2: Pair<TG1> = heap.pop().unwrap();

            let s1: TFr = TFr::from_u64_arr(&pair1.scalar.data);
            let s2: TFr = TFr::from_u64_arr(&pair2.scalar.data);

            let tfr1: TFr = TFr::add(&s1, &s2.negate());
            if !tfr1.is_zero() {
                heap.push(Pair {
                    scalar: tfr1.to_scalar(),
                    point: TG1Affine::into_affine(&pair1.point).to_proj(),
                });
            }

            heap.push(Pair {
                scalar: s2.to_scalar(),
                point: pair2.point.add(&pair1.point),
            });
        }

        let pair = heap.pop().unwrap();
        return pair.point.mul(&TFr::from_u64_arr(&pair.scalar.data));
    }

    // TODO:
    pub fn multiply_batch(&self, scalars: &[Vec<TFr>]) -> Vec<TG1> {
        assert!(scalars.len() == self.batch_points.len());

        self.batch_points
            .iter()
            .zip(scalars)
            .map(|(points, scalars)| Self::multiply_sequential_raw(points, scalars))
            .collect::<Vec<_>>()
    }
}
