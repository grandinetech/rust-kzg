// Minimalist core::cell::Cell stand-in, but with Sync marker, which
// makes it possible to pass it to multiple threads. It works, because
// *here* each Cell is written only once and by just one thread.
#[repr(transparent)]
pub struct Cell<T: ?Sized> {
    value: T,
}
unsafe impl<T: ?Sized + Sync> Sync for Cell<T> {}
impl<T> Cell<T> {
    pub fn as_ptr(&self) -> *mut T {
        &self.value as *const T as *mut T
    }

    // Helper to avoid as_ptr().as_ref().unwrap()
    #[allow(clippy::mut_from_ref)]
    pub fn as_mut(&self) -> &mut T {
        unsafe { &mut *(self.as_ptr()) }
    }
}
