macro_rules! common_impl {
    ($t:ty, $is_equal_fn:ident, $is_zero_fn:ident) => {
        impl PartialEq for $t {
            fn eq(&self, rhs: &Self) -> bool {
                unsafe { $is_equal_fn(self, rhs) == 1 }
            }
        }
        impl $t {
            pub fn zero() -> $t {
                Default::default()
            }
            pub fn uninit() -> std::mem::MaybeUninit::<$t> {
                std::mem::MaybeUninit::uninit()
            }
            pub fn clear(&mut self) {
                *self = <$t>::zero()
            }
            pub fn is_zero(&self) -> bool {
                unsafe { $is_zero_fn(self) == 1 }
            }
        }
    };
}
macro_rules! is_valid_impl {
    ($t:ty, $is_valid_fn:ident) => {
        impl $t {
            /// # Safety
            ///
            /// MCL Function, unsure why it is unsafe
            pub fn is_valid(&self) -> bool {
                unsafe { $is_valid_fn(self) == 1 }
            }
        }
    };
}

macro_rules! serialize_impl {
    ($t:ty, $size:expr, $serialize_fn:ident, $deserialize_fn:ident) => {
        impl $t {
            /// # Safety
            ///
            /// MCL Function, unsure why it is unsafe
            pub fn deserialize(&mut self, buf: &[u8]) -> bool {
                unsafe { $deserialize_fn(self, buf.as_ptr(), buf.len()) > 0 }
            }
            /// # Safety
            ///
            /// MCL Function, unsure why it is unsafe
            pub fn serialize(&self) -> Vec<u8> {
                let size = unsafe { $size } as usize;
                let mut buf: Vec<u8> = Vec::with_capacity(size);
                let n: usize;
                unsafe {
                    n = $serialize_fn(buf.as_mut_ptr(), size, self);
                }
                if n == 0 {
                    panic!("serialize");
                }
                unsafe {
                    buf.set_len(n);
                }
                buf
            }
        }
    };
}

macro_rules! str_impl {
    ($t:ty, $maxBufSize:expr, $get_str_fn:ident, $set_str_fn:ident) => {
        impl $t {
            pub fn from_str(s: &str, base: i32) -> Option<$t> {
                let mut v = <$t>::zero();
                if v.set_str(s, base) {
                    return Some(v);
                }
                None
            }
            pub fn set_str(&mut self, s: &str, base: i32) -> bool {
                unsafe { $set_str_fn(self, s.as_ptr(), s.len(), base) == 0 }
            }
            pub fn get_str(&self, io_mode: i32) -> String {
                let mut buf: [u8; $maxBufSize] = [0; $maxBufSize];
                let n: usize;
                unsafe {
                    n = $get_str_fn(buf.as_mut_ptr(), buf.len(), self, io_mode);
                }
                if n == 0 {
                    panic!("mclBnFr_getStr");
                }
                unsafe { std::str::from_utf8_unchecked(&buf[0..n]).into() }
            }
        }
    };
}

macro_rules! int_impl {
    ($t:ty, $set_int_fn:ident, $is_one_fn:ident) => {
        impl $t {
            pub fn from_int(x: i32) -> $t {
                let mut v = <$t>::zero();
                v.set_int(x);
                v
            }
            pub fn set_int(&mut self, x: i32) {
                unsafe {
                    $set_int_fn(self, x);
                }
            }
            pub fn is_one(&self) -> bool {
                unsafe { $is_one_fn(self) == 1 }
            }
        }
    };
}

macro_rules! base_field_impl {
    ($t:ty,  $set_little_endian_fn:ident, $set_little_endian_mod_fn:ident, $set_hash_of_fn:ident, $set_by_csprng_fn:ident, $is_odd_fn:ident, $is_negative_fn:ident, $square_root_fn:ident) => {
        impl $t {
            pub fn set_little_endian(&mut self, buf: &[u8]) -> bool {
                unsafe { $set_little_endian_fn(self, buf.as_ptr(), buf.len()) == 0 }
            }
            pub fn set_little_endian_mod(&mut self, buf: &[u8]) -> bool {
                unsafe { $set_little_endian_mod_fn(self, buf.as_ptr(), buf.len()) == 0 }
            }
            pub fn set_hash_of(&mut self, buf: &[u8]) -> bool {
                unsafe { $set_hash_of_fn(self, buf.as_ptr(), buf.len()) == 0 }
            }
            pub fn set_by_csprng(&mut self) {
                unsafe { $set_by_csprng_fn(self) }
            }
            pub fn is_odd(&self) -> bool {
                unsafe { $is_odd_fn(self) == 1 }
            }
            pub fn is_negative(&self) -> bool {
                unsafe { $is_negative_fn(self) == 1 }
            }
            pub fn square_root(y: &mut $t, x: &$t) -> bool {
                unsafe { $square_root_fn(y, x) == 0 }
            }
        }
    };
}

macro_rules! add_op_impl {
    ($t:ty, $add_fn:ident, $sub_fn:ident, $neg_fn:ident) => {
        impl $t {
            /// # Safety
            ///
            /// MCL Function, unsure why it is unsafe
            pub unsafe fn add(z: *mut $t, x: &$t, y: &$t) {
                $add_fn(z, x, y)
            }
            /// # Safety
            ///
            /// MCL Function, unsure why it is unsafe
            pub unsafe fn sub(z: *mut $t, x: &$t, y: &$t) {
                $sub_fn(z, x, y)
            }
            pub fn neg(y: &mut $t, x: &$t) {
                unsafe { $neg_fn(y, x) }
            }
        }
        impl<'a> Add for &'a $t {
            type Output = $t;
            fn add(self, other: &$t) -> $t {
                let mut raw_v = <$t>::uninit();
                unsafe{
                    <$t>::add(raw_v.as_mut_ptr().cast::<$t>(), &self, &other);
                    raw_v.assume_init()
                }
            }
        }
        impl<'a> AddAssign<&'a $t> for $t {
            fn add_assign(&mut self, other: &$t) {
                // how can I write this?
                // unsafe { <$t>::add(&mut self, &self, &other); }
                let mut raw_v = <$t>::uninit();
                unsafe{
                    <$t>::add(raw_v.as_mut_ptr().cast::<$t>(), &self, &other);
                    *self = raw_v.assume_init();
                }
                
            }
        }
        impl<'a> Sub for &'a $t {
            type Output = $t;
            fn sub(self, other: &$t) -> $t {
                let mut raw_v = <$t>::uninit();
                unsafe{
                    <$t>::sub(raw_v.as_mut_ptr().cast::<$t>(), &self, &other);
                    raw_v.assume_init()
                }
            }
        }
        impl<'a> SubAssign<&'a $t> for $t {
            fn sub_assign(&mut self, other: &$t) {
                let mut raw_v = <$t>::uninit();
                unsafe{
                    <$t>::sub(raw_v.as_mut_ptr().cast::<$t>(), &self, &other);
                    *self = raw_v.assume_init();
                }
            }
        }
    };
}

macro_rules! field_mul_op_impl {
    ($t:ty, $mul_fn:ident, $div_fn:ident, $inv_fn:ident, $sqr_fn:ident) => {
        impl $t {
            /// # Safety
            ///
            /// MCL Function, unsure why it is unsafe
            pub unsafe fn mul(z: *mut $t, x: &$t, y: &$t) {
                $mul_fn(z, x, y)
            }
            /// # Safety
            ///
            /// MCL Function, unsure why it is unsafe
            pub unsafe fn div(z: *mut $t, x: &$t, y: &$t) {
                $div_fn(z, x, y)
            }
            pub fn inv(y: &mut $t, x: &$t) {
                unsafe { $inv_fn(y, x) }
            }
            pub fn sqr(y: &mut $t, x: &$t) {
                unsafe { $sqr_fn(y, x) }
            }
        }
        impl<'a> Mul for &'a $t {
            type Output = $t;
            fn mul(self, other: &$t) -> $t {
                let mut raw_v = <$t>::uninit();
                unsafe{
                    <$t>::mul(raw_v.as_mut_ptr().cast::<$t>(), &self, &other);
                    raw_v.assume_init()
                }
            }
        }
        impl<'a> MulAssign<&'a $t> for $t {
            fn mul_assign(&mut self, other: &$t) {
                let mut raw_v = <$t>::uninit();
                unsafe {
                    <$t>::mul(raw_v.as_mut_ptr().cast::<$t>(), &self, &other);
                    *self = raw_v.assume_init();
                }
            }
        }
        impl<'a> Div for &'a $t {
            type Output = $t;
            fn div(self, other: &$t) -> $t {
                let mut raw_v = <$t>::uninit();
                unsafe { <$t>::div(raw_v.as_mut_ptr().cast::<$t>(), &self, &other); }
                unsafe { raw_v.assume_init() }
            }
        }
        impl<'a> DivAssign<&'a $t> for $t {
            fn div_assign(&mut self, other: &$t) {
                let mut raw_v = <$t>::uninit();
                unsafe { <$t>::div(raw_v.as_mut_ptr().cast::<$t>(), &self, &other); }
                *self = unsafe { raw_v.assume_init() };
            }
        }
    };
}

macro_rules! ec_impl {
    ($t:ty, $dbl_fn:ident, $mul_fn:ident, $normalize_fn:ident, $set_hash_and_map_fn:ident) => {
        impl $t {
            pub fn dbl(y: &mut $t, x: &$t) {
                unsafe { $dbl_fn(y, x) }
            }
            pub fn mul(z: &mut $t, x: &$t, y: &Fr) {
                unsafe { $mul_fn(z, x, y) }
            }
            pub fn normalize(y: &mut $t, x: &$t) {
                unsafe { $normalize_fn(y, x) }
            }
            pub fn set_hash_of(&mut self, buf: &[u8]) -> bool {
                unsafe { $set_hash_and_map_fn(self, buf.as_ptr(), buf.len()) == 0 }
            }
        }
    };
}


macro_rules! get_str_impl {
    ($get_str_fn:ident) => {{
        let mut buf: [u8; 256] = [0; 256];
        let n: usize;
        unsafe {
            n = $get_str_fn(buf.as_mut_ptr(), buf.len());
        }
        if n == 0 {
            panic!("get_str");
        }
        unsafe { std::str::from_utf8_unchecked(&buf[0..n]).into() }
    }};
}