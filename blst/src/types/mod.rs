pub mod fft_settings;
pub mod fk20_multi_settings;
pub mod fk20_single_settings;
pub mod fp;
pub mod fr;
pub mod g1;
pub mod g2;
pub mod kzg_settings;
pub mod poly;

#[macro_export]
macro_rules! impl_serde {
    ($blst_type:ty, $bytes_per_type:expr) => {
        #[cfg(feature = "serde")]
        impl serde::Serialize for $blst_type {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                use serde_big_array::BigArray;
                <[_; $bytes_per_type] as BigArray<u8>>::serialize(&self.to_bytes(), s)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $blst_type {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                use serde_big_array::BigArray;
                let bytes = <[u8; $bytes_per_type] as BigArray<u8>>::deserialize(d)?;
                Ok(Self::from_bytes(&bytes).unwrap())
            }
        }
    };
}