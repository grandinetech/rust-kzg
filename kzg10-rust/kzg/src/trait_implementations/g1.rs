use crate::data_types::g1::G1;
use kzg::{G1 as CommonG1};

impl CommonG1 for G1 {
    fn default() -> Self {
        G1::zero()
    }

    fn identity() -> Self {
        todo!()
    }

    fn generator() -> Self {
        todo!()
    }

    fn negative_generator() -> Self {
        todo!()
    }

    fn rand() -> Self {
        G1::random()
    }

    fn add_or_dbl(&self, b: &Self) -> Self {
        let mut g1 = G1::zero();
        G1::add(&mut g1, &self, &b);
        g1
    }

    fn is_inf(&self) -> bool {
        todo!()
    }

    fn dbl(&self) -> Self {
        todo!()
    }

    fn sub(&self, b: &Self) -> Self {
        todo!()
    }

    fn equals(&self, b: &Self) -> bool {
        G1::eq(self, b)
    }

    fn destroy(&mut self) {}
}
