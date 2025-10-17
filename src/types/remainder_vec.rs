use std::fmt::Debug;
#[cfg(not(feature = "anchor"))]
use std::io::Write;
use std::ops::{Deref, DerefMut};

#[cfg(feature = "anchor")]
use anchor_lang::prelude::{
    AnchorDeserialize as CrateDeserialize, AnchorSerialize as CrateSerialize,
};
#[cfg(not(feature = "anchor"))]
use borsh_1_5::io::Read;
#[cfg(not(feature = "anchor"))]
use borsh::{BorshDeserialize as CrateDeserialize, BorshSerialize as CrateSerialize};

/// A vector that deserializes from a stream of bytes.
///
/// This is useful for deserializing a vector that does not have
/// a length prefix. In order to determine how many elements to deserialize,
/// the type of the elements must implement the trait `Sized`.
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "anchor",
    derive(
        anchor_lang::prelude::AnchorSerialize,
        anchor_lang::prelude::AnchorDeserialize
    )
)]
pub struct RemainderVec<T: CrateSerialize + CrateDeserialize>(Vec<T>);

/// Dereferences the inner `Vec` type.
impl<T> Deref for RemainderVec<T>
where
    T: CrateSerialize + CrateDeserialize,
{
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Dereferences the inner `Vec` type as mutable.
impl<T> DerefMut for RemainderVec<T>
where
    T: CrateSerialize + CrateDeserialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// `Debug` implementation for `RemainderVec`.
///
/// This implementation simply forwards to the inner `Vec` type.
impl<T> Debug for RemainderVec<T>
where
    T: CrateSerialize + CrateDeserialize + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}

#[cfg(not(feature = "anchor"))]
impl<T> CrateDeserialize for RemainderVec<T>
where
    T: CrateSerialize + CrateDeserialize,
{
    fn deserialize_reader<R: Read>(reader: &mut R) -> borsh_1_5::io::Result<Self> {
        let mut items: Vec<T> = Vec::new();

        while let Ok(item) = T::deserialize_reader(reader) {
            items.push(item);
        }

        Ok(Self(items))
    }
}

#[cfg(not(feature = "anchor"))]
impl<T> CrateSerialize for RemainderVec<T>
where
    T: CrateSerialize + CrateDeserialize,
{
    fn serialize<W: Write>(&self, writer: &mut W) -> borsh_1_5::io::Result<()> {
        // serialize each item without adding a prefix for the length
        for item in self.0.iter() {
            item.serialize(writer)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_data() {
        // slices of bytes (3 u64 values)
        let mut data = [0u8; 24];
        data[0..8].copy_from_slice(u64::to_le_bytes(5).as_slice());
        data[8..16].copy_from_slice(u64::to_le_bytes(15).as_slice());
        data[16..].copy_from_slice(u64::to_le_bytes(7).as_slice());

        let vec = RemainderVec::<u64>::try_from_slice(&data).unwrap();

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[5, 15, 7]);
    }

    #[test]
    fn serialize_data() {
        let values = (0..10).collect::<Vec<u32>>();
        let source = RemainderVec::<u32>(values);

        let mut data = Vec::new();
        source.serialize(&mut data).unwrap();

        let restored = RemainderVec::<u32>::try_from_slice(&data).unwrap();

        assert_eq!(restored.len(), source.len());
        assert_eq!(restored.as_slice(), source.as_slice());
    }

    #[test]
    fn fail_deserialize_invalid_data_length() {
        // slices of bytes (3 u64 values) + 4 bytes
        let mut data = [0u8; 28];
        data[0..8].copy_from_slice(u64::to_le_bytes(5).as_slice());
        data[8..16].copy_from_slice(u64::to_le_bytes(15).as_slice());
        data[16..24].copy_from_slice(u64::to_le_bytes(7).as_slice());

        let error = RemainderVec::<u64>::try_from_slice(&data).unwrap_err();

        assert_eq!(error.kind(), borsh_1_5::io::ErrorKind::InvalidData);
    }
}
