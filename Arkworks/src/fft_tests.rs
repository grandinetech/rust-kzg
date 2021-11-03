// use crate::fft::fft_fr;
// // use crate::fft::fr_are_equal;
// use crate::kzg_proofs::{
//     check_proof_multi, check_proof_single, commit_to_poly, compute_proof_multi,
//     compute_proof_single, eval_poly, fr_add, fr_from_uint64, fr_mul,
//     new_fft_settings, new_poly, FFTSettings, TooLongPoly,
// };
// use crate::fft_g1::G1_IDENTITY;
// use crate::kzg_types::FsFr as Fr;
// use crate::{Eq, /* Fr,*/ P1};
// use blst::blst_fr_from_uint64;
// use kzg::Fr as FrTrait;

// #[test]
// pub(crate) fn roundtrip_fft() {
//     let size: usize = 12;

//     let fft_settings = FFTSettings::from_scale(size).unwrap();

//     let mut starting_data = vec![Fr::default(); fft_settings.max_width as usize];
//     for i in 0..fft_settings.max_width {
//         starting_data[i as usize] = Fr::from_u64_arr(&[i as u64, 0, 0, 0]);
//     }

//     // Forward and inverse FFT
//     let forward_result = fft_fr(&starting_data, false, &fft_settings).unwrap();
//     let inverse_result = fft_fr(&forward_result, true, &fft_settings).unwrap();

//     for i in 0..fft_settings.max_width {
//         assert!(&starting_data[i as usize].equals(&inverse_result[i as usize]));
//     }
// }

// #[test]
// pub(crate) fn stride_fft() {
//     let size1: usize = 9;
//     let size2: usize = 12;

//     let width: usize = 1 << size1;

//     let fft_settings1 = FFTSettings::from_scale(size1).unwrap();
//     let fft_settings2 = FFTSettings::from_scale(size2).unwrap();

//     let mut data = vec![Fr::default(); width];
//     for i in 0..width {
//         data[i as usize] = Fr::from_u64_arr(&[i as u64, 0, 0, 0]);
//     }

//     let result1 = fft_fr(&data, false, &fft_settings1).unwrap();
//     let result2 = fft_fr(&data, false, &fft_settings2).unwrap();

//     for i in 0..width {
//         assert!(result1[i].equals(&result2[i]));
//     }
// }

// #[test]
// pub(crate) fn inverse_fft() {
//     let inv_fft_expected: [[u64; 4]; 16] = [
//         [
//             0x7fffffff80000008,
//             0xa9ded2017fff2dff,
//             0x199cec0404d0ec02,
//             0x39f6d3a994cebea4,
//         ],
//         [
//             0xef296e7ffb8ca216,
//             0xd5b902cbcef9c1b6,
//             0xf06dfe5c7fca260d,
//             0x13993b7d05187205,
//         ],
//         [
//             0xe930fdda2306c7d4,
//             0x40e02aff48e2b16b,
//             0x83a712d1dd818c8f,
//             0x5dbc603bc53c7a3a,
//         ],
//         [
//             0xf9925986d0d25e90,
//             0xcdf85d0a339d7782,
//             0xee7a9a5f0410e423,
//             0x2e0d216170831056,
//         ],
//         [
//             0x80007fff80000000,
//             0x1fe05202bb00adff,
//             0x6045d26b3fd26e6b,
//             0x39f6d3a994cebea4,
//         ],
//         [
//             0x27325dd08ac4cee9,
//             0xcbb94f168ddacca9,
//             0x6843be68485784b1,
//             0x5a6faf9039451673,
//         ],
//         [
//             0xe92ffdda2306c7d4,
//             0x54dd2afcd2dfb16b,
//             0xf6554603677e87be,
//             0x5dbc603bc53c7a39,
//         ],
//         [
//             0x1cc772c9b57f126f,
//             0xfb73f4d33d3116dd,
//             0x4f9388c8d80abcf9,
//             0x3ffbc9abcdda7821,
//         ],
//         [
//             0x7fffffff80000000,
//             0xa9ded2017fff2dff,
//             0x199cec0404d0ec02,
//             0x39f6d3a994cebea4,
//         ],
//         [
//             0xe3388d354a80ed91,
//             0x5849af2fc2cd4521,
//             0xe3a64f3f31971b0b,
//             0x33f1dda75bc30526,
//         ],
//         [
//             0x16d00224dcf9382c,
//             0xfee079062d1eaa93,
//             0x3ce49204a2235046,
//             0x163147176461030e,
//         ],
//         [
//             0xd8cda22e753b3117,
//             0x880454ec72238f55,
//             0xcaf6199fc14a5353,
//             0x197df7c2f05866d4,
//         ],
//         [
//             0x7fff7fff80000000,
//             0x33dd520044fdadff,
//             0xd2f4059cc9cf699a,
//             0x39f6d3a994cebea3,
//         ],
//         [
//             0x066da6782f2da170,
//             0x85c546f8cc60e47c,
//             0x44bf3da90590f3e1,
//             0x45e085f1b91a6cf1,
//         ],
//         [
//             0x16cf0224dcf9382c,
//             0x12dd7903b71baa93,
//             0xaf92c5362c204b76,
//             0x163147176461030d,
//         ],
//         [
//             0x10d6917f04735dea,
//             0x7e04a13731049a48,
//             0x42cbd9ab89d7b1f7,
//             0x60546bd624850b42,
//         ],
//     ];

//     let fft_settings = FFTSettings::from_scale(4).unwrap();

//     let mut data = vec![Fr::default(); fft_settings.max_width as usize];
//     for i in 0..fft_settings.max_width {
//         data[i as usize] = Fr::from_u64_arr(&[i as u64, 0, 0, 0]);
//     }

//     let forward_result = fft_fr(&data, true, &fft_settings).unwrap();

//     assert_eq!(inv_fft_expected.len(), fft_settings.max_width as usize);
//     for i in 0..inv_fft_expected.len() {
//         let expected = Fr::from_u64_arr(&inv_fft_expected[i]);
//         // println!("EXPECTED: {:?}", expected);
//         // println!("EXPECTED2: {:?}", inv_fft_expected[i]);
//         // println!("RAND: {:?}", Fr::from_u64(1));
//         // println!("FROWRAD: {:?}", forward_result[i]);
//         assert!(expected.equals(&forward_result[i]));
//     }
// }
