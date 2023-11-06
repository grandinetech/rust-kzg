use std::sync::mpsc::channel;

use crate::{
    cfg_into_iter, common_utils::log2_u64, G1Affine, G1Fp, G1ProjAddAffine, Scalar256, G1, msm::batch_adder::{self, BatchAdder},
};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

trait ThreadPoolExt {
    fn joined_execute<'any, F>(&self, job: F)
    where
        F: FnOnce() + Send + 'any;
}

mod mt {
    use super::*;
    use core::mem::transmute;
    use std::sync::{Mutex, Once};
    use threadpool::ThreadPool;

    pub fn da_pool() -> ThreadPool {
        static INIT: Once = Once::new();
        static mut POOL: *const Mutex<ThreadPool> =
            0 as *const Mutex<ThreadPool>;

        INIT.call_once(|| {
            let pool = Mutex::new(ThreadPool::default());
            unsafe { POOL = transmute(Box::new(pool)) };
        });
        unsafe { (*POOL).lock().unwrap().clone() }
    }

    type Thunk<'any> = Box<dyn FnOnce() + Send + 'any>;

    impl ThreadPoolExt for ThreadPool {
        fn joined_execute<'scope, F>(&self, job: F)
        where
            F: FnOnce() + Send + 'scope,
        {
            // Bypass 'lifetime limitations by brute force. It works,
            // because we explicitly join the threads...
            self.execute(unsafe {
                transmute::<Thunk<'scope>, Thunk<'static>>(Box::new(job))
            })
        }
    }
}

macro_rules! cfg_into_mut_chunks {
    ($e: expr, $f: expr) => {{
        #[cfg(feature = "parallel")]
        let result = $e.par_chunks_mut($f);

        #[cfg(not(feature = "parallel"))]
        let result = $e.chunks_mut($f);

        result
    }};
}

pub fn parallel_pippinger<
    TG1: G1,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
>(
    bases: &[TG1Affine],
    scalars: &[Scalar256],
) -> TG1 {
    const NUM_BITS: u32 = 255;

    // Limit scalars & bases to lower of the two
    let size = std::cmp::min(bases.len(), scalars.len());
    let scalars = &scalars[..size];
    let bases = &bases[..size];

    let scalars_and_bases_iter = scalars
        .iter()
        .zip(bases)
        .filter(|(s, _)| **s != Scalar256::ZERO);
    let c = if size < 32 {
        3
    } else {
        ((log2_u64(size) * 69 / 100) as usize) + 2
    };

    // Divide 0..NUM_BITS into windows of size c & process in parallel
    let mut window_sums = [TG1::ZERO; NUM_BITS as usize];
    cfg_into_iter!(0..NUM_BITS)
        .step_by(c)
        .zip(&mut window_sums)
        .for_each(|(w_start, window_sums)| {
            // We don't need the "zero" bucket, so we only have 2^c - 1 buckets.
            let mut buckets = vec![TG1::ZERO; (1 << c) - 1];
            scalars_and_bases_iter.clone().for_each(|(scalar, base)| {
                if *scalar == Scalar256::ONE {
                    if w_start == 0 {
                        ProjAddAffine::add_assign_affine(window_sums, base);
                    }
                } else {
                    let mut scalar = scalar.data;
                    scalar_divn(&mut scalar, w_start);
                    let scalar = scalar[0] % (1 << c);
                    if scalar != 0 {
                        let idx = (scalar - 1) as usize;
                        ProjAddAffine::add_or_double_assign_affine(&mut buckets[idx], base);
                    }
                }
            });

            let mut running_sum = TG1::ZERO;
            buckets.into_iter().rev().for_each(|b| {
                running_sum.add_or_dbl_assign(&b);
                window_sums.add_or_dbl_assign(&running_sum);
            });
        });

    // Traverse windows from high to low
    let lowest = window_sums.first().unwrap();
    lowest.add(
        &window_sums[1..]
            .iter()
            .rev()
            .fold(TG1::ZERO, |mut total, sum_i| {
                total.add_assign(sum_i);
                for _ in 0..c {
                    total.dbl_assign();
                }
                total
            }),
    )
}

fn scalar_divn<const N: usize>(input: &mut [u64; N], mut n: u32) {
    if n >= (64 * N) as u32 {
        *input = [0u64; N];
        return;
    }

    while n >= 64 {
        let mut t = 0;
        for i in 0..N {
            core::mem::swap(&mut t, &mut input[N - i - 1]);
        }
        n -= 64;
    }

    if n > 0 {
        let mut t = 0;
        #[allow(unused)]
        for i in 0..N {
            let a = &mut input[N - i - 1];
            let t2 = *a << (64 - n);
            *a >>= n;
            *a |= t;
            t = t2;
        }
    }
}

// Compute msm using windowed non-adjacent form
pub fn parallel_pippinger_wnaf<
    TG1: G1,
    TG1Fp: G1Fp,
    TG1Affine: G1Affine<TG1, TG1Fp>,
    ProjAddAffine: G1ProjAddAffine<TG1, TG1Fp, TG1Affine>,
>(
    bases: &[TG1Affine],
    scalars: &[Scalar256],
) -> TG1 {
    const NUM_BITS: usize = 255;

    // Limit scalars & bases to lower of the two
    let size = std::cmp::min(bases.len(), scalars.len());
    let scalars = &scalars[..size];
    let bases = &bases[..size];

    let c = if size < 32 {
        3
    } else {
        ((log2_u64(size) * 69 / 100) as usize) + 2
    };

    let digits_count = (NUM_BITS + c - 1) / c;
    let mut scalar_digits = vec![0i64; digits_count * scalars.len()];
    cfg_into_mut_chunks!(scalar_digits, digits_count)
        .zip(scalars)
        // .filter(|(_, s)| **s != Scalar256::ZERO)
        .for_each(|(chunk , scalar)| {
            make_digits_into(scalar, c, chunk);
        });
    
    let pool = mt::da_pool();
    let ncpus = pool.max_count();
    let scalar_digits_it = scalar_digits.chunks(digits_count).zip(bases);
    
    let (tx, rx) = channel();
    for i in (0..digits_count).rev() {
        let tx = tx.clone();
        let i = i.clone();
        let scalar_digits_it = scalar_digits_it.clone();
        pool.joined_execute(move || {
            let mut buckets = vec![TG1::ZERO; 1 << c];
            for ( digits, base) in scalar_digits_it {
                use core::cmp::Ordering;
                let scalar = digits[i];
                match 0.cmp(&scalar) {
                    Ordering::Less => ProjAddAffine::add_assign_affine(&mut buckets[(scalar - 1) as usize], base),
                    Ordering::Greater => ProjAddAffine::sub_assign_affine(&mut buckets[(-scalar - 1) as usize], *base),
                    Ordering::Equal => (),
                }
            }

            let mut running_sum = TG1::ZERO;
            let mut window_sum = TG1::ZERO;
            buckets.into_iter().rev().for_each(|b| {
                running_sum.add_or_dbl_assign(&b);
                window_sum.add_or_dbl_assign(&running_sum);
            });
            tx.send(window_sum);
        });
    }

    let mut total_window_sum = TG1::ZERO;
    for _ in 0..digits_count {
        total_window_sum.add_assign(&rx.recv().unwrap());
        for _ in 0..c {
            total_window_sum.dbl_assign();
        }
    }
    total_window_sum

    // let lowest = window_sums.first().unwrap();
    // lowest.add(
    //     &window_sums[1..]
    //         .iter()
    //         .rev()
    //         .fold(TG1::ZERO, |mut total, sum_i| {
    //             total.add_assign(sum_i);
    //             for _ in 0..c {
    //                 total.dbl_assign();
    //             }
    //             total
    //         }))
}

// From: https://github.com/arkworks-rs/gemini/blob/main/src/kzg/msm/variable_base.rs#L20
fn make_digits(a: &Scalar256, w: usize) -> Vec<i64> {
    let scalar = &a.data;
    let radix: u64 = 1 << w;
    let window_mask: u64 = radix - 1;

    const NUM_BITS: usize = 255;
    let mut carry = 0u64;
    let digits_count = (NUM_BITS + w - 1) / w;
    let mut digits = vec![0i64; digits_count];
    for (i, digit) in digits.iter_mut().enumerate() {
        // Construct a buffer of bits of the scalar, starting at `bit_offset`.
        let bit_offset = i * w;
        let u64_idx = bit_offset / 64;
        let bit_idx = bit_offset % 64;
        // Read the bits from the scalar
        let bit_buf = if bit_idx < 64 - w || u64_idx == scalar.len() - 1 {
            // This window's bits are contained in a single u64,
            // or it's the last u64 anyway.
            scalar[u64_idx] >> bit_idx
        } else {
            // Combine the current u64's bits with the bits from the next u64
            (scalar[u64_idx] >> bit_idx) | (scalar[1 + u64_idx] << (64 - bit_idx))
        };

        // Read the actual coefficient value from the window
        let coef = carry + (bit_buf & window_mask); // coef = [0, 2^r)

        // Recenter coefficients from [0,2^w) to [-2^w/2, 2^w/2)
        carry = (coef + radix / 2) >> w;
        *digit = (coef as i64) - (carry << w) as i64;
    }

    digits[digits_count - 1] += (carry << w) as i64;

    digits
}

// From: https://github.com/arkworks-rs/gemini/blob/main/src/kzg/msm/variable_base.rs#L20
fn make_digits_into(a: &Scalar256, w: usize, digits: &mut [i64]) {
    let scalar = &a.data;
    let radix: u64 = 1 << w;
    let window_mask: u64 = radix - 1;

    const NUM_BITS: usize = 255;
    let mut carry = 0u64;
    let digits_count = (NUM_BITS + w - 1) / w;
    for (i, digit) in digits.iter_mut().enumerate() {
        // Construct a buffer of bits of the scalar, starting at `bit_offset`.
        let bit_offset = i * w;
        let u64_idx = bit_offset / 64;
        let bit_idx = bit_offset % 64;
        // Read the bits from the scalar
        let bit_buf = if bit_idx < 64 - w || u64_idx == scalar.len() - 1 {
            // This window's bits are contained in a single u64,
            // or it's the last u64 anyway.
            scalar[u64_idx] >> bit_idx
        } else {
            // Combine the current u64's bits with the bits from the next u64
            (scalar[u64_idx] >> bit_idx) | (scalar[1 + u64_idx] << (64 - bit_idx))
        };

        // Read the actual coefficient value from the window
        let coef = carry + (bit_buf & window_mask); // coef = [0, 2^r)

        // Recenter coefficients from [0,2^w) to [-2^w/2, 2^w/2)
        carry = (coef + radix / 2) >> w;
        *digit = (coef as i64) - (carry << w) as i64;
    }

    digits[digits_count - 1] += (carry << w) as i64;
}