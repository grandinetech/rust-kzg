#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum KzgRet {
    KzgOk = 0,
    KzgBadArgs = 1,
    KzgError = 2,
    KzgMalloc = 3
}
