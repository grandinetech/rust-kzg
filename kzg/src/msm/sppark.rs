pub struct SpparkPrecomputation(*mut char); // table holds a reference to value, initialized in C land. It should be never dereferenced

impl SpparkPrecomputation {
    pub fn new() -> Self {}
}

pub trait SpparkBackend {
    fn precompute() -> *mut char;
}
