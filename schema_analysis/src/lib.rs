#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc(html_root_url = "https://docs.rs/schema_analysis/0.3.4/")]
/*!
# Universal-ish Schema Analysis

[![](https://meritbadge.herokuapp.com/schema_analysis)](https://crates.io/crates/schema_analysis)
[![](https://docs.rs/schema_analysis/badge.svg)](https://docs.rs/schema_analysis/)

Ever wished you could figure out what was in that json file? Or maybe it was xml... Ehr, yaml?
It was definitely toml.

Alas, many great tools will only work with one of those formats, and the internet is not so
nice a place as to finally understand that no, xml is not an acceptable document format.

Enter this neat little tool, a single interface to any self-describing format supported by
our gymnast friend, [serde].

## Features

- Works with any self-describing format with a Serde implementation.
- Suitable for large files[^1].
- Keeps track of some useful info for each type.
- Keeps track of null/normal/missing/duplicate values separately.
- Integrates with [Schemars](schemars) and [json_typegen](https://github.com/evestera/json_typegen) to produce types and json schema if needed.
- There's a demo website [here](https://schema-analysis.com/).

[^1]: This is just a weirdly shaped parser, so values are discarded as soon as they have been analyzed.
This should hopefully translate in memory requirements that scale with the size of [Schema], not
the input data. If you ever had a schema analysis tool break on you after a mere million documents,
this'll probably be appreciated.

## Usage

```
# use schema_analysis::{Schema, InferredSchema, StructuralEq};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let data: &[u8] = b"true";

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
# Ok(())
# }
```

That's it.

Check [Schema] to see what info you get, and [targets] to see the available integrations (which
include code and json schema generation).

## Advanced Usage

I know, I know, the internet is evil and has decided to plague you with not one, but thousands,
maybe even millions, of files.

Unfortunately Serde relies on type information to work, so ~~there is nothing we can do about it~~
we can bring out the big guns: [DeserializeSeed](serde::de::DeserializeSeed).
It's everything you love about Serde, but with runtime state.

```
# use serde::de::DeserializeSeed;
# use schema_analysis::{Schema, InferredSchema, context::NumberContext, Aggregate};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let a_lot_of_json_files: &[&str] = &[ "1", "2", "1000" ];
let mut iter = a_lot_of_json_files.iter();

if let Some(file) = iter.next() {
    // We use the first file to generate a new schema to work with.
    let mut inferred: InferredSchema = serde_json::from_str(file)?;

    // Then we iterate over the rest to expand the schema.
    for file in iter {
        let mut json_deserializer = serde_json::Deserializer::from_str(file);
        // DeserializeSeed is implemented on &mut InferredSchema
        // So here it borrows the data mutably and runs it against the deserializer.
        let () = inferred.deserialize(&mut json_deserializer)?;
    }

    // The result in this case would be a simple integer schema
    // that 'has met' the numbers 1, 2, and 100.
    let mut context: NumberContext<i128> = Default::default();
    context.aggregate(&1);
    context.aggregate(&2);
    context.aggregate(&1000);

    assert_eq!(inferred.schema, Schema::Integer(context));
}
# Ok(())
# }
```

Furthermore, if you need to generate separate schemas (for example to run the analysis on multiple
threads) you can use the [Coalesce] trait to merge them after-the-fact.

## I really wish I could convert that [Schema] in something, you know, actually useful.

You are in luck! You can check out [here](targets) the integrations with
[json_typegen](json_typegen_shared) and [Schemars](schemars) to convert the analysis into useful
files like Rust types and json schemas.
You can also find a demo website [here](https://schema-analysis.com/).

## How does this work?

For a the short story long go [here](analysis), the juicy bit is that Serde is kind enough to let
the format tell us what it is working with, we take it from there and construct a nice schema
from that info.

### Performance

> These are not proper benchmarks, but should give a vague idea of the performance on a 3 years old i7 laptop with the raw data already loaded into memory.

| Size                  | wasm (MB/s)  | native (MB/s) | Format | File # |
| --------------------- | ------------ | ------------- | ------ | ------ |
| [~180MB]              | ~20s (9)     | ~5s (36)      | json   | 1      |
| [~650MB]              | ~150s (4.3)  | ~50s (13)     | json   | 1      |
| [~1.7GB]              | ~470s (3.6)  | ~145s (11.7)  | json   | 1      |
| [~2.1GB]              | <sup>a</sup> | ~182s (11.5)  | json   | 1      |
| [~13.3GB]<sup>b</sup> |              | ~810s (16.4)  | xml    | ~200k  |

<sup>a</sup> This one seems to go over some kind of browser limit when fetching the data in the Web Worker, I believe I would have to split large files to handle it.

<sup>b</sup> ~2.7GB compressed. This one seems like it would be a worst-case scenario because it includes decompression overhead and the files had a section that was formatted text which resulted in crazy schemas. (The json pretty printed schema was almost 0.5GB!)


[~180MB]: https://github.com/zemirco/sf-city-lots-json/blob/master/citylots.json
[~650MB]: https://catalog.data.gov/dataset/forestry-planting-spaces
[~1.7GB]: https://catalog.data.gov/dataset/nys-thruway-origin-and-destination-points-for-all-vehicles-15-minute-intervals-2018-q4
[~2.1GB]: https://catalog.data.gov/dataset/turnstile-usage-data-2016
[~13.3GB]: https://ftp.ncbi.nlm.nih.gov/pub/pmc/oa_bulk/
*/

mod schema;

use traits::CoalescingAggregator;

pub mod analysis;
pub mod context;
pub mod helpers;
pub mod targets;
pub mod traits;

pub use analysis::{InferredSchema, InferredSchemaWithContext};
pub use context::{Aggregators, Context};
pub use schema::{Field, FieldStatus, Schema};
pub use traits::{Aggregate, Coalesce, StructuralEq};
