//! Integration with [schemars](https://github.com/GREsau/schemars)

use crate::{Schema, context::Context};

impl<C: Context> Schema<C> {
    /// Convert into a json_schema using the default settings.
    pub fn to_json_schema_with_schemars(&self) -> serde_json::Result<String> {
        let default = Default::default();
        self.to_json_schema_with_schemars_version(&default)
    }

    /// Convert into a specific version of json_schema.
    pub fn to_json_schema_with_schemars_version(
        &self,
        version: &JsonSchemaVersion,
    ) -> serde_json::Result<String> {
        let settings: schemars::generate::SchemaSettings = version.to_schemars_settings();
        let generator: schemars::generate::SchemaGenerator = settings.into();

        let root = self.to_schemars_schema(generator);
        serde_json::to_string_pretty(&root)
    }

    /// Convert using a provided generator (which also holds the settings) to a json schema.
    pub fn to_schemars_schema(
        &self,
        mut generator: schemars::generate::SchemaGenerator,
    ) -> schemars::Schema {
        let mut schema = helpers::inferred_to_schemars(&mut generator, self);
        if let Some(meta_schema) = generator.settings().meta_schema.as_deref() {
            schema.insert("$schema".to_owned(), meta_schema.into());
        }
        for transform in generator.transforms_mut() {
            transform.transform(&mut schema);
        }
        schema
    }
}

/// The currently supported json schema versions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum JsonSchemaVersion {
    /// [schemars::generate::SchemaSettings::draft07]
    Draft07,
    /// [schemars::generate::SchemaSettings::draft2019_09]
    #[default]
    Draft2019_09,
    /// [schemars::generate::SchemaSettings::draft2020_12]
    Draft2020_12,
    /// [schemars::generate::SchemaSettings::openapi3]
    OpenApi3,
}
impl JsonSchemaVersion {
    /// Convert the version to full settings.
    pub fn to_schemars_settings(&self) -> schemars::generate::SchemaSettings {
        use schemars::generate::SchemaSettings;
        match self {
            JsonSchemaVersion::Draft07 => SchemaSettings::draft07(),
            JsonSchemaVersion::Draft2019_09 => SchemaSettings::draft2019_09(),
            JsonSchemaVersion::Draft2020_12 => SchemaSettings::draft2020_12(),
            JsonSchemaVersion::OpenApi3 => SchemaSettings::openapi3(),
        }
    }
}

mod helpers {
    use ordermap::{OrderMap, OrderSet};
    use schemars::JsonSchema;
    use schemars::json_schema;
    use serde_json::Value;

    use crate::{Field, Schema, context::Context};

    /// Converts an inferred [Schema] to a schemars [Schema](schemars::Schema).
    pub fn inferred_to_schemars<C: Context>(
        generator: &mut schemars::generate::SchemaGenerator,
        inferred: &Schema<C>,
    ) -> schemars::Schema {
        // Note: we can use the generator even if we don't generate the final root schema
        //  using it because simple values will not be referrenced.
        //  Do not use for complex values.
        match inferred {
            Schema::Null(_) => generator.subschema_for::<()>(),
            Schema::Boolean(_) => generator.subschema_for::<bool>(),

            // Using specific integer/float types causes the schema to remember the
            // specific representation.
            Schema::Integer(_) => json_schema!({
                "type": "integer"
            }),

            Schema::Float(_) => json_schema!({
                "type": "number"
            }),

            Schema::String(_) => generator.subschema_for::<String>(),
            Schema::Bytes(_) => generator.subschema_for::<Vec<u8>>(),

            Schema::Sequence { field, .. } => schemars::json_schema!({
                "type": "array",
                "items": internal_field_to_schemars_schema(generator, field)
            }),

            Schema::Struct { fields, .. } => {
                let required: OrderSet<_> = fields
                    .iter()
                    // Null values are handled in the Field function.
                    .filter(|(_, v)| !v.status.may_be_missing)
                    .map(|(k, _)| k.clone())
                    .collect();
                let properties: OrderMap<_, _> = fields
                    .iter()
                    .map(|(k, field)| {
                        (
                            k.clone(),
                            internal_field_to_schemars_schema(generator, field),
                        )
                    })
                    .collect();

                let mut schema = json_schema!({ "type": "object" });
                if !properties.is_empty() {
                    schema.insert(
                        "properties".to_owned(),
                        serde_json::to_value(properties).unwrap(),
                    );
                }
                if !required.is_empty() {
                    schema.insert(
                        "required".to_owned(),
                        serde_json::to_value(required).unwrap(),
                    );
                }
                schema
            }

            Schema::Union { variants } => {
                let json_schemas: Vec<_> = variants
                    .iter()
                    .map(|s| inferred_to_schemars(generator, s))
                    .collect();

                json_schema!({
                    "anyOf": json_schemas,
                })
            }
        }
    }

    /// Converts a [Field] into a [Schema](schemars::Schema).
    fn internal_field_to_schemars_schema<C: Context>(
        generator: &mut schemars::generate::SchemaGenerator,
        field: &Field<C>,
    ) -> schemars::Schema {
        // Note: we can use the generator even if we don't generate the final root schema
        //  using it because simple values will not be referrenced.
        //  Do not use for complex values.

        let mut schema = match &field.schema {
            Some(schema) => inferred_to_schemars(generator, schema),
            None => schemars::Schema::from(true),
        };

        if field.status.may_be_null {
            allow_null(generator, &mut schema);
        }
        schema
    }

    /// Taken from:
    /// https://github.com/GREsau/schemars/blob/master/schemars/src/json_schema_impls/core.rs
    /// https://github.com/GREsau/schemars/blob/master/schemars/src/_private/mod.rs
    /// Alt hash: e67495be31e784d32f3d3310edb925458b0f2574
    #[expect(clippy::collapsible_if)] // Min diff
    fn allow_null(
        generator: &mut schemars::generate::SchemaGenerator,
        schema: &mut schemars::Schema,
    ) {
        fn is_null_schema(value: &Value) -> bool {
            <&schemars::Schema>::try_from(value).is_ok_and(|s| has_type(s.as_value(), "null"))
        }

        match (schema.as_bool(), schema.as_object_mut()) {
            (None, Some(obj)) => {
                if obj.len() == 1
                    && obj
                        .get("anyOf")
                        .and_then(Value::as_array)
                        .is_some_and(|a| a.iter().any(is_null_schema))
                {
                    return;
                }

                if contains_immediate_subschema(obj) {
                    *schema = json_schema!({
                        "anyOf": [
                            obj,
                            <()>::json_schema(generator)
                        ]
                    });
                    // No need to check `type`/`const`/`enum` because they're trivially not present
                    return;
                }

                if let Some(instance_type) = obj.get_mut("type") {
                    match instance_type {
                        Value::Array(array) => {
                            let null = Value::from("null");
                            if !array.contains(&null) {
                                array.push(null);
                            }
                        }
                        Value::String(string) => {
                            if string != "null" {
                                let current_type = core::mem::take(string).into();
                                *instance_type = Value::Array(vec![current_type, "null".into()]);
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(c) = obj.remove("const") {
                    if !c.is_null() {
                        obj.insert("enum".to_string(), Value::Array(vec![c, Value::Null]));
                    }
                } else if let Some(Value::Array(e)) = obj.get_mut("enum") {
                    if !e.contains(&Value::Null) {
                        e.push(Value::Null);
                    }
                }
            }
            (Some(true), None) => {}
            (Some(false), None) => {
                *schema = <()>::json_schema(generator);
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn has_type(value: &Value, ty: &str) -> bool {
        match value.get("type") {
            Some(Value::Array(values)) => values.iter().any(|v| v.as_str() == Some(ty)),
            Some(Value::String(s)) => s == ty,
            _ => false,
        }
    }

    fn contains_immediate_subschema(schema_obj: &serde_json::Map<String, Value>) -> bool {
        ["if", "allOf", "anyOf", "oneOf", "$ref"]
            .into_iter()
            .any(|k| schema_obj.contains_key(k))
    }
}
