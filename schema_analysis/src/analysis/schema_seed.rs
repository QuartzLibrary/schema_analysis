use serde::de::{Error, Visitor};

use crate::{traits::Coalesce, Aggregate, Schema};

use super::{
    field::{FieldVisitor, FieldVisitorSeed},
    schema::SchemaVisitor,
    Context,
};

pub struct SchemaVisitorSeed<'s> {
    pub context: &'s Context,
    pub schema: &'s mut Schema,
}

impl<'de, 's> Visitor<'de> for SchemaVisitorSeed<'s> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("anything")
    }

    fn visit_bool<E: Error>(mut self, value: bool) -> Result<Self::Value, E> {
        match &mut self.schema {
            // The schema matches
            Schema::Boolean(aggregators) => aggregators.aggregate(&value),
            // Extend a different schema
            schema => {
                let new_schema = SchemaVisitor {
                    context: self.context,
                }
                .visit_bool(value)?;

                schema.coalesce(new_schema);
            }
        }
        Ok(())
    }
    fn visit_i128<E: Error>(mut self, value: i128) -> Result<Self::Value, E> {
        match &mut self.schema {
            // The schema matches
            Schema::Integer(aggregators) => aggregators.aggregate(&value),
            // Extend a different schema
            schema => {
                let new_schema = SchemaVisitor {
                    context: self.context,
                }
                .visit_i128(value)?;

                schema.coalesce(new_schema);
            }
        }
        Ok(())
    }
    fn visit_f64<E: Error>(mut self, value: f64) -> Result<Self::Value, E> {
        match &mut self.schema {
            // The schema matches
            Schema::Float(aggregators) => aggregators.aggregate(&value),
            // Extend a different schema
            schema => {
                let new_schema = SchemaVisitor {
                    context: self.context,
                }
                .visit_f64(value)?;

                schema.coalesce(new_schema);
            }
        }
        Ok(())
    }
    fn visit_borrowed_str<E: Error>(mut self, value: &'de str) -> Result<Self::Value, E> {
        match &mut self.schema {
            // The schema matches
            Schema::String(aggregators) => aggregators.aggregate(value),
            // Extend a different schema
            schema => {
                let new_schema = SchemaVisitor {
                    context: self.context,
                }
                .visit_borrowed_str(value)?;

                schema.coalesce(new_schema);
            }
        }
        Ok(())
    }
    fn visit_borrowed_bytes<E: Error>(mut self, value: &'de [u8]) -> Result<Self::Value, E> {
        match &mut self.schema {
            // The schema matches
            Schema::Bytes(aggregators) => aggregators.aggregate(value),
            // Extend a different schema
            schema => {
                let new_schema = SchemaVisitor {
                    context: self.context,
                }
                .visit_borrowed_bytes(value)?;

                schema.coalesce(new_schema);
            }
        }
        Ok(())
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
    /// because otherwise null values are handled by `Field`.
    fn visit_none<E: Error>(mut self) -> Result<Self::Value, E> {
        match &mut self.schema {
            // The schema matches
            Schema::Null(aggregators) => {
                aggregators.aggregate(&());
            }
            // Extend a different schema
            schema => {
                let new_schema = SchemaVisitor {
                    context: self.context,
                }
                .visit_none()?;

                schema.coalesce(new_schema);
            }
        }
        Ok(())
    }
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let () = deserializer.deserialize_any(self)?;
        Ok(())
    }
    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        // serde_json calls this method for `null`.
        self.visit_none()
    }

    fn visit_newtype_struct<D>(self, _deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unreachable!("newtype structs are a rust construct")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut count = 0;
        match &mut self.schema {
            // The schema matches
            Schema::Sequence {
                field: ref mut boxed_field,
                context: ref mut aggregators,
            } => {
                let field = boxed_field.as_mut();

                while let Some(()) = seq.next_element_seed(FieldVisitorSeed {
                    context: self.context,
                    field,
                })? {
                    count += 1;
                }

                if count == 0 {
                    field.status.may_be_missing = true;
                }

                aggregators.aggregate(&count);
            }
            // Extend a different schema
            schema => {
                let sequence_schema = SchemaVisitor {
                    context: self.context,
                }
                .visit_seq(seq)?;
                schema.coalesce(sequence_schema);
            }
        };
        Ok(())
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut keys = Vec::new();
        match &mut self.schema {
            Schema::Struct {
                fields,
                context: aggregators,
            } => {
                while let Some(key) = map.next_key::<String>()? {
                    match fields.get_mut(&key) {
                        Some(old_field) => {
                            old_field.status.allow_duplicates(keys.contains(&key));
                            let () = map.next_value_seed(FieldVisitorSeed {
                                context: self.context,
                                field: old_field,
                            })?;
                        }

                        None => {
                            let mut new_field = map.next_value_seed(FieldVisitor {
                                context: self.context,
                            })?;
                            // If we are adding it to an existing schema it means that it was
                            // missing when this schema was created.
                            new_field.status.may_be_missing = true;
                            new_field.status.allow_duplicates(keys.contains(&key));
                            fields.insert(key.clone(), new_field);
                        }
                    }

                    keys.push(key);
                }

                for (k, f) in fields {
                    if !keys.contains(k) {
                        f.status.may_be_missing = true;
                    }
                }

                aggregators.aggregate(&keys);
            }
            schema => {
                let sequence_schema = SchemaVisitor {
                    context: self.context,
                }
                .visit_map(map)?;
                schema.coalesce(sequence_schema);
            }
        }
        Ok(())
    }

    fn visit_enum<A>(self, _data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        unreachable!("enum types are usually not available from the format's side")
    }
}
