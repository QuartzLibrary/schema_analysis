/*!
Integration with [json_typegen](https://github.com/evestera/json_typegen)

You can:
```rust
# use schema_analysis::Schema;
# use schema_analysis::targets::json_typegen::{Shape, OutputMode, Options};
#
# let schema: Schema = Schema::Boolean(Default::default());
#
// Convert to a json_typegen Shape.
let shape: Shape = schema.to_json_typegen_shape();

// Convert to a specific json_typegen output with default options.
let output: String = schema.process_with_json_typegen(OutputMode::Rust).unwrap();

// Convert a json_typegen Shape with custom options.
let output: String = json_typegen_shared::codegen_from_shape("Root", &Shape::Bool, Options::default()).unwrap();
```
*/

pub use json_typegen_shared::{codegen_from_shape, ErrorKind, JTError, Options, OutputMode, Shape};

use crate::{Field, Schema};

impl Schema {
    /// Convert a [Schema] to a json_typegen [Shape].
    pub fn to_json_typegen_shape(&self) -> Shape {
        schema_to_shape(self)
    }

    /// Convert a [Schema] to a supported json_typegen output
    pub fn process_with_json_typegen(&self, mode: OutputMode) -> Result<String, JTError> {
        let mut options = Options::default();
        options.output_mode = mode;
        self.process_with_json_typegen_options("Root", &options)
    }

    /// Convert a [Schema] to a supported json_typegen output using custom settings.
    pub fn process_with_json_typegen_options(
        &self,
        name: &str,
        options: &Options,
    ) -> Result<String, JTError> {
        let shape = self.to_json_typegen_shape();
        codegen_from_shape(name, &shape, options.clone())
    }
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
            elem_type: Box::new(convert_field(field.as_ref(), field.status.may_be_null)),
        },
        Schema::Struct { fields, .. } => Shape::Struct {
            fields: fields
                .iter()
                .map(|(name, field)| (name.clone(), convert_field(field, field.status.is_option())))
                .collect(),
        },
        // From Shape docs:
        // `Any` represents conflicting inference information that can not be represented by any
        //   single shape
        Schema::Union { .. } => Shape::Any,
    }
}

/// This function also takes `is_option` because fields in structs are considered 'optional' also
/// if they are missing, while sequences whose fields may be missing are merely empty.
///
/// In both cases the field is optional if it may have a value of null/none.
fn convert_field(field: &Field, is_option: bool) -> Shape {
    // From Shape docs:
    // `Bottom` represents the absence of any inference information
    // `Optional(T)` represents that a value is nullable, or not always present
    // `Null` represents optionality with no further information. [Equivalent to `Optional(Bottom)`]

    // So:
    // `Bottom` would be equivalent to a field with a `None` schema.
    // `Optional(T)` would be equivalent to a field marked as possibly missing or possibly null.
    // `Null` would be equivalent to a field that is both missing/null and has no schema.

    match &field.schema {
        Some(s) if is_option => Shape::Optional(Box::new(schema_to_shape(s))),
        Some(s) => schema_to_shape(s),
        None if is_option => Shape::Null,
        None => Shape::Bottom,
    }
}
