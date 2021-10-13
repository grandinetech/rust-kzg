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