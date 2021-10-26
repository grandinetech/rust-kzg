// #[cfg(test)]
// mod tests {
//     use kzg::{FFTSettings, Fr, G1};

//     #[test]
//     fn test_fft_settings_alloc() {
//         let mut settings = FFTSettings::new(16).unwrap();
//         assert_eq!(settings.max_width, 2 << 16 - 1);
//         settings.destroy();
//     }

//     #[test]
//     fn roundtrip_fft_fr() {
//         let size: u32 = 12;
//         let mut fs = FFTSettings::new(size).unwrap();
//         assert_eq!(fs.max_width, 2 << size - 1);
//         let mut data = vec![Fr::default(); fs.max_width];
//         for i in 0..fs.max_width {
//             data[i] = Fr::from_u64(i as u64);
//         }
//         let mut coeffs = fs.fft_fr(&mut data, false);
//         assert_eq!(coeffs.len(), 2 << size - 1);
//         data = fs.fft_fr(&mut coeffs, true);
//         assert_eq!(data.len(), 2 << size - 1);
//         // Verify that the result is still ascending values of i
//         for i in 0..fs.max_width {
//             let temp = Fr::from_u64(i as u64);
//             assert!(Fr::is_equal(temp, data[i]));
//         }
//         fs.destroy();
//     }

//     #[test]
//     fn roundtrip_fft_g1() {
//         let size: u32 = 10;
//         let mut fs = FFTSettings::new(size).unwrap();
//         assert_eq!(fs.max_width, 2 << size - 1);
//         // make_data
//         let expected = FFTSettings::make_data(fs.max_width);
//         let mut data = FFTSettings::make_data(fs.max_width);
//         // Forward and reverse FFT
//         let mut coeffs = fs.fft_g1(&mut data, false);
//         assert_eq!(coeffs.len(), 2 << size - 1);
//         data = fs.fft_g1(&mut coeffs, true);
//         assert_eq!(data.len(), 2 << size - 1);
//         // Verify that the result is still ascending values of i
//         for i in 0..fs.max_width {
//             assert!(G1::is_equal(&expected[i], &data[i]));
//         }
//         fs.destroy();
//     }
// }
