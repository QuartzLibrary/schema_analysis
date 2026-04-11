# schema_analysis

## Universal-ish Schema Analysis

Ever wished you could figure out what was in that json file? Or maybe it was xml... Ehr, yaml?
It was definitely toml.

Alas, many great tools will only work with one of those formats, and the internet is not so
nice a place as to finally understand that no, xml is not an acceptable data format.

Enter this neat little tool, a single interface to any self-describing format supported by
our gymnast friend, serde.

### Features

- Works with any self-describing format with a Serde implementation.
- Suitable for large files.
- Keeps track of some useful info for each type (opt out with --minimal).
- Keeps track of null/missing/duplicate values separately.
- Integrates with [Schemars](https://github.com/GREsau/schemars) and 
  [json_typegen](https://github.com/evestera/json_typegen) to produce types and a json schema if needed.
- There's a demo website [here](https://schema-analysis.com/).

### Installation

```bash
# Run without installing
npx schema_analysis data.json
# or
uvx schema_analysis data.json
# or
pipx run schema_analysis data.json

# Install
npm install -g schema_analysis
# or
pip install schema_analysis
# or
uv tool install schema_analysis
# or
cargo install schema_analysis --features cli --locked
```

### CLI Usage

`schema_analysis` can infer schemas and generate types from data directly from the command line.

```
schema_analysis [OPTIONS] [FILES]...
```

It auto-detects the input format from file extensions (`.json`, `.yaml`/`.yml`, `.xml`, `.toml`, `.cbor`, `.bson`)
and reads from stdin if no files are provided.

**Options:**

| Option | Description | Default |
| --- | --- | --- |
| `--format <FORMAT>` | Override input format (`json`, `yaml`, `xml`, `toml`, `cbor`, `bson`) | auto-detected |
| `--output <OUTPUT>` | Output mode (`schema`, `rust`, `typescript`, `typescript-alias`, `kotlin`, `kotlin-kotlinx`, `json-schema`, `shape`) | `schema` |
| `--name <NAME>` | Root type name for code generation | `Root` |
| `--compact` | Compact JSON output (no pretty printing) | |
| `--minimal` | Skip analysis info (counts, samples, min/max, etc.), outputting only the schema structure | |

**Examples:**

```bash
# Infer a schema from a JSON file
schema_analysis data.json

# Generate Rust types
schema_analysis data.json --output rust --name MyData

# Generate TypeScript interfaces
schema_analysis api.json --output typescript --name ApiResponse

# Generate JSON Schema
schema_analysis data.json --output json-schema

# Merge multiple files into a single schema
schema_analysis file1.json file2.json file3.json

# Read from stdin
cat data.json | schema_analysis --format json
```

### Agent Skill

An [agent skill](https://agentskills.io/) is included in [`.agents/skills/schema-analysis`](.agents/skills/schema-analysis/SKILL.md),
compatible with Claude Code, Cursor, GitHub Copilot, and [other agents](https://agentskills.io/specification#supported-products).

Install the skill with:

```bash
npx skills add QuartzLibrary/schema_analysis
```

Or copy [`.agents/skills/schema-analysis`](.agents/skills/schema-analysis) into your project's `.agents/skills/` directory.

### Library Usage

For use as a library, see the [Rust crate](https://crates.io/crates/schema_analysis/) or the [repo](https://github.com/QuartzLibrary/schema_analysis).

### Performance

> These are not proper benchmarks, but should give a vague idea of the performance on a i7-7700HQ laptop (2017) laptop with the raw data already loaded into memory.

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
