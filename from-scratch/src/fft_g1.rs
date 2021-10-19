// use kzg::{G1, Fr};

// pub fn fft_g1_fast(output: &G1, input: *G1, stride: u64, roots: *Fr, roots_stride: u64,
//         n: u64) {
//     let half = n / 2;
//     if half > 0 {
//         // fft_g1_fast(output, input, stride * 2, roots, roots_stride * 2, half);
//         // fft_g1_fast(output + half, input + stride, stride * 2, roots, roots_stride * 2, half);
//         // for i in 0..half {
//         //    let y_times_root: G1;
//         //
//         // }
//     }
// }

// fn fast_log_2_byte(mut b: byte) -> i32 {
//     let shift: i32;
//     let mut r: i32;
//
//     r = ((b > 0xF) as i32) << 2;
//     b >>= r;
//     shift = ((b > 0x3) as i32) << 1;
//     b >>= shift + 1;
//     r |= shift | b as i32;
//     return r;
// }
//
// pub unsafe fn g1_mul(output: &mut G1, group_element: &G1, multiplier: &Fr) {
//     let mut scalar: Scalar = Default::default();
//     blst_scalar_from_fr(&mut scalar, multiplier);
//
//     let mut i = size_of::<Scalar>();
//     while i != 0 && scalar.b[i - 1] == 0 {
//         println!("{}", scalar.b[i-1]);
//         i -= 1;
//     }
//     if i == 0 {
//         *output = G1_IDENTITY;
//     } else if i == 1 && scalar.b[0] == 1 {
//         *output = *group_element;
//     } else {
//         blst_p1_mult(output, group_element, scalar.b.as_ptr(), (8 * (i as i32) - 7 + fast_log_2_byte(scalar.b[i - 1])) as usize)
//     }
// }