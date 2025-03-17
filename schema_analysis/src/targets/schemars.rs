//! Integration with [schemars](https://github.com/GREsau/schemars)

use std::error::Error;

use schemars::schema as schemars_types;

use crate::{context::Context, Schema};

impl<C: Context> Schema<C> {
    /// Convert into a json_schema using the default settings.
    pub fn to_json_schema_with_schemars(&self) -> Result<String, impl Error> {
        self.to_json_schema_with_schemars_version(&Default::default())
    }

    /// Convert into a specific version of json_schema.
    pub fn to_json_schema_with_schemars_version(
        &self,
        version: &JsonSchemaVersion,
    ) -> Result<String, impl Error> {
        let settings: schemars::gen::SchemaSettings = version.to_schemars_settings();
        let mut generator: schemars::gen::SchemaGenerator = settings.into();

        let root = self.to_schemars_schema(&mut generator);
        serde_json::to_string_pretty(&root)
    }

    /// Convert using a provided generator (which also holds the settings) to a json schema.
    pub fn to_schemars_schema(
        &self,
        generator: &mut schemars::gen::SchemaGenerator,
    ) -> schemars_types::RootSchema {
        let inner = helpers::inferred_to_schemars(generator, self);
        helpers::wrap_in_root(inner, generator.settings())
    }
}

/// The currently supported json schema versions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JsonSchemaVersion {
    /// `schemars::gen::SchemaSettings::draft07`
    Draft07,
    /// `schemars::gen::SchemaSettings::draft2019_09`
    Draft2019_09,
    /// `schemars::gen::SchemaSettings::openapi3`
    OpenApi3,
}
impl Default for JsonSchemaVersion {
    fn default() -> Self {
        Self::Draft2019_09
    }
}
impl JsonSchemaVersion {
    /// Convert the version to full settings.
    pub fn to_schemars_settings(&self) -> schemars::gen::SchemaSettings {
        use schemars::gen::SchemaSettings;
        match self {
            JsonSchemaVersion::Draft07 => SchemaSettings::draft07(),
            JsonSchemaVersion::Draft2019_09 => SchemaSettings::draft2019_09(),
            JsonSchemaVersion::OpenApi3 => SchemaSettings::openapi3(),
        }
    }
}

mod helpers {

    use std::collections::BTreeSet;

    use schemars::schema as schemars_types;

    use crate::{context::Context, Field, Schema};

    /// Wraps a [Schema](schemars_types::Schema) in a [RootSchema](schemars_types::RootSchema).
    pub fn wrap_in_root(
        inner: schemars_types::Schema,
        settings: &schemars::gen::SchemaSettings,
    ) -> schemars_types::RootSchema {
        schemars_types::RootSchema {
            meta_schema: settings.meta_schema.clone(),
            definitions: Default::default(),
            schema: inner.into_object(),
        }
    }

    /// Converts an inferred [Schema] to a schemars [Schema](schemars_types::Schema).
    pub fn inferred_to_schemars<C: Context>(
        generator: &mut schemars::gen::SchemaGenerator,
        inferred: &Schema<C>,
    ) -> schemars_types::Schema {
        // Note: we can use the generator even if we don't generate the final root schema
        //  using it because simple values will not be referrenced.
        //  Do not use for complex values.
        match inferred {
            Schema::Null(_) => generator.subschema_for::<()>(),
            Schema::Boolean(_) => generator.subschema_for::<bool>(),

            // Using specific integer/float types causes the schema to remember the
            // specific representation.
            Schema::Integer(_) => schemars_types::SchemaObject {
                instance_type: Some(schemars_types::InstanceType::Integer.into()),
                ..Default::default()
            }
            .into(),
            Schema::Float(_) => schemars_types::SchemaObject {
                instance_type: Some(schemars_types::InstanceType::Number.into()),
                ..Default::default()
            }
            .into(),

            Schema::String(_) => generator.subschema_for::<String>(),
            Schema::Bytes(_) => generator.subschema_for::<Vec<u8>>(),

            Schema::Sequence { field, .. } => schemars_types::SchemaObject {
                instance_type: Some(schemars_types::InstanceType::Array.into()),
                array: Some(Box::new(schemars_types::ArrayValidation {
                    items: Some(internal_field_to_schemars_schema(generator, field).into()),
                    ..Default::default()
                })),
                ..Default::default()
            }
            .into(),

            Schema::Struct { fields, .. } => {
                let required: BTreeSet<String> = fields
                    .iter()
                    // Null values are handled in the Field function.
                    .filter(|(_, v)| !v.status.may_be_missing)
                    .map(|(k, _)| k.clone())
                    .collect();
                let properties = fields
                    .iter()
                    .map(|(k, field)| {
                        (
                            k.clone(),
                            internal_field_to_schemars_schema(generator, field),
                        )
                    })
                    .collect();
                schemars_types::SchemaObject {
                    instance_type: Some(schemars_types::InstanceType::Object.into()),
                    object: Some(Box::new(schemars_types::ObjectValidation {
                        required,
                        properties,
                        ..Default::default()
                    })),
                    ..Default::default()
                }
                .into()
            }

            Schema::Union { variants } => {
                let json_schemas = variants
                    .iter()
                    .map(|s| inferred_to_schemars(generator, s))
                    .collect();
                schemars_types::SchemaObject {
                    subschemas: Some(Box::new(schemars_types::SubschemaValidation {
                        any_of: Some(json_schemas),
                        ..Default::default()
                    })),
                    ..Default::default()
                }
                .into()
            }
        }
    }

    /// Converts a [Field] into a [Schema](schemars_types::Schema).
    fn internal_field_to_schemars_schema<C: Context>(
        generator: &mut schemars::gen::SchemaGenerator,
        field: &Field<C>,
    ) -> schemars_types::Schema {
        // Note: we can use the generator even if we don't generate the final root schema
        //  using it because simple values will not be referrenced.
        //  Do not use for complex values.

        let mut schema = match &field.schema {
            Some(schema) => inferred_to_schemars(generator, schema),
            None => schemars_types::Schema::Bool(true),
        };

        if field.status.may_be_null {
            // Taken from:
            // https://github.com/GREsau/schemars/blob/master/schemars/src/json_schema_impls/core.rs
            if generator.settings().option_add_null_type {
                schema = match schema {
                    schemars_types::Schema::Bool(true) => schemars_types::Schema::Bool(true),
                    schemars_types::Schema::Bool(false) => generator.subschema_for::<()>(),
                    schemars_types::Schema::Object(schemars_types::SchemaObject {
                        instance_type: Some(ref mut instance_type),
                        ..
                    }) => {
                        add_null_type(instance_type);
                        schema
                    }
                    schema => schemars_types::SchemaObject {
                        // TODO technically the schema already accepts null, so this may be unnecessary
                        subschemas: Some(Box::new(schemars_types::SubschemaValidation {
                            any_of: Some(vec![schema, generator.subschema_for::<()>()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }
                    .into(),
                }
            }
            if generator.settings().option_nullable {
                let mut schema_obj = schema.into_object();
                schema_obj
                    .extensions
                    .insert("nullable".to_owned(), serde_json::json!(true));
                schema = schemars_types::Schema::Object(schema_obj);
            };
        }
        schema
    }

    /// Taken from:
    /// https://github.com/GREsau/schemars/blob/master/schemars/src/json_schema_impls/core.rs
    fn add_null_type(
        instance_type: &mut schemars_types::SingleOrVec<schemars_types::InstanceType>,
    ) {
        match instance_type {
            schemars_types::SingleOrVec::Single(ty)
                if **ty != schemars_types::InstanceType::Null =>
            {
                *instance_type = vec![**ty, schemars_types::InstanceType::Null].into()
            }
            schemars_types::SingleOrVec::Vec(ty)
                if !ty.contains(&schemars_types::InstanceType::Null) =>
            {
                ty.push(schemars_types::InstanceType::Null)
            }
            _ => {}
        };
    }
}
