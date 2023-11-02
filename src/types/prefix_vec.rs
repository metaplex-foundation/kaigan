use std::fmt::Debug;
use std::io::Write;
use std::ops::{Deref, DerefMut};

use borsh::maybestd::io::Read;
use borsh::{BorshDeserialize, BorshSerialize};

/// Macro to automate the generation of `PrefixVec` types.
macro_rules! prefix_vec_types {
    ( ($n:tt, $p:tt), $(($name:tt, $prefix:tt)),+ ) => {
        prefix_vec_types!(($n, $p));
        prefix_vec_types!($( ($name, $prefix) ),+);
    };
    ( ($name:tt, $prefix_type:tt) ) => {
        /// A vector where the element data is prefixed by the vector length.
        #[derive(Clone, Eq, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $name<T: BorshSerialize + BorshDeserialize>(Vec<T>);

        /// Deferences the inner `Vec` type.
        impl<T> Deref for $name<T>
        where
            T: BorshSerialize + BorshDeserialize,
        {
            type Target = Vec<T>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        /// Deferences the inner `Vec` type as mutable.
        impl<T> DerefMut for $name<T>
        where
            T: BorshSerialize + BorshDeserialize,
        {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        /// `Debug` implementation.
        ///
        /// This implementation simply forwards to the inner `Vec` type.
        impl<T> Debug for $name<T>
        where
            T: BorshSerialize + BorshDeserialize + Debug,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_fmt(format_args!("{:?}", self.0))
            }
        }

        impl<T> BorshDeserialize for $name<T>
        where
            T: BorshSerialize + BorshDeserialize,
        {
            fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::maybestd::io::Result<Self> {
                // read the length of the vec
                let mut buffer = vec![0u8; std::mem::size_of::<$prefix_type>()];
                reader.read_exact(&mut buffer)?;
                let length = $prefix_type::deserialize(&mut buffer.as_slice())? as usize;

                // buffer to read each item
                let item_length = std::mem::size_of::<T>();
                let mut buffer = vec![0u8; item_length];
                // vec to store the items
                let mut items: Vec<T> = Vec::with_capacity(length);

                while items.len() < length {
                    match reader.read(&mut buffer)? {
                        0 => break,
                        n if n == item_length => {
                            items.push(T::deserialize(&mut buffer.as_slice())?)
                        }
                        e => {
                            return Err(borsh::maybestd::io::Error::new(
                                borsh::maybestd::io::ErrorKind::InvalidData,
                                format!(
                                    "unexpected number of bytes (read {e}, expected {item_length})"
                                ),
                            ))
                        }
                    }
                }

                if items.len() != length {
                    return Err(borsh::maybestd::io::Error::new(
                        borsh::maybestd::io::ErrorKind::InvalidData,
                        format!(
                            "unexpected vec length (read {}, expected {length})",
                            items.len()
                        ),
                    ));
                }

                Ok(Self(items))
            }
        }

        impl<T> BorshSerialize for $name<T>
        where
            T: BorshSerialize + BorshDeserialize,
        {
            fn serialize<W: Write>(&self, writer: &mut W) -> borsh::maybestd::io::Result<()> {
                if self.0.len() > $prefix_type::MAX as usize {
                    return Err(borsh::maybestd::io::Error::new(
                        borsh::maybestd::io::ErrorKind::InvalidData,
                        format!(
                            "size of vec too big for type: {} > {}",
                            self.0.len(),
                            $prefix_type::MAX
                        ),
                    ));
                }
                // add the length prefix
                $prefix_type::serialize(&(self.0.len() as $prefix_type), writer)?;
                // serialize each item
                for item in self.0.iter() {
                    item.serialize(writer)?;
                }

                Ok(())
            }
        }
    };
}

prefix_vec_types!(
    (U8PrefixVec, u8),
    (U16PrefixVec, u16),
    (U32PrefixVec, u32),
    (U64PrefixVec, u64)
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_data() {
        // slices of bytes (u8 length + 3 u64 values)
        let mut data = [0u8; 25];
        data[0..1].copy_from_slice(u8::to_le_bytes(3).as_slice());
        data[1..9].copy_from_slice(u64::to_le_bytes(15).as_slice());
        data[9..17].copy_from_slice(u64::to_le_bytes(7).as_slice());
        data[17..].copy_from_slice(u64::to_le_bytes(10).as_slice());

        let vec = U8PrefixVec::<u64>::try_from_slice(&data).unwrap();

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[15, 7, 10]);

        // slices of bytes (u16 length + 3 u64 values)
        let mut data = [0u8; 26];
        data[0..2].copy_from_slice(u16::to_le_bytes(3).as_slice());
        data[2..10].copy_from_slice(u64::to_le_bytes(15).as_slice());
        data[10..18].copy_from_slice(u64::to_le_bytes(7).as_slice());
        data[18..].copy_from_slice(u64::to_le_bytes(10).as_slice());

        let vec = U16PrefixVec::<u64>::try_from_slice(&data).unwrap();

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[15, 7, 10]);

        // slices of bytes (u32 length + 3 u64 values)
        let mut data = [0u8; 28];
        data[0..4].copy_from_slice(u32::to_le_bytes(3).as_slice());
        data[4..12].copy_from_slice(u64::to_le_bytes(15).as_slice());
        data[12..20].copy_from_slice(u64::to_le_bytes(7).as_slice());
        data[20..].copy_from_slice(u64::to_le_bytes(10).as_slice());

        let vec = U32PrefixVec::<u64>::try_from_slice(&data).unwrap();

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[15, 7, 10]);

        // slices of bytes (u64 length + 3 u64 values)
        let mut data = [0u8; 32];
        data[0..8].copy_from_slice(u64::to_le_bytes(3).as_slice());
        data[8..16].copy_from_slice(u64::to_le_bytes(15).as_slice());
        data[16..24].copy_from_slice(u64::to_le_bytes(7).as_slice());
        data[24..].copy_from_slice(u64::to_le_bytes(10).as_slice());

        let vec = U64PrefixVec::<u64>::try_from_slice(&data).unwrap();

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[15, 7, 10]);
    }

    #[test]
    fn serialize_data() {
        // u8 length
        let values = (0..10).collect::<Vec<u32>>();
        let source = U8PrefixVec::<u32>(values);

        let mut data = [0u8; 41];
        source.serialize(&mut data.as_mut_slice()).unwrap();

        let restored = U8PrefixVec::<u32>::try_from_slice(&data).unwrap();

        assert_eq!(restored.len(), source.len());
        assert_eq!(restored.as_slice(), source.as_slice());

        // u16 length
        let values = (0..10).collect::<Vec<u32>>();
        let source = U16PrefixVec::<u32>(values);

        let mut data = [0u8; 42];
        source.serialize(&mut data.as_mut_slice()).unwrap();

        let restored = U16PrefixVec::<u32>::try_from_slice(&data).unwrap();

        assert_eq!(restored.len(), source.len());
        assert_eq!(restored.as_slice(), source.as_slice());

        // u32 length
        let values = (0..10).collect::<Vec<u32>>();
        let source = U32PrefixVec::<u32>(values);

        let mut data = [0u8; 44];
        source.serialize(&mut data.as_mut_slice()).unwrap();

        let restored = U32PrefixVec::<u32>::try_from_slice(&data).unwrap();

        assert_eq!(restored.len(), source.len());
        assert_eq!(restored.as_slice(), source.as_slice());

        // u64 length

        let values = (0..10).collect::<Vec<u32>>();
        let source = U64PrefixVec::<u32>(values);

        let mut data = [0u8; 48];
        source.serialize(&mut data.as_mut_slice()).unwrap();

        let restored = U64PrefixVec::<u32>::try_from_slice(&data).unwrap();

        assert_eq!(restored.len(), source.len());
        assert_eq!(restored.as_slice(), source.as_slice());
    }

    #[test]
    fn fail_deserialize_invalid_data() {
        // slices of bytes (3 u64 values) + 4 bytes
        let mut data = [0u8; 28];
        data[0..8].copy_from_slice(u64::to_le_bytes(5).as_slice());
        data[8..16].copy_from_slice(u64::to_le_bytes(15).as_slice());
        data[16..24].copy_from_slice(u64::to_le_bytes(7).as_slice());

        let error = U64PrefixVec::<u64>::try_from_slice(&data).unwrap_err();

        assert_eq!(error.kind(), borsh::maybestd::io::ErrorKind::InvalidData);
    }

    #[test]
    fn fail_deserialize_invalid_length() {
        // slices of bytes (u64 length + 3 u64 values)
        let mut data = [0u8; 32];
        data[0..8].copy_from_slice(u64::to_le_bytes(2).as_slice());
        data[8..16].copy_from_slice(u64::to_le_bytes(15).as_slice());
        data[16..24].copy_from_slice(u64::to_le_bytes(7).as_slice());
        data[24..].copy_from_slice(u64::to_le_bytes(10).as_slice());

        let error = U64PrefixVec::<u64>::try_from_slice(&data).unwrap_err();

        assert_eq!(error.kind(), borsh::maybestd::io::ErrorKind::InvalidData);
    }

    #[test]
    fn fail_serialize_invalid_length_type() {
        // u8 length
        let values = (0..256).collect::<Vec<u32>>();
        let source = U8PrefixVec::<u32>(values);

        let mut data = [0u8; 41];
        let error = source.serialize(&mut data.as_mut_slice()).unwrap_err();

        assert_eq!(error.kind(), borsh::maybestd::io::ErrorKind::InvalidData);
    }
}
