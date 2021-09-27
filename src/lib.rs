#[repr(C)]
pub enum KzgRet {
    KzgOk = 0,
    KzgBadArgs,
    KzgError,
    KzgMalloc
}

#[link(name = "ckzg", kind = "static")]
extern "C" {
    fn hello() -> KzgRet;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
