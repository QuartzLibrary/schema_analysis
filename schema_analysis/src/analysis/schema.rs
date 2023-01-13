use std::collections::BTreeMap;

use serde::de::{Error, Visitor};

use crate::{Aggregate, Field, Schema};

use super::{
    field::{FieldVisitor, FieldVisitorSeed},
    Context,
};

pub struct SchemaVisitor<'s> {
    pub context: &'s Context,
}

impl<'de, 's> Visitor<'de> for SchemaVisitor<'s> {
    type Value = Schema;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("anything")
    }

    fn visit_bool<E: Error>(self, value: bool) -> Result<Self::Value, E> {
        let mut aggregators = self.context.for_boolean();
        aggregators.aggregate(&value);

        Ok(Schema::Boolean(aggregators))
    }
    fn visit_i128<E: Error>(self, value: i128) -> Result<Self::Value, E> {
        let mut aggregators = self.context.for_integer();
        aggregators.aggregate(&value);

        Ok(Schema::Integer(aggregators))
    }
    fn visit_f64<E: Error>(self, value: f64) -> Result<Self::Value, E> {
        let mut aggregators = self.context.for_float();
        aggregators.aggregate(&value);

        Ok(Schema::Float(aggregators))
    }
    fn visit_borrowed_str<E: Error>(self, value: &'de str) -> Result<Self::Value, E> {
        let mut aggregators = self.context.for_string();
        aggregators.aggregate(value);

        Ok(Schema::String(aggregators))
    }
    fn visit_borrowed_bytes<E: Error>(self, value: &'de [u8]) -> Result<Self::Value, E> {
        let mut aggregators = self.context.for_bytes();
        aggregators.aggregate(value);

        Ok(Schema::Bytes(aggregators))
    }

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

    /// This method should only be called if the Null value is at the root of the document,
    /// because otherwise null values are handled at the field level.
    fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
        let mut aggregators = self.context.for_null();
        aggregators.aggregate(&());

        Ok(Schema::Null(aggregators))
    }
    /// Some & None are handled at the field level, with the exception of the root where the
    /// schema itself might be Null in some formats.
    ///
    /// Since the formats I am currently aware do not support explicitly marking a field as
    /// optional (which would mean possibly calling visit_some at the root level),
    /// I am marking this as unreachable!().
    fn visit_some<D>(self, _deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // self.deserialize(deserializer)
        unreachable!()
    }
    /// serde_json calls this method for `null`, so we assume `visit_unit == visit_none`.
    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        self.visit_none()
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut count = 0;

        let initial_seed = FieldVisitor {
            context: self.context,
        };

        let mut field = match seq.next_element_seed(initial_seed)? {
            Some(mut field) => {
                count += 1;

                while let Some(()) = seq.next_element_seed(FieldVisitorSeed {
                    context: self.context,
                    field: &mut field,
                })? {
                    count += 1;
                }

                field
            }
            // If the sequence is empty, just create an empty field with no inner schema.
            None => Field::default(),
        };

        if count == 0 {
            field.status.may_be_missing = true;
        }

        let mut aggregators = self.context.for_sequence();
        aggregators.aggregate(&count);

        Ok(Schema::Sequence {
            field: Box::new(field),
            context: aggregators,
        })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut keys = Vec::new();
        let mut fields: BTreeMap<String, Field> = BTreeMap::new();

        while let Some(key) = map.next_key::<String>()? {
            match fields.get_mut(&key) {
                Some(old_field) => {
                    map.next_value_seed(FieldVisitorSeed {
                        context: self.context,
                        field: old_field,
                    })?;
                    old_field.status.allow_duplicates(true);
                }

                None => {
                    let new_field = map.next_value_seed(FieldVisitor {
                        context: self.context,
                    })?;
                    fields.insert(key.clone(), new_field);
                }
            }

            keys.push(key.clone());
        }

        let mut aggregators = self.context.for_map_struct();
        aggregators.aggregate(&keys);

        Ok(Schema::Struct {
            fields,
            context: aggregators,
        })
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
