// Implementation of atomic bitmap
// Modified from non-sync implementation
use core::sync::atomic::{AtomicU32, Ordering};

use alloc::vec::Vec;

pub struct Bitmap {
    size: usize,
    data: Vec<AtomicU32>,
}

impl Bitmap {
    pub fn new(size: usize) -> Bitmap {
        let mut data: Vec<AtomicU32> = Vec::with_capacity(size);
        data.resize_with(size, Default::default);

        Bitmap { size, data }
    }

    pub fn test_and_set(&self, bucket: u32) -> bool {
        let word = (bucket >> 5) as usize;
        let bit = 1 << (bucket & 0x1F);

        let mut old = self.data[word].load(Ordering::Relaxed);
        loop {
            // If bit is 'already' set then return true
            if old & bit != 0 {
                return true;
            }

            let new = old | bit;
            match self.data[word].compare_exchange_weak(
                old,
                new,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                // We managed to take bit, return false
                Ok(_) => return false,
                // If write failed either another bit at index was set, or wanted index was set
                Err(x) => old = x,
            };
        }
    }

    pub fn clear(&self) {
        for i in 0..self.size {
            self.data[i].store(0, Ordering::Relaxed);
        }
    }
}
