use std::collections::VecDeque;

use bstr::ByteSlice;
use serde::Deserializer;

use crate::room::client::packet::error::{self, PacketError};

pub struct PacketDeserializer<'de> {
    data: VecDeque<&'de bstr::BStr>,
}
impl<'de> PacketDeserializer<'de> {
    pub fn new(data: &'de bstr::BStr, delimiter: &'static bstr::BStr) -> Self {
        Self {
            data: data.split_str(delimiter).map(bstr::BStr::new).collect(),
        }
    }
    fn take_next_part(&mut self) -> Result<&'de bstr::BStr, PacketError> {
        self.data.pop_front().ok_or(PacketError::Incomplete)
    }
    fn peek_next_part(&self) -> Option<&&'de bstr::BStr> {
        self.data.front()
    }
    fn take_next_as_str(&mut self) -> Result<&'de str, PacketError> {
        str::from_utf8(self.take_next_part()?).map_err(|_| PacketError::Invalid("utf-8"))
    }
}
struct PacketDeserializerSeqAccess<'de, 'a> {
    de: &'a mut PacketDeserializer<'de>,
}
impl<'de, 'a> PacketDeserializerSeqAccess<'de, 'a> {
    const fn new(de: &'a mut PacketDeserializer<'de>) -> Self {
        Self { de }
    }
}
impl<'de> serde::de::SeqAccess<'de> for PacketDeserializerSeqAccess<'de, '_> {
    type Error = PacketError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de).map(Some)
    }
}
struct PacketDeserializerEnum<'de, 'a> {
    de: &'a mut PacketDeserializer<'de>,
}
impl<'de, 'a> PacketDeserializerEnum<'de, 'a> {
    const fn new(de: &'a mut PacketDeserializer<'de>) -> Self {
        Self { de }
    }
}
impl<'de> serde::de::EnumAccess<'de> for PacketDeserializerEnum<'de, '_> {
    type Error = PacketError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}
impl<'de> serde::de::VariantAccess<'de> for PacketDeserializerEnum<'de, '_> {
    type Error = PacketError;
    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }
    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.de.deserialize_seq(visitor)
    }
    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.de.deserialize_seq(visitor)
    }
}
macro_rules! impl_number {
    ($method_name: ident, $visit_name: ident, $type: ty) => {
        fn $method_name<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            visitor.$visit_name(
                (&*self.take_next_part()?)
                    .to_str()
                    .map_err(|_| PacketError::Invalid("utf-8"))?
                    .parse::<$type>()
                    .map_err(|_| PacketError::Invalid("number"))?,
            )
        }
    };
}

impl<'de> Deserializer<'de> for &mut PacketDeserializer<'de> {
    type Error = error::PacketError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_bool(match &**self.take_next_part()? {
            b"1" => Ok(true),
            b"0" => Ok(false),
            _ => Err(PacketError::Invalid("bool")),
        }?)
    }

    impl_number!(deserialize_i8, visit_i8, i8);
    impl_number!(deserialize_i16, visit_i16, i16);
    impl_number!(deserialize_i32, visit_i32, i32);
    impl_number!(deserialize_i64, visit_i64, i64);
    impl_number!(deserialize_i128, visit_i128, i128);
    impl_number!(deserialize_u8, visit_u8, u8);
    impl_number!(deserialize_u16, visit_u16, u16);
    impl_number!(deserialize_u32, visit_u32, u32);
    impl_number!(deserialize_u64, visit_u64, u64);
    impl_number!(deserialize_u128, visit_u128, u128);
    impl_number!(deserialize_f32, visit_f32, f32);
    impl_number!(deserialize_f64, visit_f64, f64);

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_char(
            self.take_next_as_str()?
                .chars()
                .next()
                .ok_or(PacketError::Incomplete)?,
        )
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.take_next_as_str()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_string(self.take_next_as_str()?.to_owned())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.take_next_part()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_byte_buf(self.take_next_part()?.to_vec())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let next = self.peek_next_part();
        if let Some(x) = next
            && *x == bstr::B(b"null")
        {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.take_next_part()?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(PacketDeserializerSeqAccess::new(self))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_enum(PacketDeserializerEnum::new(self))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use serde::Deserialize;

    use super::*;
    use crate::room::client::{direction::Direction, packet::IncomingPacket};
    #[test]
    fn test() {
        let data_and_deserialized = [
            (
                bstr::B(b"jmp\xff\xff20\xff\xff1"),
                IncomingPacket::Jump { x: 20, y: 1 },
            ),
            (
                bstr::B(b"tr\xff\xff50"),
                IncomingPacket::ChangeTransparency(50),
            ),
            (
                bstr::B(b"f\xff\xff0"),
                IncomingPacket::ChangeFacingDirection(Direction::Up),
            ),
        ];
        for (data, result) in &data_and_deserialized {
            let mut deserializer =
                PacketDeserializer::new(bstr::BStr::new(*data), b"\xFF\xFF".into());

            assert_eq!(
                IncomingPacket::deserialize(&mut deserializer).unwrap(),
                *result
            );
        }
    }
}
