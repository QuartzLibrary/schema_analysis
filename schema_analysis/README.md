# schema_analysis

## Universal-ish Schema Analysis

Ever wished you could figure out what was in that json file? Or maybe it was xml... Ehr, yaml?
It was definitely toml.

Alas, many great tools will only work with one of those formats, and the internet is not so
nice a place as to finally understand that no, xml is not an acceptable document format.

Enter this neat little tool, a single interface to any self-describing format supported by
our gymmnast friend, serde.

### Features

- Works with any self-describing format with a Serde implementation.
- Suitable for large files.
- Keeps track of some useful info for each type.
- Keeps track of null/normal/missing/duplicate values separately.
- Integrates with [Schemars](https://github.com/GREsau/schemars) and 
  [json_typegen](https://github.com/evestera/json_typegen) to produce types and json schema if needed.
- There's a demo website [here](https://schema-analysis.com/).

### Usage

```rust
let data: &[u8] = "true".as_bytes();

// Just pick your format, and deserialize InferredSchema as if it were a normal type.
let inferred: InferredSchema = serde_json::from_slice(data)?;
// let inferred: InferredSchema = serde_yaml::from_slice(data)?;
// let inferred: InferredSchema = serde_cbor::from_slice(data)?;
// let inferred: InferredSchema = toml::from_slice(data)?;
// let inferred: InferredSchema = rawbson::de::from_bytes(data)?;
// let inferred: InferredSchema = quick_xml::de::from_reader(data)?;

// InferredSchema is a wrapper around Schema
let schema: Schema = inferred.schema;
let expected: Schema = Schema::Boolean(Default::default());
assert!(schema.structural_eq(&expected));

// The wrapper is there so we can both do the magic above, and also store the data for later
let serialized_schema: String = serde_json::to_string_pretty(&schema)?;
```

That's it.

Check [Schema](src/analysis/schema.rs) to see what info you get, 
and [targets](src/targets) to see the available integrations (which include code and 
json schema generation).

### Advanced Usage

I know, I know, the internet is evil and has decided to plague you with not one, but thousands,
maybe even millions, of files.

Unfortunately Serde relies on type information to work, so ~~there is nothing we can do about it~~
we can bring out the big guns: [DeserializeSeed](https://docs.serde.rs/serde/de/trait.DeserializeSeed.html).
It's everything you love about Serde, but with runtime state.

```rust
let a_lot_of_files: &[&[u8]] = &[ "1".as_bytes(), "2".as_bytes(), "1000".as_bytes() ];
let mut iter = a_lot_of_files.iter();

if let Some(file) = iter.next() {
    let mut inferred: InferredSchema = serde_json::from_slice(file)?;
    for file in iter {
        let mut json_deserializer = serde_json::Deserializer::from_slice(file);
        // DeserializeSeed is implemented on &mut InferredSchema
        // So here it borrows the data mutably and runs it against the deserializer.
        let () = inferred.deserialize(&mut json_deserializer)?;
    }
    let mut context: NumberContext<i128> = Default::default();
    context.aggregate(&1);
    context.aggregate(&2);
    context.aggregate(&1000);

    let expected: Schema = Schema::Integer(context);
    assert_eq!(inferred.schema, expected);
}
```

Furthermore, if you need to generate separate schemas (for example to run the analysis on multiple
threads) you can use the Coalesce trait to merge them after-the-fact.

### I really wish I could convert that Schema in something, you know, actually useful.

You are in luck! You can check out [here](src/targets) the integrations with
[json_typegen](https://github.com/evestera/json_typegen) and [Schemars](https://github.com/GREsau/schemars) 
to convert the analysis into useful files like Rust types and json schemas.
You can also find a demo website [here](https://schema-analysis.com/).

### How does this work?

For a the short story long go [here](src/analysis/mod.rs), the juicy bit is that Serde is kind enough to let
the format tell us what it is working with, we take it from there and construct a nice schema
from that info.