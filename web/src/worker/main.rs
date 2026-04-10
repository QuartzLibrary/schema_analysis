use std::io::BufReader;

use schema_analysis::{InferredSchema, Schema};
use schema_analysis_web::{FileId, Format, FromWorker, ToWorker};
use wasm_bindgen::prelude::*;
use web_worker::oneshot::OneshotScope;

/// If smaller than this, just load it into memory.
const SMALL_SIZE: usize = 50 * 1024 * 1024; // 50 MB

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(if cfg!(debug_assertions) {
        log::Level::Debug
    } else {
        log::Level::Info
    })
    .unwrap();

    Box::leak(Box::new(OneshotScope::new_async_unordered(
        move |msg: JsValue| async move {
            let request = ToWorker::from_js(msg);

            let response: FromWorker = handle(request);

            response.to_js()
        },
    )));

    log::info!("[Worker] Init");
}

fn handle(ToWorker { format, files }: ToWorker) -> FromWorker {
    FromWorker(
        files
            .into_iter()
            .map(|(id, file)| (id, infer(id, &file, format)))
            .collect(),
    )
}

fn infer(id: FileId, file: &web_sys::File, format: Format) -> Result<Schema, String> {
    let size = file.size() as usize;

    let small = size < SMALL_SIZE;

    log::info!("[Worker] Inferring {size} bytes of {format:?} for file {id}.");

    let (result, time) = utile::time::time(|| match format {
        Format::Json if small => infer_json_small(&as_string(file)?),
        Format::Json => infer_json(reader(file)),

        Format::Yaml if small => infer_yaml_small(&as_string(file)?),
        Format::Yaml => infer_yaml(&as_bytes(file)),

        Format::Cbor if small => infer_cbor_small(&as_bytes(file)),
        Format::Cbor => infer_cbor(reader(file)),

        Format::Toml => infer_toml(&as_string(file)?),

        Format::Bson if small => infer_bson_small(&as_bytes(file)),
        Format::Bson => infer_bson(reader(file)),

        Format::Xml if small => infer_xml_small(&as_string(file)?),
        Format::Xml => infer_xml(reader(file)),
    });

    log::info!("[Worker] Inference took {time:?} for {size} bytes of {format}.");

    result.map(|r| r.schema)
}

fn infer_json(reader: impl std::io::Read) -> Result<InferredSchema, String> {
    serde_json::from_reader(reader).map_err(|e| e.to_string())
}
fn infer_json_small(slice: &str) -> Result<InferredSchema, String> {
    serde_json::from_str(slice).map_err(|e| e.to_string())
}

fn infer_yaml(data: &[u8]) -> Result<InferredSchema, String> {
    serde_yaml::from_slice(data).map_err(|e| e.to_string())
}
fn infer_yaml_small(slice: &str) -> Result<InferredSchema, String> {
    serde_yaml::from_str(slice).map_err(|e| e.to_string())
}

fn infer_cbor(reader: impl std::io::Read) -> Result<InferredSchema, String> {
    serde_cbor::from_reader(reader).map_err(|e| e.to_string())
}
fn infer_cbor_small(slice: &[u8]) -> Result<InferredSchema, String> {
    serde_cbor::from_slice(slice).map_err(|e| e.to_string())
}

fn infer_toml(s: &str) -> Result<InferredSchema, String> {
    toml::from_str(s).map_err(|e| e.to_string())
}

fn infer_bson(reader: impl std::io::Read) -> Result<InferredSchema, String> {
    bson::from_reader(reader).map_err(|e| e.to_string())
}
fn infer_bson_small(slice: &[u8]) -> Result<InferredSchema, String> {
    bson::from_slice(slice).map_err(|e| e.to_string())
}

fn infer_xml(reader: impl std::io::BufRead) -> Result<InferredSchema, String> {
    quick_xml::de::from_reader(reader).map_err(|e| e.to_string())
}
fn infer_xml_small(slice: &str) -> Result<InferredSchema, String> {
    quick_xml::de::from_str(slice).map_err(|e| e.to_string())
}

fn reader(file: &web_sys::File) -> StoredFileReader<'_> {
    StoredFileReader::File(BufReader::with_capacity(
        CHUNK_SIZE as usize,
        BlobReader::new(file),
    ))
}
fn as_bytes(file: &web_sys::File) -> Vec<u8> {
    let reader = web_sys::FileReaderSync::new().unwrap();
    let ab = reader.read_as_array_buffer(file).unwrap();
    js_sys::Uint8Array::new(&ab).to_vec()
}
fn as_string(file: &web_sys::File) -> Result<String, String> {
    let reader = web_sys::FileReaderSync::new().unwrap();
    reader.read_as_text(file).map_err(|e| format!("{e:?}"))
}

// ---------------------------------------------------------------------------
// Blob reader (worker-only: uses FileReaderSync)
// ---------------------------------------------------------------------------

const CHUNK_SIZE: u32 = 50 * 1024 * 1024; // 50 MB

/// Reads from a [`web_sys::Blob`] synchronously in chunks using
/// [`web_sys::FileReaderSync`] (available in worker contexts only).
pub struct BlobReader<'a> {
    blob: &'a web_sys::Blob,
    reader: web_sys::FileReaderSync,
    offset: f64,
    length: f64,
}

impl<'a> BlobReader<'a> {
    pub fn new(blob: &'a web_sys::Blob) -> Self {
        Self {
            blob,
            reader: web_sys::FileReaderSync::new().unwrap(),
            offset: 0.0,
            length: blob.size(),
        }
    }
}

impl std::io::Read for BlobReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let remaining = self.length - self.offset;
        if remaining <= 0.0 {
            return Ok(0);
        }
        let to_read = remaining.min(buf.len() as f64).min(CHUNK_SIZE as f64);
        let end = self.offset + to_read;
        let slice = self
            .blob
            .slice_with_f64_and_f64(self.offset, end)
            .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
        let ab = self
            .reader
            .read_as_array_buffer(&slice)
            .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
        let view = js_sys::Uint8Array::new(&ab);
        let len = to_read as usize;
        view.copy_to(&mut buf[..len]);
        self.offset = end;
        Ok(len)
    }
}

pub enum StoredFileReader<'a> {
    Pasted(std::io::Cursor<&'a [u8]>),
    File(BufReader<BlobReader<'a>>),
}

impl std::io::Read for StoredFileReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Pasted(cursor) => cursor.read(buf),
            Self::File(reader) => reader.read(buf),
        }
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        match self {
            Self::Pasted(cursor) => cursor.read_vectored(bufs),
            Self::File(reader) => reader.read_vectored(bufs),
        }
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        match self {
            Self::Pasted(cursor) => cursor.read_to_end(buf),
            Self::File(reader) => reader.read_to_end(buf),
        }
    }

    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        match self {
            Self::Pasted(cursor) => cursor.read_to_string(buf),
            Self::File(reader) => reader.read_to_string(buf),
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        match self {
            Self::Pasted(cursor) => cursor.read_exact(buf),
            Self::File(reader) => reader.read_exact(buf),
        }
    }
}

impl std::io::BufRead for StoredFileReader<'_> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        match self {
            Self::Pasted(cursor) => cursor.fill_buf(),
            Self::File(reader) => reader.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match self {
            Self::Pasted(cursor) => cursor.consume(amt),
            Self::File(reader) => reader.consume(amt),
        }
    }

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        match self {
            Self::Pasted(cursor) => cursor.read_until(byte, buf),
            Self::File(reader) => reader.read_until(byte, buf),
        }
    }

    fn skip_until(&mut self, byte: u8) -> std::io::Result<usize> {
        match self {
            Self::Pasted(cursor) => cursor.skip_until(byte),
            Self::File(reader) => reader.skip_until(byte),
        }
    }

    fn read_line(&mut self, buf: &mut String) -> std::io::Result<usize> {
        match self {
            Self::Pasted(cursor) => cursor.read_line(buf),
            Self::File(reader) => reader.read_line(buf),
        }
    }
}
