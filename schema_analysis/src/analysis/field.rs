use serde::de::{DeserializeSeed, Error, Visitor};

use crate::Field;

use super::{schema::SchemaVisitor, schema_seed::SchemaVisitorSeed, Context};

pub struct FieldVisitor<'s> {
    pub context: &'s Context,
}

impl<'de, 's> DeserializeSeed<'de> for FieldVisitor<'s> {
    type Value = Field;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut field = Field::default();
        let () = deserializer.deserialize_any(FieldVisitorSeed {
            context: self.context,
            field: &mut field,
        })?;

        Ok(field)
    }
}

pub struct FieldVisitorSeed<'s> {
    pub context: &'s Context,
    pub field: &'s mut Field,
}

impl<'de, 's> DeserializeSeed<'de> for FieldVisitorSeed<'s> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

macro_rules! method_impl {
    ($method_name:ident, $type:ty) => {
        fn $method_name<E: Error>(self, value: $type) -> Result<Self::Value, E> {
            match &mut self.field.schema {
                // If a schema is already present, then we can use it as seed and let
                // the schema side of things take care of the rest.
                Some(schema) => {
                    let () = SchemaVisitorSeed {
                        context: self.context,
                        schema,
                    }
                    .$method_name(value)?;
                }
                // Otherwise we need to generate a new schema.
                None => {
                    let schema = SchemaVisitor {
                        context: self.context,
                    }
                    .$method_name(value)?;
                    self.field.schema = Some(schema);
                }
            }
            // Since we have visited a value with this field,
            // we mark it to remember that a non-null value was found.
            self.field.status.may_be_normal = true;
            Ok(())
        }
    };
}

impl<'de, 's> Visitor<'de> for FieldVisitorSeed<'s> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("anything")
    }

    method_impl!(visit_bool, bool);
    method_impl!(visit_i128, i128);
    method_impl!(visit_f64, f64);
    method_impl!(visit_borrowed_str, &str);
    method_impl!(visit_borrowed_bytes, &[u8]);

    fn visit_i8<E: Error>(self, value: i8) -> Result<Self::Value, E> {
        self.visit_i128(value.into())
    }
    fn visit_i16<E: Error>(self, value: i16) -> Result<Self::Value, E> {
        self.visit_i128(value.into())
    }
    fn visit_i32<E: Error>(self, value: i32) -> Result<Self::Value, E> {
        self.visit_i128(value.into())
    }
    fn visit_i64<E: Error>(self, value: i64) -> Result<Self::Value, E> {
        self.visit_i128(value.into())
    }
    fn visit_u8<E: Error>(self, value: u8) -> Result<Self::Value, E> {
        self.visit_i128(value.into())
    }
    fn visit_u16<E: Error>(self, value: u16) -> Result<Self::Value, E> {
        self.visit_i128(value.into())
    }
    fn visit_u32<E: Error>(self, value: u32) -> Result<Self::Value, E> {
        self.visit_i128(value.into())
    }
    fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
        self.visit_i128(value.into())
    }
    fn visit_u128<E: Error>(self, value: u128) -> Result<Self::Value, E> {
        let as_i128 = std::convert::TryInto::try_into(value)
            .map_err(|_| E::custom("u128 value too large to fit into a i138"))?;
        self.visit_i128(as_i128)
    }

    fn visit_f32<E: Error>(self, value: f32) -> Result<Self::Value, E> {
        self.visit_f64(value.into())
    }

    fn visit_char<E: Error>(self, value: char) -> Result<Self::Value, E> {
        self.visit_string(value.into())
    }
    fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
        self.visit_borrowed_str(value)
    }
    fn visit_string<E: Error>(self, value: String) -> Result<Self::Value, E> {
        self.visit_borrowed_str(&value)
    }

    fn visit_bytes<E: Error>(self, value: &[u8]) -> Result<Self::Value, E> {
        self.visit_borrowed_bytes(value)
    }
    fn visit_byte_buf<E: Error>(self, value: Vec<u8>) -> Result<Self::Value, E> {
        self.visit_borrowed_bytes(&value)
    }

    /// If a field is null, then we simply mark it as such and move on.
    /// The schema is left untouched.
    fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
        self.field.status.may_be_null = true;
        Ok(())
    }
    /// If a field is marked as being 'some', then we assume it means that it may be null
    /// and mark it as such.
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.field.status.may_be_null = true;
        let () = self.deserialize(deserializer)?;
        Ok(())
    }
    /// serde_json calls this method for `null`, so we assume `visit_unit == visit_none`.
    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        self.visit_none()
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        match &mut self.field.schema {
            Some(schema) => {
                let () = SchemaVisitorSeed {
                    context: self.context,
                    schema,
                }
                .visit_seq(seq)?;
            }
            None => {
                let schema = SchemaVisitor {
                    context: self.context,
                }
                .visit_seq(seq)?;
                self.field.schema = Some(schema);
            }
        }
        self.field.status.may_be_normal = true;
        Ok(())
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        match &mut self.field.schema {
            Some(schema) => {
                let () = SchemaVisitorSeed {
                    context: self.context,
                    schema,
                }
                .visit_map(map)?;
            }
            None => {
                let schema = SchemaVisitor {
                    context: self.context,
                }
                .visit_map(map)?;
                self.field.schema = Some(schema);
            }
        }
        self.field.status.may_be_normal = true;
        Ok(())
    }

    fn visit_newtype_struct<D>(self, _deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unreachable!("newtype structs are a rust construct")
    }

    fn visit_enum<A>(self, _data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        unreachable!("enum types are usually not available from the format's side")
    }
}
