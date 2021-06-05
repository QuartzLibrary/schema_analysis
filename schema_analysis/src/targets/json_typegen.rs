//! Integration with [json_typegen](https://github.com/evestera/json_typegen)
//!
//! Currently re-exports a couple private modules from a fork and has a copied-and-pasted helper
//! function below.
//!
//! FIXME: ping json_typegen author.

use std::error::Error;

pub use json_typegen_shared::{
    generation,
    options::{Options, OutputMode, StringTransform},
    shape::Shape,
    ErrorKind, JTError,
};

use crate::{Field, Schema};

impl Schema {
    /// Convert a [Schema] to a json_typegen [Shape].
    pub fn to_json_typegen_shape(&self) -> Shape {
        schema_to_shape(self)
    }

    /// Convert a [Schema] to a supported json_typegen output
    pub fn process_with_json_typegen(&self, mode: OutputMode) -> Result<String, impl Error> {
        let mut options = Options::default();
        options.output_mode = mode;
        self.process_with_json_typegen_options("Root", &options)
    }

    /// Convert a [Schema] to a supported json_typegen output using custom settings.
    pub fn process_with_json_typegen_options(
        &self,
        name: &str,
        options: &Options,
    ) -> Result<String, impl Error> {
        let shape = self.to_json_typegen_shape();
        process_json_typegen_shape(name, &shape, &options)
    }
}

/// Convert a json_typegen_shared [Shape] to a supported json_typegen output.
///
/// This helper function adapts json_typegen code to work directly on the [Shape]
/// (instead of deriving the [Shape] and processing it in one go).
pub fn process_json_typegen_shape(
    name: &str,
    shape: &Shape,
    options: &Options,
) -> Result<String, JTError> {
    let options = options.clone();

    // Taken from:
    // https://github.com/evestera/json_typegen/blob/HEAD/json_typegen_shared/src/lib.rs

    let mut generated_code = if options.runnable {
        generation::rust::rust_program(name, &shape, options)
    } else {
        let (name, defs) = match options.output_mode {
            OutputMode::Rust => generation::rust::rust_types(name, &shape, options),
            OutputMode::JsonSchema => generation::json_schema::json_schema(name, &shape, options),
            OutputMode::KotlinJackson | OutputMode::KotlinKotlinx => {
                generation::kotlin::kotlin_types(name, &shape, options)
            }
            OutputMode::Shape => generation::shape::shape_string(name, &shape, options),
            OutputMode::Typescript => {
                generation::typescript::typescript_types(name, &shape, options)
            }
            OutputMode::TypescriptTypeAlias => {
                generation::typescript_type_alias::typescript_type_alias(name, &shape, options)
            }
        };
        defs.ok_or_else(|| JTError::from(ErrorKind::ExistingType(name.to_string())))?
    };

    // Ensure generated code ends with exactly one newline
    generated_code.truncate(generated_code.trim_end().len());
    generated_code.push('\n');

    Ok(generated_code)
}

impl From<Schema> for Shape {
    fn from(schema: Schema) -> Self {
        schema_to_shape(&schema)
    }
}

fn schema_to_shape(schema: &Schema) -> Shape {
    match schema {
        Schema::Null(_) => Shape::Null,
        Schema::Boolean(_) => Shape::Bool,
        Schema::Integer(_) => Shape::Integer,
        Schema::Float(_) => Shape::Floating,
        Schema::String(_) => Shape::StringT,
        Schema::Bytes(_) => Shape::Any,
        Schema::Sequence { field, .. } => Shape::VecT {
            elem_type: Box::new(convert_field(field.as_ref())),
        },
        Schema::Struct { fields, .. } => Shape::Struct {
            fields: fields
                .iter()
                .map(|(name, value)| (name.clone(), convert_field(&value)))
                .collect(),
        },
        // From Shape docs:
        // `Any` represents conflicting inference information that can not be represented by any
        //   single shape
        Schema::Union { .. } => Shape::Any,
    }
}

fn convert_field(field: &Field) -> Shape {
    // From Shape docs:
    // `Bottom` represents the absence of any inference information
    // `Optional(T)` represents that a value is nullable, or not always present
    // `Null` represents optionality with no further information. [Equivalent to `Optional(Bottom)`]

    // `Bottom` would be equivalent to a field with a `None` schema.
    // `Optional(T)` would be equivalent to a field marked as possibly missing or possibly null.
    // `Null` would be equivalent to a field that is both missing/null and has no schema.

    let is_option = field.status.is_option();
    match &field.schema {
        Some(s) if is_option => Shape::Optional(Box::new(schema_to_shape(s))),
        Some(s) => schema_to_shape(s),
        None if is_option => Shape::Null,
        None => Shape::Bottom,
    }
}
