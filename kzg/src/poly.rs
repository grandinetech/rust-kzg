use super::Poly;
use super::Error;

#[link(name = "ckzg", kind = "static")]
extern "C" {
    pub fn new_poly(out: *mut Poly, length: u64) -> Error;
    pub fn free_poly(p: *mut Poly);
}

pub fn create_poly(length: u64) -> Result<Poly, Error> {
    let mut poly = Poly::default();
    unsafe {
        let error = new_poly(&mut poly, length);
        return match error {
            Error::KzgOk => Ok(poly),
            _ => Err(error)
        }
    }
}

pub fn destroy_poly(poly: &mut Poly) {
    unsafe {
        free_poly(poly);
    }
}
