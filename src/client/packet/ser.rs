use bstr::{B, BString};
use serde::{Serialize, Serializer, ser::Impossible};

use super::error::PacketError;

#[derive(Clone)]
pub struct PacketSerializer {
    pub delimiter: &'static [u8],
    parts: Vec<BString>,
}
impl PacketSerializer {
    pub const fn new(delimiter: &'static [u8]) -> Self {
        Self {
            delimiter,
            parts: Vec::new(),
        }
    }
}
macro_rules! impl_number {
    ($name: ident, $type: ty) => {
        fn $name(self, v: $type) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_string().into())
        }
    };
}
macro_rules! impl_sub_serializer {
    ($fn: ident, $trait: path) => {
        impl $trait for PacketSerializer {
            type Ok = BString;

            type Error = PacketError;
            fn $fn<T>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: ?Sized + Serialize,
            {
                let part = value.serialize(self.clone())?;
                Ok(self.parts.push(part))
            }
            fn end(self) -> Result<Self::Ok, Self::Error> {
                Ok(bstr::join(self.delimiter, self.parts).into())
            }
        }
    };
}
impl_sub_serializer!(serialize_field, serde::ser::SerializeTupleStruct);
impl_sub_serializer!(serialize_element, serde::ser::SerializeSeq);
impl_sub_serializer!(serialize_element, serde::ser::SerializeTuple);
impl_sub_serializer!(serialize_field, serde::ser::SerializeTupleVariant);

impl serde::ser::SerializeStructVariant for PacketSerializer {
    type Ok = BString;

    type Error = PacketError;
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let part = value.serialize(self.clone())?;
        self.parts.push(part);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(bstr::join(self.delimiter, self.parts).into())
    }
}
pub struct PacketTupleSerializer {}
pub struct PacketTupleStructSerializer {}
impl Serializer for PacketSerializer {
    type Ok = BString;

    type Error = PacketError;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Impossible<Self::Ok, Self::Error>;

    type SerializeStruct = Impossible<Self::Ok, Self::Error>;

    type SerializeStructVariant = Self;

    impl_number!(serialize_u8, u8);
    impl_number!(serialize_u16, u16);
    impl_number!(serialize_u32, u32);
    impl_number!(serialize_u64, u64);
    impl_number!(serialize_u128, u128);
    impl_number!(serialize_i8, i8);
    impl_number!(serialize_i16, i16);
    impl_number!(serialize_i32, i32);
    impl_number!(serialize_i64, i64);
    impl_number!(serialize_i128, i128);
    impl_number!(serialize_f32, f32);
    impl_number!(serialize_f64, f64);

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(if v { b"1".into() } else { b"0".into() })
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(v.as_bytes().into())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0u8; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(v.into())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(b"null".into())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(BString::default())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(BString::default())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(variant.bytes().collect())
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let delim = self.delimiter;
        Ok([variant.bytes().collect(), value.serialize(self)?]
            .join(delim)
            .into())
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(Self::SerializeTuple {
            delimiter: self.delimiter,
            parts: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(Self::SerializeTupleStruct {
            delimiter: self.delimiter,
            parts: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        tracing::error!(
            "tried to serialize a map, which is not supported by the packet serializer, are you using #[serde(flatten)] somewhere?"
        );
        tracing::error!("{:?}", self.parts);
        unimplemented!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        unimplemented!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let mut parts = Vec::with_capacity(len + 1);
        parts.push(B(variant).into());
        Ok(Self::SerializeTupleVariant {
            delimiter: self.delimiter,
            parts,
        })
    }
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let mut parts = Vec::with_capacity(len + 1);
        parts.push(B(variant).into());
        Ok(Self::SerializeStructVariant {
            delimiter: self.delimiter,
            parts,
        })
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(Self::SerializeSeq {
            delimiter: self.delimiter,
            parts: len.map(Vec::with_capacity).unwrap_or_default(),
        })
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::{player::ids::PlayerId, room::client::packet::OutgoingPacket};
    #[test]
    fn test() {
        let packets_and_serialized = [
            (
                OutgoingPacket::Jump {
                    player_id: PlayerId(10),
                    x: 20,
                    y: 1,
                },
                bstr::B(b"jmp\xff\xff10\xff\xff20\xff\xff1"),
            ),
            (
                OutgoingPacket::BattleAnimation(PlayerId(999), 99),
                bstr::B(b"ba\xff\xff999\xff\xff99"),
            ),
        ];
        for (packet, result) in &packets_and_serialized {
            let serializer = PacketSerializer::new(b"\xFF\xFF");

            assert_eq!(
                packet.serialize(serializer),
                Ok(BString::new(result.to_vec()))
            );
        }
    }
}
