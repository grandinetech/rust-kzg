use blst::{blst_fp, blst_fp2, blst_p1, blst_p2};

use crate::types::g1::FsG1;
use crate::types::g2::FsG2;

use num_bigint::BigUint;
use std::str::FromStr;
use std::str;

pub const G1_IDENTITY: FsG1 = FsG1::from_xyz(
    blst_fp { l: [0; 6] },
    blst_fp { l: [0; 6] },
    blst_fp { l: [0; 6] },
);

pub const SCALE_FACTOR: u64 = 5;

pub const SAMPLES_PER_BLOB: u64 = 4;
pub const FIELD_ELEMENTS_PER_SAMPLE: u64 = 8;

pub const NUM_ROOTS: usize = (SAMPLES_PER_BLOB * FIELD_ELEMENTS_PER_SAMPLE) as usize;

pub fn roots_of_unity_arr() -> [[u64; 4]; 32] {
    assert_eq!(NUM_ROOTS, 32);

    let mut roots: [[u64; 4]; 32] = [[0; 4]; 32];
    let roots_u256 = roots_of_unity_u256(NUM_ROOTS as u64 /* order */);

    let mut column = 0;
    for root in &roots_u256 {
        let fr_hex = format!("{:0>64X}", root);
        let subs = fr_hex.as_bytes()
            .chunks(16)
            .map(|buf| unsafe { str::from_utf8_unchecked(buf) })
            .collect::<Vec<&str>>();
        roots[column][0] = u64::from_str_radix(subs[3], 16).unwrap();
        roots[column][1] = u64::from_str_radix(subs[2], 16).unwrap();
        roots[column][2] = u64::from_str_radix(subs[1], 16).unwrap();
        roots[column][3] = u64::from_str_radix(subs[0], 16).unwrap();
        column += 1;
    }

    roots
}

pub fn roots_of_unity_u256(order: u64) -> Vec<BigUint> {
    let bls_modulus = BigUint::from_str("52435875175126190479447740508185965837690552500527637822603658699938581184513").unwrap();
    let primitive_root_of_unity = BigUint::from_str("7").unwrap();

    assert_eq!((&bls_modulus - 1u8) % order, BigUint::from_str("0").unwrap());
    assert_eq!(NUM_ROOTS, 32);

    let mut roots: Vec<BigUint> = vec!{};
    let root_of_unity = primitive_root_of_unity.modpow(&((&bls_modulus - 1u8) / order), &bls_modulus);
    let mut current_root_of_unity = BigUint::from(1u8);
    for _ in 0..NUM_ROOTS {
        roots.push(current_root_of_unity.clone());
        current_root_of_unity = (current_root_of_unity * &root_of_unity) % &bls_modulus;
    }

    roots
}

pub const G1_GENERATOR: FsG1 = FsG1 {
    0: blst_p1 {
        x: blst_fp {
            l: [
                0x5cb38790fd530c16,
                0x7817fc679976fff5,
                0x154f95c7143ba1c1,
                0xf0ae6acdf3d0e747,
                0xedce6ecc21dbf440,
                0x120177419e0bfb75,
            ],
        },
        y: blst_fp {
            l: [
                0xbaac93d50ce72271,
                0x8c22631a7918fd8e,
                0xdd595f13570725ce,
                0x51ac582950405194,
                0x0e1c8c3fad0059c0,
                0x0bbc3efc5008a26a,
            ],
        },
        z: blst_fp {
            l: [
                0x760900000002fffd,
                0xebf4000bc40c0002,
                0x5f48985753c758ba,
                0x77ce585370525745,
                0x5c071a97a256ec6d,
                0x15f65ec3fa80e493,
            ],
        },
    },
};

pub const G1_NEGATIVE_GENERATOR: FsG1 = FsG1 {
    0: blst_p1 {
        x: blst_fp {
            l: [
                0x5cb38790fd530c16,
                0x7817fc679976fff5,
                0x154f95c7143ba1c1,
                0xf0ae6acdf3d0e747,
                0xedce6ecc21dbf440,
                0x120177419e0bfb75,
            ],
        },
        y: blst_fp {
            l: [
                0xff526c2af318883a,
                0x92899ce4383b0270,
                0x89d7738d9fa9d055,
                0x12caf35ba344c12a,
                0x3cff1b76964b5317,
                0x0e44d2ede9774430,
            ],
        },
        z: blst_fp {
            l: [
                0x760900000002fffd,
                0xebf4000bc40c0002,
                0x5f48985753c758ba,
                0x77ce585370525745,
                0x5c071a97a256ec6d,
                0x15f65ec3fa80e493,
            ],
        },
    },
};

pub const G2_GENERATOR: FsG2 = FsG2 {
    0: blst_p2 {
        x: blst_fp2 {
            fp: [
                blst_fp {
                    l: [
                        0xf5f28fa202940a10,
                        0xb3f5fb2687b4961a,
                        0xa1a893b53e2ae580,
                        0x9894999d1a3caee9,
                        0x6f67b7631863366b,
                        0x058191924350bcd7,
                    ],
                },
                blst_fp {
                    l: [
                        0xa5a9c0759e23f606,
                        0xaaa0c59dbccd60c3,
                        0x3bb17e18e2867806,
                        0x1b1ab6cc8541b367,
                        0xc2b6ed0ef2158547,
                        0x11922a097360edf3,
                    ],
                },
            ],
        },
        y: blst_fp2 {
            fp: [
                blst_fp {
                    l: [
                        0x4c730af860494c4a,
                        0x597cfa1f5e369c5a,
                        0xe7e6856caa0a635a,
                        0xbbefb5e96e0d495f,
                        0x07d3a975f0ef25a2,
                        0x0083fd8e7e80dae5,
                    ],
                },
                blst_fp {
                    l: [
                        0xadc0fc92df64b05d,
                        0x18aa270a2b1461dc,
                        0x86adac6a3be4eba0,
                        0x79495c4ec93da33a,
                        0xe7175850a43ccaed,
                        0x0b2bc2a163de1bf2,
                    ],
                },
            ],
        },
        z: blst_fp2 {
            fp: [
                blst_fp {
                    l: [
                        0x760900000002fffd,
                        0xebf4000bc40c0002,
                        0x5f48985753c758ba,
                        0x77ce585370525745,
                        0x5c071a97a256ec6d,
                        0x15f65ec3fa80e493,
                    ],
                },
                blst_fp {
                    l: [
                        0x0000000000000000,
                        0x0000000000000000,
                        0x0000000000000000,
                        0x0000000000000000,
                        0x0000000000000000,
                        0x0000000000000000,
                    ],
                },
            ],
        },
    },
};

pub const G2_NEGATIVE_GENERATOR: FsG2 = FsG2 {
    0: blst_p2 {
        x: blst_fp2 {
            fp: [
                blst_fp {
                    l: [
                        0xf5f28fa202940a10,
                        0xb3f5fb2687b4961a,
                        0xa1a893b53e2ae580,
                        0x9894999d1a3caee9,
                        0x6f67b7631863366b,
                        0x058191924350bcd7,
                    ],
                },
                blst_fp {
                    l: [
                        0xa5a9c0759e23f606,
                        0xaaa0c59dbccd60c3,
                        0x3bb17e18e2867806,
                        0x1b1ab6cc8541b367,
                        0xc2b6ed0ef2158547,
                        0x11922a097360edf3,
                    ],
                },
            ],
        },
        y: blst_fp2 {
            fp: [
                blst_fp {
                    l: [
                        0x6d8bf5079fb65e61,
                        0xc52f05df531d63a5,
                        0x7f4a4d344ca692c9,
                        0xa887959b8577c95f,
                        0x4347fe40525c8734,
                        0x197d145bbaff0bb5,
                    ],
                },
                blst_fp {
                    l: [
                        0x0c3e036d209afa4e,
                        0x0601d8f4863f9e23,
                        0xe0832636bacc0a84,
                        0xeb2def362a476f84,
                        0x64044f659f0ee1e9,
                        0x0ed54f48d5a1caa7,
                    ],
                },
            ],
        },
        z: blst_fp2 {
            fp: [
                blst_fp {
                    l: [
                        0x760900000002fffd,
                        0xebf4000bc40c0002,
                        0x5f48985753c758ba,
                        0x77ce585370525745,
                        0x5c071a97a256ec6d,
                        0x15f65ec3fa80e493,
                    ],
                },
                blst_fp {
                    l: [
                        0x0000000000000000,
                        0x0000000000000000,
                        0x0000000000000000,
                        0x0000000000000000,
                        0x0000000000000000,
                        0x0000000000000000,
                    ],
                },
            ],
        },
    },
};

pub const TRUSTED_SETUP_GENERATOR: [u8; 32usize] = [
    0xa4, 0x73, 0x31, 0x95, 0x28, 0xc8, 0xb6, 0xea, 0x4d, 0x08, 0xcc, 0x53, 0x18, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
