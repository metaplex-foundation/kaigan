use borsh::io::Read;
use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt::Debug;
use std::io::Write;
use std::ops::Deref;
use std::str::FromStr;

/// A wrapped `str` type.
///
/// This is useful for deserializing a string value that does not have
/// a length prefix.
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RemainderStr(String);

impl RemainderStr {
    pub fn from(value: String) -> Self {
        value.into()
    }
}

impl FromStr for RemainderStr {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Deref for RemainderStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for RemainderStr {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for RemainderStr {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Debug for RemainderStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}

impl BorshDeserialize for RemainderStr {
    fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
        let mut value: String = String::new();
        while let Ok(c) = u8::deserialize_reader(reader) {
            value.push(c as char);
        }
        Ok(Self(value))
    }
}

impl BorshSerialize for RemainderStr {
    fn serialize<W: Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
        // serialize the bytes of the string without adding a prefix
        for c in self.0.as_bytes() {
            c.serialize(writer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_data() {
        // slices of bytes
        let mut data = [0u8; 3];
        data[0] = b's';
        data[1] = b't';
        data[2] = b'r';

        let str = RemainderStr::try_from_slice(&data).unwrap();

        assert_eq!(str.len(), 3);
        assert_eq!(str.deref(), "str");
    }

    #[test]
    fn serialize_data() {
        let source: RemainderStr = "this is a longer str".into();

        let mut data = [0u8; "this is a longer str".len()];
        source.serialize(&mut data.as_mut_slice()).unwrap();

        let restored = RemainderStr::try_from_slice(&data).unwrap();

        assert_eq!(restored.len(), source.len());
        assert_eq!(restored, source);
    }
}
