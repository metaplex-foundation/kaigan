use std::fmt::Debug;
use std::io::Write;
use std::ops::{Deref, DerefMut};

use borsh::maybestd::io::Read;
use borsh::{BorshDeserialize, BorshSerialize};

/// A vector that deserializes from a stream of bytes.
///
/// This is useful for deserializing a vector that does not have
/// a length prefix. In order to determine how many elements to deserialize,
/// the type of the elements must implement the trait `Sized`.
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RemainderVec<T: BorshSerialize + BorshDeserialize>(Vec<T>);

/// Deferences the inner `Vec` type.
impl<T> Deref for RemainderVec<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Deferences the inner `Vec` type as mutable.
impl<T> DerefMut for RemainderVec<T>
where
    T: BorshSerialize + BorshDeserialize,
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
    T: BorshSerialize + BorshDeserialize + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}

impl<T> BorshDeserialize for RemainderVec<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::maybestd::io::Result<Self> {
        let length = std::mem::size_of::<T>();
        // buffer to read the data
        let mut buffer = vec![0u8; length];
        // vec to store the items
        let mut items: Vec<T> = Vec::new();

        loop {
            match reader.read(&mut buffer)? {
                0 => break,
                n if n == length => items.push(T::deserialize(&mut buffer.as_slice())?),
                e => {
                    return Err(borsh::maybestd::io::Error::new(
                        borsh::maybestd::io::ErrorKind::InvalidData,
                        format!("unexpected number of bytes (read {e}, expected {length})"),
                    ))
                }
            }
        }

        Ok(Self(items))
    }
}

impl<T> BorshSerialize for RemainderVec<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    fn serialize<W: Write>(&self, writer: &mut W) -> borsh::maybestd::io::Result<()> {
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

        let mut data = [0u8; 40];
        source.serialize(&mut data.as_mut_slice()).unwrap();

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

        assert_eq!(error.kind(), borsh::maybestd::io::ErrorKind::InvalidData);
    }
}
