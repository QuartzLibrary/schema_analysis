---
name: schema-analysis
description: >
  Infer schemas and generate types (Rust, TypeScript, Kotlin, JSON Schema) from
  data files in JSON, YAML, XML, TOML, CBOR, or BSON. Use when you need to
  understand the structure of a data file, generate type definitions for code,
  produce a JSON Schema, or compare the shapes of multiple data files.
license: MIT
compatibility: Requires npx (Node.js 18+), or uvx (Python 3.8+), or cargo
metadata:
  author: QuartzLibrary
  version: "0.7.0"
---

# schema_analysis

Infer schemas from data files and generate typed code or JSON Schema.

## Running

```bash
npx schema_analysis [OPTIONS] [FILES]...
# or
uvx schema_analysis [OPTIONS] [FILES]...
```

Input format is auto-detected from file extensions (`.json`, `.yaml`/`.yml`, `.xml`, `.toml`, `.cbor`, `.bson`, use `--format` to override).
Reads from stdin if no files are given (defaults to JSON; use `--format` to override).

## Options

| Flag | Description |
|---|---|
| `--format <FORMAT>` | Override input format: `json`, `yaml`, `xml`, `toml`, `cbor`, `bson` |
| `--output <OUTPUT>` | Output mode (default: `schema`) |
| `--name <NAME>` | Root type name for code generation (default: `Root`) |
| `--compact` | Compact JSON output. **Always use this flag.** |
| `--minimal` | Omit statistics (counts, samples, min/max). Structure only. |

### Output modes (`--output`)

| Value | Produces |
|---|---|
| `schema` | Inferred schema JSON with field metadata |
| `rust` | Rust structs with `Serialize`/`Deserialize` derives |
| `typescript` | TypeScript `interface` definitions |
| `typescript-alias` | TypeScript `type` alias definitions |
| `kotlin` | Kotlin data classes (Jackson) |
| `kotlin-kotlinx` | Kotlin data classes (Kotlinx Serialization) |
| `json-schema` | JSON Schema (Draft 2020-12) |
| `shape` | Abstract shape representation |

## Choosing the right invocation

**"I need to understand the shape of this data"** - use the default schema output:

```bash
npx schema_analysis data.json --compact --minimal
```

This gives you the type tree with nullability/optionality flags and nothing else.

**"I need types for code I'm writing"** - generate them directly:

```bash
npx schema_analysis data.json --output typescript --compact --name ApiResponse
npx schema_analysis data.json --output rust --compact --name MyData
npx schema_analysis data.json --output kotlin --compact --name Event
```

Paste the output straight into the target file.

**"I need a JSON Schema for validation or documentation":**

```bash
npx schema_analysis data.json --output json-schema --compact
```

**"I have data in memory, not in a file"** - pipe it through stdin:

```bash
echo '{"key":"value"}' | npx schema_analysis --format json --compact --minimal
```

Always pass `--format` when reading from stdin.

**"I have multiple sample files and need the unified schema":**

```bash
npx schema_analysis sample1.json sample2.json sample3.json --compact --minimal
```

Fields present in some files but not others will have `"may_be_missing": true`.

## Reading the schema output

The schema is a recursive JSON tree. Each node has a `type` (`"struct"`, `"sequence"`, `"string"`, `"integer"`, `"float"`, `"boolean"`, `"null"`).

Struct nodes have `"fields"`: a map of field name to field descriptor.
Sequence nodes have `"field"`: the schema of the array elements.

Every field descriptor carries four flags:

| Flag | Meaning |
|---|---|
| `may_be_null` | Was `null` in at least one record |
| `may_be_normal` | Had a non-null value in at least one record |
| `may_be_missing` | Was absent from at least one record (shows optionality across files) |
| `may_be_duplicate` | Key appeared more than once in the same record (relevant for XML) |

When `--minimal` is **not** used, extra statistics appear: `count`, `samples`, `min`/`max` (numbers), `min_max_length` (strings), `trues`/`falses` (booleans). Use the full output only when you need those statistics.

### Example

Input: `{"name":"Alice","age":30,"scores":[95,87]}`

`--compact --minimal` output:

```json
{"type":"struct","fields":{"name":{"may_be_null":false,"may_be_normal":true,"may_be_missing":false,"may_be_duplicate":false,"type":"string"},"age":{"may_be_null":false,"may_be_normal":true,"may_be_missing":false,"may_be_duplicate":false,"type":"integer"},"scores":{"may_be_null":false,"may_be_normal":true,"may_be_missing":false,"may_be_duplicate":false,"type":"sequence","field":{"may_be_null":false,"may_be_normal":true,"may_be_missing":false,"may_be_duplicate":false,"type":"integer"},"context":null}},"context":null}
```

`--output typescript` output:

```typescript
export interface Root {
    name: string;
    age: number;
    scores: number[];
}
```

`--output rust --name MyData` output:

```rust
use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MyData {
    pub name: String,
    pub age: i64,
    pub scores: Vec<i64>,
}
```
