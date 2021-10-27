use crate::data_types::g1::G1;
use kzg::{G1 as CommonG1};

impl CommonG1 for G1 {
    fn default() -> Self {
        G1::zero()
    }
    
    fn rand() -> Self {
        G1::random()
    }

    fn add_or_double(&mut self, b: &Self) -> Self {
        let mut g1 = G1::zero();
        G1::add(&mut g1, &self, &b);
        g1
    }

    fn equals(&self, b: &Self) -> bool {
        G1::eq(self, b)
    }

    fn rand() -> Self {
        todo!()
    }
    fn destroy(&mut self) {}
}
