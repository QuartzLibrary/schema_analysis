use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use clap::Parser;
use schema_analysis::context::{Context, DefaultContext};
use schema_analysis::helpers::xml::cleanup_xml_schema;
use schema_analysis::targets::json_typegen::OutputMode;
use schema_analysis::targets::schemars::JsonSchemaVersion;
use schema_analysis::{Coalesce, InferredSchema, Schema};
use serde::Serialize;
use serde::de::DeserializeSeed;

#[derive(Parser)]
#[command(
    name = "schema_analysis",
    version,
    about = "Infer schemas from any Serde-compatible format"
)]
struct Cli {
    /// Input files to analyze (reads from stdin if none provided)
    files: Vec<PathBuf>,

    /// Input format (auto-detected from file extension when possible)
    #[arg(long, value_enum)]
    format: Option<InputFormat>,

    /// Output mode
    #[arg(long, value_enum, default_value = "schema")]
    output: Output,

    /// Root type name for code generation
    #[arg(long, default_value = "Root")]
    name: String,

    /// Compact JSON output (no pretty printing)
    #[arg(long)]
    compact: bool,

    /// Only output the schema structure, without analysis info (counts, samples, min/max, etc.)
    #[arg(long)]
    no_analysis: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum InputFormat {
    Json,
    Yaml,
    Xml,
    Toml,
    Cbor,
    Bson,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Output {
    Schema,
    Rust,
    Typescript,
    TypescriptAlias,
    Kotlin,
    KotlinKotlinx,
    JsonSchema,
    Shape,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let format = resolve_format(&cli)?;

    let output = if cli.no_analysis {
        let mut schema = infer_schema::<()>(format, &cli.files)?;
        if format == InputFormat::Xml {
            cleanup_xml_schema(&mut schema);
        }
        generate_output(&schema, &cli)?
    } else {
        let mut schema = infer_schema::<DefaultContext>(format, &cli.files)?;
        if format == InputFormat::Xml {
            cleanup_xml_schema(&mut schema);
        }
        generate_output(&schema, &cli)?
    };

    println!("{output}");

    Ok(())
}

fn resolve_format(cli: &Cli) -> Result<InputFormat> {
    if let Some(format) = cli.format {
        return Ok(format);
    }
    if cli.files.is_empty() {
        return Ok(InputFormat::Json);
    }
    detect_format(&cli.files[0])
}
fn detect_format(path: &Path) -> Result<InputFormat> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());

    match ext.as_deref() {
        Some("json") => Ok(InputFormat::Json),
        Some("yaml" | "yml") => Ok(InputFormat::Yaml),
        Some("xml") => Ok(InputFormat::Xml),
        Some("toml") => Ok(InputFormat::Toml),
        Some("cbor") => Ok(InputFormat::Cbor),
        Some("bson") => Ok(InputFormat::Bson),
        Some(other) => bail!(
            "Unrecognized file extension '.{other}'. Use --format to specify the input format.\n\
             Supported formats: json, yaml, xml, toml, cbor, bson"
        ),
        None => bail!(
            "File '{}' has no extension. Use --format to specify the input format.",
            path.display()
        ),
    }
}

fn infer_schema<C: Context + Default>(format: InputFormat, files: &[PathBuf]) -> Result<Schema<C>>
where
    Schema<C>: Coalesce,
{
    if files.is_empty() {
        let reader = BufReader::new(io::stdin().lock());
        let inferred: InferredSchema<C> =
            deserialize(format, reader).context("Failed to parse stdin")?;
        return Ok(inferred.schema);
    }

    let mut state: Option<InferredSchema<C>> = None;

    for path in files {
        let reader = open_file(path)?;
        match &mut state {
            None => {
                state = Some(
                    deserialize(format, reader)
                        .with_context(|| format!("Failed to parse '{}'", path.display()))?,
                );
            }
            Some(inferred) => {
                merge(format, inferred, reader)
                    .with_context(|| format!("Failed to parse '{}'", path.display()))?;
            }
        }
    }

    Ok(state.expect("files is non-empty").schema)
}

fn open_file(path: &Path) -> Result<BufReader<std::fs::File>> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open '{}'", path.display()))?;
    Ok(BufReader::new(file))
}

/// Deserialize the first source into a new [InferredSchema].
fn deserialize<C: Context + Default>(
    format: InputFormat,
    reader: impl Read,
) -> Result<InferredSchema<C>> {
    match format {
        InputFormat::Json => serde_json::from_reader(reader).context("Failed to parse JSON"),
        InputFormat::Yaml => {
            let bytes = read_all(reader)?;
            serde_yaml::from_slice(&bytes).context("Failed to parse YAML")
        }
        InputFormat::Xml => {
            quick_xml::de::from_reader(BufReader::new(reader)).context("Failed to parse XML")
        }
        InputFormat::Toml => {
            let s = read_all_string(reader)?;
            toml::from_str(&s).context("Failed to parse TOML")
        }
        InputFormat::Cbor => serde_cbor::from_reader(reader).context("Failed to parse CBOR"),
        InputFormat::Bson => {
            let bytes = read_all(reader)?;
            bson::from_slice(&bytes).context("Failed to parse BSON")
        }
    }
}

/// Merge a subsequent source into an existing [InferredSchema] via [DeserializeSeed].
fn merge<C: Context>(
    format: InputFormat,
    inferred: &mut InferredSchema<C>,
    reader: impl Read,
) -> Result<()>
where
    Schema<C>: Coalesce,
{
    match format {
        InputFormat::Json => {
            let mut de = serde_json::Deserializer::from_reader(reader);
            inferred.deserialize(&mut de)?;
        }
        InputFormat::Cbor => {
            let mut de = serde_cbor::Deserializer::from_reader(reader);
            inferred.deserialize(&mut de)?;
        }
        InputFormat::Yaml => {
            let bytes = read_all(reader)?;
            for document in serde_yaml::Deserializer::from_slice(&bytes) {
                inferred.deserialize(document)?;
            }
        }
        InputFormat::Xml => {
            let mut de = quick_xml::de::Deserializer::from_reader(BufReader::new(reader));
            inferred.deserialize(&mut de)?;
        }
        InputFormat::Toml => {
            let s = read_all_string(reader)?;
            let mut de = toml::Deserializer::new(&s);
            inferred.deserialize(&mut de)?;
        }
        InputFormat::Bson => {
            let doc = bson::Document::from_reader(reader).context("Failed to parse BSON")?;
            let de = bson::Deserializer::new(bson::Bson::Document(doc));
            inferred.deserialize(de)?;
        }
    }
    Ok(())
}

fn read_all(mut reader: impl Read) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}

fn read_all_string(mut reader: impl Read) -> Result<String> {
    let mut s = String::new();
    reader.read_to_string(&mut s)?;
    Ok(s)
}

fn generate_output<C: Context>(schema: &Schema<C>, cli: &Cli) -> Result<String>
where
    Schema<C>: Serialize,
{
    match cli.output {
        Output::Schema => {
            if cli.compact {
                serde_json::to_string(schema).context("Failed to serialize schema")
            } else {
                serde_json::to_string_pretty(schema).context("Failed to serialize schema")
            }
        }

        Output::JsonSchema => schema
            .to_json_schema_with_schemars_version(&JsonSchemaVersion::Draft2019_09)
            .context("Failed to generate JSON Schema"),

        output => {
            let mode = match output {
                Output::Rust => OutputMode::Rust,
                Output::Typescript => OutputMode::Typescript,
                Output::TypescriptAlias => OutputMode::TypescriptTypeAlias,
                Output::Kotlin => OutputMode::KotlinJackson,
                Output::KotlinKotlinx => OutputMode::KotlinKotlinx,
                Output::Shape => OutputMode::Shape,
                Output::Schema | Output::JsonSchema => unreachable!(),
            };
            schema
                .process_with_json_typegen_options(&cli.name, &{
                    let mut opts = schema_analysis::targets::json_typegen::Options::default();
                    opts.output_mode = mode;
                    opts
                })
                .map_err(|e| anyhow::anyhow!("{e}"))
        }
    }
}
