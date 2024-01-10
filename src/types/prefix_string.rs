use borsh::{
    maybestd::io::{Error, ErrorKind, Read, Result},
    BorshDeserialize, BorshSerialize,
};
use std::fmt::Debug;
use std::io::Write;
use std::ops::Deref;

/// Macro to automate the generation of `PrefixString` types.
macro_rules! prefix_string_types {
    ( ($n:tt, $p:tt), $(($name:tt, $prefix:tt)),+ ) => {
        prefix_string_types!(($n, $p));
        prefix_string_types!($( ($name, $prefix) ),+);
    };
    ( ($name:tt, $prefix_type:tt) ) => {
        /// A string prefixed by "custom" length type.
        #[derive(Clone, Eq, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $name(String);

        /// Deferences the inner `Vec` type.
        impl Deref for $name
        {
            type Target = String;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        /// `Debug` implementation.
        ///
        /// This implementation simply forwards to the inner `String` type.
        impl Debug for $name
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_fmt(format_args!("{:?}", self.0))
            }
        }

        impl BorshDeserialize for $name
        {
            fn deserialize_reader<R: Read>(reader: &mut R) -> Result<Self> {
                // read the length of the String
                let mut buffer = vec![0u8; std::mem::size_of::<$prefix_type>()];
                reader.read_exact(&mut buffer)?;
                let length = $prefix_type::deserialize(&mut buffer.as_slice())? as usize;

                let mut buffer = vec![0u8; length];
                reader.read_exact(&mut buffer)?;

                Ok(Self(String::from_utf8(buffer)
                    .map_err(|_| Error::new(
                        ErrorKind::InvalidData,
                        "invalid utf8"
                    )
                )?))
            }
        }

        impl BorshSerialize for $name
        {
            fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
                if self.0.len() > $prefix_type::MAX as usize {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "size of string too big for prefix type: {} > {}",
                            self.0.len(),
                            $prefix_type::MAX
                        ),
                    ));
                }
                // add the length prefix
                $prefix_type::serialize(&(self.0.len() as $prefix_type), writer)?;
                // serialize the string (without its "natural" prefix)
                writer.write_all(self.0.as_bytes())?;

                Ok(())
            }
        }
    };
}

// Generate the prefix vec types.
prefix_string_types!(
    (U8PrefixString, u8),
    (U16PrefixString, u16),
    (U64PrefixString, u64)
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_data() {
        // slices of bytes (u8 length + "string")
        let mut data = [0u8; 7];
        data[0] = 6;
        data[1..7].copy_from_slice("string".as_bytes());

        let string = U8PrefixString::try_from_slice(&data).unwrap();

        assert_eq!(string.len(), 6);
        assert_eq!(*string, "string");

        // slices of bytes (u16 length + "string")
        let mut data = [0u8; 8];
        data[0..2].copy_from_slice(u16::to_le_bytes(6).as_slice());
        data[2..8].copy_from_slice("string".as_bytes());

        let string = U16PrefixString::try_from_slice(&data).unwrap();

        assert_eq!(string.len(), 6);
        assert_eq!(*string, "string");

        // slices of bytes (u64 length + "string")
        let mut data = [0u8; 14];
        data[0..8].copy_from_slice(u64::to_le_bytes(6).as_slice());
        data[8..14].copy_from_slice("string".as_bytes());

        let string = U64PrefixString::try_from_slice(&data).unwrap();

        assert_eq!(string.len(), 6);
        assert_eq!(*string, "string");
    }

    #[test]
    fn serialize_data() {
        // u8 length
        let string = String::from("string");
        let source = U8PrefixString(string);

        let mut data = [0u8; 7];
        source.serialize(&mut data.as_mut_slice()).unwrap();

        println!("serialized data: {:?}", data);
        let restored = U8PrefixString::try_from_slice(&data).unwrap();

        assert_eq!(restored.len(), source.len());
        assert_eq!(*restored, *source);

        // u16 length
        let string = String::from("string");
        let source = U16PrefixString(string);

        let mut data = [0u8; 8];
        source.serialize(&mut data.as_mut_slice()).unwrap();

        let restored = U16PrefixString::try_from_slice(&data).unwrap();

        assert_eq!(restored.len(), source.len());
        assert_eq!(*restored, *source);

        // u64 length

        let string = String::from("string");
        let source = U64PrefixString(string);

        let mut data = [0u8; 14];
        source.serialize(&mut data.as_mut_slice()).unwrap();

        let restored = U64PrefixString::try_from_slice(&data).unwrap();

        assert_eq!(restored.len(), source.len());
        assert_eq!(*restored, *source);
    }

    #[test]
    fn fail_serialize_invalid_length_type() {
        // u8 length
        let string = "0".repeat(256);
        let source = U8PrefixString(string);

        let mut data = [0u8; 257];
        let error = source.serialize(&mut data.as_mut_slice()).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidData);
    }
}
