use crate::finite::BlstFr;
use kzg::{FFTSettings, Fr};
use std::slice;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct KzgFFTSettings4844 {
    pub max_width: u64,
    pub expanded_roots_of_unity: *mut BlstFr,
    pub reverse_roots_of_unity: *mut BlstFr,
    pub roots_of_unity: *mut BlstFr,
}

extern "C" {}

impl FFTSettings<BlstFr> for KzgFFTSettings4844 {
    fn default() -> Self {
        Self {
            max_width: 0,
            expanded_roots_of_unity: &mut Fr::default(),
            reverse_roots_of_unity: &mut Fr::default(),
            roots_of_unity: &mut Fr::default(),
        }
    }

    // underscore was added to avoid warnings when new is unused
    fn new(_scale: usize) -> Result<Self, String> {
        todo!();
    }

    fn get_max_width(&self) -> usize {
        self.max_width as usize
    }

    fn get_expanded_roots_of_unity_at(&self, i: usize) -> BlstFr {
        unsafe { *(self.expanded_roots_of_unity.add(i)) as BlstFr }
    }

    fn get_expanded_roots_of_unity(&self) -> &[BlstFr] {
        unsafe { slice::from_raw_parts(self.expanded_roots_of_unity, self.max_width as usize) }
    }

    fn get_reverse_roots_of_unity_at(&self, i: usize) -> BlstFr {
        unsafe { *self.reverse_roots_of_unity.add(i) as BlstFr }
    }

    fn get_reversed_roots_of_unity(&self) -> &[BlstFr] {
        unsafe { slice::from_raw_parts(self.reverse_roots_of_unity, self.max_width as usize) }
    }
}

impl Drop for KzgFFTSettings4844 {
    fn drop(&mut self) {
    }
}
