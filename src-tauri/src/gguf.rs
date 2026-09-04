//! Minimal GGUF header reader.
//!
//! Reads only the key/value metadata section — never tensor data — so it is
//! cheap even for 100 GB shards. Used to give the AI tuner architectural facts
//! (layer count, native context, embedding width, expert count) instead of
//! letting it guess from a filename.

use serde::Serialize;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

const GGUF_MAGIC: &[u8; 4] = b"GGUF";
/// Reading stops after this many bytes of metadata; real headers are a few MiB
/// at most (tokenizer vocab), so this is a safety valve rather than a limit.
const MAX_HEADER_BYTES: u64 = 256 * 1024 * 1024;
const MAX_ARRAY_ELEMENTS: u64 = 16 * 1024 * 1024;
const MAX_CAPTURED_ARRAY_VALUES: usize = 4_096;
const MAX_ARRAY_DEPTH: u8 = 4;
const MAX_TENSOR_COUNT: u64 = 1_000_000;
const MAX_RECORDED_TENSORS: usize = 4_096;
const MAX_TENSOR_DIMENSIONS: u32 = 8;

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum MetadataScalar {
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    Boolean(bool),
    String(String),
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(tag = "shape", rename_all = "camelCase")]
pub enum MetadataValue {
    Scalar {
        value: MetadataScalar,
    },
    Array {
        element_type: u32,
        count: u64,
        values: Vec<MetadataScalar>,
        truncated: bool,
    },
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MetadataFact {
    pub key: String,
    pub value: MetadataValue,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TensorDescriptor {
    pub name: String,
    pub dimensions: Vec<u64>,
    pub element_type: u32,
    pub offset: u64,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GgufSummary {
    pub version: u32,
    pub architecture: String,
    pub name: String,
    pub size_label: String,
    pub file_type: Option<u32>,
    pub block_count: Option<u64>,
    pub context_length: Option<u64>,
    pub embedding_length: Option<u64>,
    pub head_count: Option<u64>,
    pub head_count_kv: Option<u64>,
    pub key_length: Option<u64>,
    pub value_length: Option<u64>,
    pub expert_count: Option<u64>,
    pub expert_used_count: Option<u64>,
    pub vocab_size: Option<u64>,
    pub rope_freq_base: Option<f64>,
    pub tensor_count: u64,
    pub kv_count: u64,
    pub metadata_facts: Vec<MetadataFact>,
    pub tensor_descriptors: Vec<TensorDescriptor>,
    pub tensor_descriptors_truncated: bool,
    pub header_bytes: u64,
}

#[derive(Clone, Debug, PartialEq)]
enum Value {
    U(u64),
    I(i64),
    F(f64),
    Bool(bool),
    Str(String),
    Array {
        element_type: u32,
        count: u64,
        values: Vec<Value>,
        truncated: bool,
    },
}

impl Value {
    fn as_u64(&self) -> Option<u64> {
        match self {
            Value::U(v) => Some(*v),
            Value::I(v) if *v >= 0 => Some(*v as u64),
            _ => None,
        }
    }
    fn as_f64(&self) -> Option<f64> {
        match self {
            Value::F(v) => Some(*v),
            Value::U(v) => Some(*v as f64),
            Value::I(v) => Some(*v as f64),
            _ => None,
        }
    }
    fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(v) => Some(v),
            _ => None,
        }
    }

    fn as_metadata_scalar(&self) -> Option<MetadataScalar> {
        match self {
            Value::U(value) => Some(MetadataScalar::Unsigned(*value)),
            Value::I(value) => Some(MetadataScalar::Signed(*value)),
            Value::F(value) => Some(MetadataScalar::Float(*value)),
            Value::Bool(value) => Some(MetadataScalar::Boolean(*value)),
            Value::Str(value) => Some(MetadataScalar::String(value.clone())),
            Value::Array { .. } => None,
        }
    }

    fn as_metadata_value(&self) -> Option<MetadataValue> {
        match self {
            Value::Array {
                element_type,
                count,
                values,
                truncated,
            } => Some(MetadataValue::Array {
                element_type: *element_type,
                count: *count,
                values: values
                    .iter()
                    .filter_map(Value::as_metadata_scalar)
                    .collect(),
                truncated: *truncated
                    || values
                        .iter()
                        .any(|value| matches!(value, Value::Array { .. })),
            }),
            scalar => scalar
                .as_metadata_scalar()
                .map(|value| MetadataValue::Scalar { value }),
        }
    }
}

struct Reader<R: Read> {
    inner: R,
    consumed: u64,
}

fn checked_header_length(consumed: u64, requested: u64) -> io::Result<usize> {
    let remaining = MAX_HEADER_BYTES.saturating_sub(consumed);
    if requested > remaining {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "GGUF metadata exceeds the supported header size",
        ));
    }
    usize::try_from(requested).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "GGUF metadata length does not fit this platform",
        )
    })
}

impl<R: Read> Reader<R> {
    fn bytes(&mut self, n: usize) -> io::Result<Vec<u8>> {
        checked_header_length(
            self.consumed,
            u64::try_from(n).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "GGUF metadata length is invalid",
                )
            })?,
        )?;
        let mut buf = vec![0_u8; n];
        self.inner.read_exact(&mut buf)?;
        self.consumed += n as u64;
        Ok(buf)
    }
    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.bytes(1)?[0])
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.bytes(8)?.try_into().unwrap()))
    }
    fn string(&mut self) -> io::Result<String> {
        let requested = self.u64()?;
        let len = checked_header_length(self.consumed, requested)?;
        Ok(String::from_utf8_lossy(&self.bytes(len)?).into_owned())
    }
    fn scalar(&mut self, kind: u32) -> io::Result<Value> {
        Ok(match kind {
            0 => Value::U(self.u8()? as u64),
            1 => Value::I(self.u8()? as i8 as i64),
            2 => Value::U(u16::from_le_bytes(self.bytes(2)?.try_into().unwrap()) as u64),
            3 => Value::I(i16::from_le_bytes(self.bytes(2)?.try_into().unwrap()) as i64),
            4 => Value::U(self.u32()? as u64),
            5 => Value::I(self.u32()? as i32 as i64),
            6 => Value::F(f32::from_le_bytes(self.bytes(4)?.try_into().unwrap()) as f64),
            7 => Value::Bool(self.u8()? != 0),
            8 => Value::Str(self.string()?),
            10 => Value::U(self.u64()?),
            11 => Value::I(self.u64()? as i64),
            12 => Value::F(f64::from_le_bytes(self.bytes(8)?.try_into().unwrap())),
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unknown GGUF value type {other}"),
                ))
            }
        })
    }
    fn value(&mut self, kind: u32, capture: bool, depth: u8) -> io::Result<Value> {
        if kind != 9 {
            return self.scalar(kind);
        }
        if depth >= MAX_ARRAY_DEPTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "GGUF metadata array nesting exceeds the supported depth",
            ));
        }
        let element = self.u32()?;
        let count = self.u64()?;
        if count > MAX_ARRAY_ELEMENTS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "GGUF metadata array exceeds the supported element count",
            ));
        }
        let mut values = Vec::with_capacity(
            usize::try_from(count)
                .unwrap_or(usize::MAX)
                .min(MAX_CAPTURED_ARRAY_VALUES),
        );
        for index in 0..count {
            let keep = capture && index < MAX_CAPTURED_ARRAY_VALUES as u64;
            let value = self.value(element, keep, depth + 1)?;
            if keep {
                values.push(value);
            }
        }
        Ok(Value::Array {
            element_type: element,
            count,
            values,
            truncated: capture && count > MAX_CAPTURED_ARRAY_VALUES as u64,
        })
    }
}

fn relevant_metadata_key(key: &str) -> bool {
    key.starts_with("general.")
        || key.starts_with("adapter.")
        || matches!(key, "tokenizer.ggml.model" | "tokenizer.ggml.pre")
        || key.contains(".attention.")
        || key.contains(".recurrent.")
        || key.contains(".ssm.")
        || [
            ".block_count",
            ".context_length",
            ".embedding_length",
            ".projection_dim",
            ".expert_count",
            ".expert_used_count",
            ".vocab_size",
            ".rope.",
        ]
        .iter()
        .any(|part| key.contains(part))
}

fn parse<R: Read>(inner: R) -> io::Result<GgufSummary> {
    let mut reader = Reader { inner, consumed: 0 };
    if reader.bytes(4)?.as_slice() != GGUF_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Not a GGUF file (bad magic)",
        ));
    }
    let version = reader.u32()?;
    if !(2..=3).contains(&version) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsupported GGUF version {version}"),
        ));
    }
    let tensor_count = reader.u64()?;
    let kv_count = reader.u64()?;
    let mut summary = GgufSummary {
        version,
        tensor_count,
        kv_count,
        ..Default::default()
    };
    let mut arch = String::new();
    let mut pairs = Vec::with_capacity(kv_count.min(4096) as usize);
    for _ in 0..kv_count {
        let key = reader.string()?;
        let kind = reader.u32()?;
        let value = reader.value(kind, relevant_metadata_key(&key), 0)?;
        if key == "general.architecture" {
            arch = value.as_str().unwrap_or_default().to_string();
        }
        pairs.push((key, value));
    }
    summary.architecture = arch.clone();
    let get = |suffix: &str| -> Option<&Value> {
        pairs
            .iter()
            .find(|(key, _)| key == &format!("{arch}.{suffix}"))
            .map(|(_, value)| value)
    };
    let general = |name: &str| -> Option<&Value> {
        pairs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    };
    summary.name = general("general.name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    summary.size_label = general("general.size_label")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    summary.file_type = general("general.file_type")
        .and_then(Value::as_u64)
        .map(|v| v as u32);
    summary.block_count = get("block_count").and_then(Value::as_u64);
    summary.context_length = get("context_length").and_then(Value::as_u64);
    summary.embedding_length = get("embedding_length").and_then(Value::as_u64);
    summary.head_count = get("attention.head_count").and_then(Value::as_u64);
    summary.head_count_kv = get("attention.head_count_kv").and_then(Value::as_u64);
    summary.key_length = get("attention.key_length").and_then(Value::as_u64);
    summary.value_length = get("attention.value_length").and_then(Value::as_u64);
    summary.expert_count = get("expert_count").and_then(Value::as_u64);
    summary.expert_used_count = get("expert_used_count").and_then(Value::as_u64);
    summary.vocab_size = get("vocab_size").and_then(Value::as_u64);
    summary.rope_freq_base = get("rope.freq_base").and_then(Value::as_f64);
    summary.metadata_facts = pairs
        .iter()
        .filter(|(key, _)| relevant_metadata_key(key))
        .filter_map(|(key, value)| {
            value.as_metadata_value().map(|value| MetadataFact {
                key: key.clone(),
                value,
            })
        })
        .collect();
    summary.metadata_facts.sort_by(|a, b| a.key.cmp(&b.key));
    if tensor_count > MAX_TENSOR_COUNT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "GGUF tensor count exceeds the supported limit",
        ));
    }
    for index in 0..tensor_count {
        let name = reader.string()?;
        let dimension_count = reader.u32()?;
        if dimension_count > MAX_TENSOR_DIMENSIONS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "GGUF tensor dimension count exceeds the supported limit",
            ));
        }
        let mut dimensions = Vec::with_capacity(dimension_count as usize);
        for _ in 0..dimension_count {
            dimensions.push(reader.u64()?);
        }
        let element_type = reader.u32()?;
        let offset = reader.u64()?;
        if index < MAX_RECORDED_TENSORS as u64 {
            summary.tensor_descriptors.push(TensorDescriptor {
                name,
                dimensions,
                element_type,
                offset,
            });
        }
    }
    summary.tensor_descriptors_truncated = tensor_count > MAX_RECORDED_TENSORS as u64;
    summary.header_bytes = reader.consumed;
    Ok(summary)
}

pub fn read_summary(path: &Path) -> Result<GgufSummary, String> {
    let mut file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    file.seek(SeekFrom::Start(0))
        .map_err(|error| error.to_string())?;
    parse(file).map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn put_str(out: &mut Vec<u8>, s: &str) {
        out.extend((s.len() as u64).to_le_bytes());
        out.extend(s.as_bytes());
    }
    fn kv_str(out: &mut Vec<u8>, key: &str, value: &str) {
        put_str(out, key);
        out.extend(8_u32.to_le_bytes());
        put_str(out, value);
    }
    fn kv_u32(out: &mut Vec<u8>, key: &str, value: u32) {
        put_str(out, key);
        out.extend(4_u32.to_le_bytes());
        out.extend(value.to_le_bytes());
    }
    fn kv_f32(out: &mut Vec<u8>, key: &str, value: f32) {
        put_str(out, key);
        out.extend(6_u32.to_le_bytes());
        out.extend(value.to_le_bytes());
    }
    fn kv_i32_array(out: &mut Vec<u8>, key: &str, values: &[i32]) {
        put_str(out, key);
        out.extend(9_u32.to_le_bytes());
        out.extend(5_u32.to_le_bytes());
        out.extend((values.len() as u64).to_le_bytes());
        for v in values {
            out.extend(v.to_le_bytes());
        }
    }

    fn tensor_descriptor(
        out: &mut Vec<u8>,
        name: &str,
        dimensions: &[u64],
        element_type: u32,
        offset: u64,
    ) {
        put_str(out, name);
        out.extend((dimensions.len() as u32).to_le_bytes());
        for dimension in dimensions {
            out.extend(dimension.to_le_bytes());
        }
        out.extend(element_type.to_le_bytes());
        out.extend(offset.to_le_bytes());
    }

    fn fixture() -> Vec<u8> {
        let mut out = Vec::new();
        out.extend(GGUF_MAGIC);
        out.extend(3_u32.to_le_bytes());
        out.extend(266_u64.to_le_bytes());
        out.extend(12_u64.to_le_bytes());
        kv_str(&mut out, "general.architecture", "lfm2");
        kv_str(&mut out, "general.name", "LFM Fixture");
        kv_str(&mut out, "general.size_label", "2.7B");
        kv_u32(&mut out, "general.file_type", 7);
        kv_u32(&mut out, "lfm2.block_count", 30);
        kv_u32(&mut out, "lfm2.context_length", 131072);
        kv_u32(&mut out, "lfm2.embedding_length", 2048);
        kv_u32(&mut out, "lfm2.attention.key_length", 128);
        kv_u32(&mut out, "lfm2.attention.value_length", 64);
        kv_i32_array(&mut out, "lfm2.attention.head_count_kv", &[8; 30]);
        kv_u32(&mut out, "lfm2.recurrent.state_size", 16);
        kv_f32(&mut out, "lfm2.rope.freq_base", 10_000_000.0);
        for index in 0..266 {
            tensor_descriptor(
                &mut out,
                &format!("blk.{index}.weight"),
                &[2048, 2048],
                12,
                index * 4096,
            );
        }
        out.extend(b"TENSOR DATA THAT MUST NEVER BE READ");
        out
    }

    #[test]
    fn reads_architecture_facts_from_metadata_only() {
        let summary = parse(fixture().as_slice()).unwrap();
        assert_eq!(summary.version, 3);
        assert_eq!(summary.architecture, "lfm2");
        assert_eq!(summary.name, "LFM Fixture");
        assert_eq!(summary.size_label, "2.7B");
        assert_eq!(summary.file_type, Some(7));
        assert_eq!(summary.block_count, Some(30));
        assert_eq!(summary.context_length, Some(131072));
        assert_eq!(summary.embedding_length, Some(2048));
        assert_eq!(summary.key_length, Some(128));
        assert_eq!(summary.value_length, Some(64));
        assert_eq!(
            summary.head_count_kv, None,
            "array-valued kv head counts are not a scalar"
        );
        assert_eq!(summary.rope_freq_base, Some(10_000_000.0));
        assert_eq!(summary.tensor_count, 266);
        assert_eq!(summary.kv_count, 12);
        assert_eq!(summary.expert_count, None);
    }

    #[test]
    fn captures_companion_identity_metadata_keys() {
        assert!(relevant_metadata_key("adapter.type"));
        assert!(relevant_metadata_key("tokenizer.ggml.model"));
        assert!(relevant_metadata_key("tokenizer.ggml.pre"));
        assert!(!relevant_metadata_key("tokenizer.ggml.tokens"));
    }

    #[test]
    fn preserves_relevant_scalar_and_array_metadata_shapes() {
        let summary = parse(fixture().as_slice()).unwrap();
        let heads = summary
            .metadata_facts
            .iter()
            .find(|fact| fact.key == "lfm2.attention.head_count_kv")
            .unwrap();
        assert_eq!(
            heads.value,
            MetadataValue::Array {
                element_type: 5,
                count: 30,
                values: vec![MetadataScalar::Signed(8); 30],
                truncated: false,
            }
        );

        let recurrent = summary
            .metadata_facts
            .iter()
            .find(|fact| fact.key == "lfm2.recurrent.state_size")
            .unwrap();
        assert_eq!(
            recurrent.value,
            MetadataValue::Scalar {
                value: MetadataScalar::Unsigned(16),
            }
        );
    }

    #[test]
    fn reads_bounded_tensor_descriptors_without_tensor_payload() {
        let bytes = fixture();
        let summary = parse(bytes.as_slice()).unwrap();

        assert_eq!(summary.tensor_descriptors.len(), 266);
        assert!(!summary.tensor_descriptors_truncated);
        assert_eq!(summary.tensor_descriptors[0].name, "blk.0.weight");
        assert_eq!(summary.tensor_descriptors[0].dimensions, vec![2048, 2048]);
        assert_eq!(summary.tensor_descriptors[0].element_type, 12);
        assert!(summary.header_bytes < bytes.len() as u64);
    }

    #[test]
    fn rejects_non_gguf_files() {
        let error = parse(b"RIFF....".as_slice()).unwrap_err();
        assert!(error.to_string().contains("bad magic"));
    }

    #[test]
    fn rejects_an_allocation_before_it_crosses_the_header_bound() {
        let error = checked_header_length(MAX_HEADER_BYTES, 1).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn reads_from_disk_without_touching_tensor_bytes() {
        let path =
            std::env::temp_dir().join(format!("localmotive-hdr-{}.gguf", std::process::id()));
        let mut file = File::create(&path).unwrap();
        file.write_all(&fixture()).unwrap();
        drop(file);
        let summary = read_summary(&path).unwrap();
        assert_eq!(summary.block_count, Some(30));
        std::fs::remove_file(path).unwrap();
    }
}
