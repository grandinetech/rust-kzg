#[cfg(test)]
mod tests {
    use kzg::{Fr, G1Affine, G1Mul, G1};
    use rust_kzg_constantine::types::fr::CtFr;
    use rust_kzg_constantine::types::g1::{CtG1, CtG1Affine};

    #[test]
    fn test_uncompressed_serialization_roundtrip() {
        // Test with generator
        let point = CtG1::generator();
        let affine = CtG1Affine::into_affine(&point);

        let bytes = affine.to_bytes_uncompressed();
        let recovered = CtG1Affine::from_bytes_uncompressed(bytes).expect("Failed to deserialize");

        assert_eq!(affine, recovered, "Generator roundtrip failed");
    }

    #[test]
    fn test_uncompressed_serialization_infinity() {
        let point = CtG1Affine::zero();

        let bytes = point.to_bytes_uncompressed();

        // Check that infinity flag is set (bit 6 of first byte)
        assert_eq!(bytes[0], 0x40, "Infinity flag not set correctly");

        // All other bytes should be zero
        for &byte in &bytes[1..] {
            assert_eq!(byte, 0, "Non-flag bytes should be zero for infinity");
        }

        let recovered =
            CtG1Affine::from_bytes_uncompressed(bytes).expect("Failed to deserialize infinity");
        assert!(
            recovered.is_infinity(),
            "Deserialized point should be infinity"
        );
    }

    #[test]
    fn test_uncompressed_serialization_random_points() {
        // Test multiple random points
        for i in 1..10 {
            let scalar = CtFr::from_u64(i * 12345 + 67890);
            let point = CtG1::generator().mul(&scalar);
            let affine = CtG1Affine::into_affine(&point);

            let bytes = affine.to_bytes_uncompressed();
            let recovered = CtG1Affine::from_bytes_uncompressed(bytes)
                .unwrap_or_else(|e| panic!("Failed to deserialize point {}: {}", i, e));

            assert_eq!(affine, recovered, "Roundtrip failed for point {}", i);
        }
    }

    #[test]
    fn test_uncompressed_with_high_bit_coordinates() {
        // Create points where coordinates might have high bits set
        // Use large scalar multipliers to get varied coordinate values
        let large_scalar = CtFr::from_u64(u64::MAX - 1);
        let point = CtG1::generator().mul(&large_scalar);
        let affine = CtG1Affine::into_affine(&point);

        let bytes = affine.to_bytes_uncompressed();

        // Verify that flag bits are NOT set in serialized form for non-infinity point
        // (compression bit should be 0, infinity bit should be 0)
        assert_eq!(
            bytes[0] & 0xC0,
            0,
            "Unexpected flag bits set for regular point"
        );

        let recovered = CtG1Affine::from_bytes_uncompressed(bytes)
            .expect("Failed to deserialize point with high bit coordinates");

        assert_eq!(
            affine, recovered,
            "Roundtrip failed for point with high bit coordinates"
        );
    }

    #[test]
    fn test_known_generator_bytes() {
        // Get the actual serialization from constantine for the generator
        // This serves as a regression test - the bytes should remain consistent
        let generator = CtG1::generator();
        let affine = CtG1Affine::into_affine(&generator);
        let bytes = affine.to_bytes_uncompressed();

        // Verify it deserializes correctly
        let recovered = CtG1Affine::from_bytes_uncompressed(bytes)
            .expect("Failed to deserialize generator bytes");
        assert_eq!(affine, recovered, "Generator round-trip failed");

        // Verify no flags are set (should be 0 for uncompressed, non-infinity point)
        assert_eq!(
            bytes[0] & 0xE0,
            0,
            "Unexpected flags set in generator serialization"
        );

        // Print bytes for cross-backend verification (useful for manual testing)
        #[cfg(feature = "std")]
        {
            println!("Generator bytes (hex): {}", hex::encode(&bytes[..]));
            println!("First 8 bytes: {:02x?}", &bytes[0..8]);
            println!("Bytes 48-56: {:02x?}", &bytes[48..56]);
        }
    }

    #[test]
    fn test_known_infinity_bytes() {
        // Infinity point should serialize to all zeros except the infinity flag
        let expected_bytes = {
            let mut bytes = [0u8; 96];
            bytes[0] = 0x40; // Only infinity flag set
            bytes
        };

        let infinity = CtG1Affine::zero();
        let bytes = infinity.to_bytes_uncompressed();

        assert_eq!(
            bytes, expected_bytes,
            "Infinity serialization doesn't match expected bytes"
        );

        // Verify round-trip
        let recovered = CtG1Affine::from_bytes_uncompressed(expected_bytes)
            .expect("Failed to deserialize known infinity bytes");
        assert!(
            recovered.is_infinity(),
            "Recovered point should be infinity"
        );
    }

    // // generate points to replace hex::decode in test_uncompressed_known_points()
    // #[test]
    // fn print_known_points() {
    //     for k in [5u64, 7, 1235, 9999] {
    //         let point = CtG1::generator().mul(&CtFr::from_u64(k));
    //         let affine = CtG1Affine::into_affine(&point);
    //         let bytes = affine.to_bytes_uncompressed();
    //         println!("g * {} = {}", k, hex::encode(bytes));
    //     }
    // }

    #[test]
    fn test_uncompressed_known_points() {
        // g * 5
        let point = CtG1::generator().mul(&CtFr::from_u64(5));
        let affine = CtG1Affine::into_affine(&point);
        let bytes = affine.to_bytes_uncompressed();
        let expected = hex::decode("0befb962052d5be4fa0cd24153cb8d710593bb36cbf5d7d289f3afc44b4fd7f2b031cca2d46d20db6c3aea956edc0d65149a008e9c0217f87906a800fc343bbb83af773c28f5fce6f978a25d58dd410239ff5eb5d2a4f6b9afc43ba27a23863c").unwrap();
        assert_eq!(bytes.to_vec(), expected);
        assert!(affine.eq(&CtG1Affine::from_bytes_uncompressed(bytes).unwrap()));

        //g * 7
        let point = CtG1::generator().mul(&CtFr::from_u64(7));
        let affine = CtG1Affine::into_affine(&point);
        let bytes = affine.to_bytes_uncompressed();
        let expected = hex::decode("0d3056fc0db4365ff29c6756cc2dd13a5508d390e218ff49a8588f9235e2e40d018298254a48192dbf6f80fad9849c75092ab1a757dc00ed54b15b24176ee9d24869c5dda756bdfea158f28b5eff937fed6b86ac4a437cb894ceeaaf25173e97").unwrap();
        assert_eq!(bytes.to_vec(), expected);
        assert!(affine.eq(&CtG1Affine::from_bytes_uncompressed(bytes).unwrap()));

        // g * 1235
        let point = CtG1::generator().mul(&CtFr::from_u64(1235));
        let affine = CtG1Affine::into_affine(&point);
        let bytes = affine.to_bytes_uncompressed();
        let expected = hex::decode("18b5fee2a1cdf7726124be509790e12d6169df13c2fdf7bd127a0872fe117575426f9fd0464c15b53f766c1861d510f312c51fdeaff5ec6ae20025033bcb4d6ed6ed064abfbbad523187ec727c6f858cfda248ff03da74e292523519a1a4d56e").unwrap();
        assert_eq!(bytes.to_vec(), expected);
        assert!(affine.eq(&CtG1Affine::from_bytes_uncompressed(bytes).unwrap()));

        // g * 9999
        let point = CtG1::generator().mul(&CtFr::from_u64(9999));
        let affine = CtG1Affine::into_affine(&point);
        let bytes = affine.to_bytes_uncompressed();
        let expected = hex::decode("0da245189a3242ca54c7397d77285fc0252722b8a53366efd1080f5752128206194d7d882b2a3becd6a52cf1faebf0e8198ee528c3360a55cfe7ca3e92c75546dd5800bb46f204e8ddc8491dfefb4b37e57c024a45e58c75aba1797c439ae799").unwrap();
        assert_eq!(bytes.to_vec(), expected);
        assert!(affine.eq(&CtG1Affine::from_bytes_uncompressed(bytes).unwrap()));
    }

    #[test]
    fn test_msb_handling() {
        // BLS12-381 field prime is 381 bits, so top 3 bits of 384-bit representation are unused
        // The field modulus is: 0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab
        // This means valid field elements can have bits set up to bit 380

        use kzg::{Fr, G1Mul};

        // Create points with large scalar multipliers to get varied coordinates
        for scalar_val in [u64::MAX, u64::MAX - 1, u64::MAX / 2, 1u64 << 63] {
            let scalar = CtFr::from_u64(scalar_val);
            let point = CtG1::generator().mul(&scalar);
            let affine = CtG1Affine::into_affine(&point);

            let bytes = affine.to_bytes_uncompressed();

            // Verify top 3 bits are clear (flag bits)
            assert_eq!(
                bytes[0] & 0xE0,
                0,
                "Top 3 bits should be clear for scalar {}",
                scalar_val
            );

            // Verify round-trip
            let recovered = CtG1Affine::from_bytes_uncompressed(bytes).unwrap_or_else(|e| {
                panic!(
                    "Failed to deserialize point with scalar {}: {}",
                    scalar_val, e
                )
            });

            assert_eq!(
                affine, recovered,
                "Round-trip failed for scalar {}",
                scalar_val
            );

            // Verify the point is still valid by checking compressed form matches
            let compressed1 = point.to_bytes();
            let compressed2 = recovered.to_proj().to_bytes();
            assert_eq!(
                compressed1, compressed2,
                "Compressed forms don't match for scalar {}",
                scalar_val
            );
        }
    }

    #[test]
    fn test_invalid_compression_flag() {
        let mut bytes = [0u8; 96];
        bytes[0] = 0x80; // Set compression flag

        let result = CtG1Affine::from_bytes_uncompressed(bytes);
        assert!(
            result.is_err(),
            "Should reject compressed flag in uncompressed format"
        );
    }

    #[test]
    fn test_invalid_sort_flag() {
        let mut bytes = [0u8; 96];
        bytes[0] = 0x20; // Set sort flag

        let result = CtG1Affine::from_bytes_uncompressed(bytes);
        assert!(
            result.is_err(),
            "Should reject sort flag in uncompressed format"
        );
    }

    #[test]
    fn test_invalid_infinity_encoding() {
        let mut bytes = [0u8; 96];
        bytes[0] = 0x40; // Set infinity flag
        bytes[1] = 0x01; // But has non-zero data

        let result = CtG1Affine::from_bytes_uncompressed(bytes);
        assert!(
            result.is_err(),
            "Should reject infinity point with non-zero data"
        );
    }
}
