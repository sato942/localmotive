//! Minimal GGUF header reader.
//!
//! Reads only the key/value metadata section — never tensor data — so it is
//! cheap even for 100 GB shards. Used to give the AI tuner architectural facts
//! (layer count, native context, embedding width, expert count) instead of
//! letting it guess from a filename.

use serde::Serialize;
use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
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
/// Metadata key/value pairs are counted; a file claiming more fails before
/// any per-pair work starts (audit DC-08).
const MAX_KV_COUNT: u64 = 1_000_000;
/// One metadata key (audit DC-08).
const MAX_KEY_BYTES: u64 = 16 * 1024;
/// One retained string value (audit DC-08).
const MAX_RETAINED_STRING_BYTES: u64 = 64 * 1024;
/// Aggregate bytes retained across the whole parsed summary (audit DC-08).
const MAX_RETAINED_METADATA_BYTES: u64 = 2 * 1024 * 1024;
/// Aggregate parser work units (one per scalar or array element touched),
/// bounding total CPU spent on adversarially arranged metadata (audit DC-08).
const MAX_PARSER_WORK_UNITS: u64 = 64 * 1024 * 1024;

/// The documented allocation/work limits of one parse. Production always
/// uses `Default`; tests can pass tiny limits to assert exact boundary
/// behavior deterministically (audit DC-08 V1).
#[derive(Clone, Copy, Debug)]
pub struct ParseLimits {
    pub max_kv_count: u64,
    pub max_key_bytes: u64,
    pub max_string_bytes: u64,
    pub max_retained_bytes: u64,
    pub max_work_units: u64,
    pub max_array_elements: u64,
}

impl Default for ParseLimits {
    fn default() -> Self {
        Self {
            max_kv_count: MAX_KV_COUNT,
            max_key_bytes: MAX_KEY_BYTES,
            max_string_bytes: MAX_RETAINED_STRING_BYTES,
            max_retained_bytes: MAX_RETAINED_METADATA_BYTES,
            max_work_units: MAX_PARSER_WORK_UNITS,
            max_array_elements: MAX_ARRAY_ELEMENTS,
        }
    }
}

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
    /// Optional split metadata written by tooling that produced a shard set
    /// (audit S-02). Absent keys stay `None`: the completeness of the tensor
    /// set is then unknowable from metadata alone, and callers must not
    /// upgrade it to a claim.
    pub split_no: Option<u64>,
    pub split_count: Option<u64>,
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

struct Reader<'a, R: Read> {
    inner: R,
    consumed: u64,
    work: u64,
    retained: u64,
    cancel: Option<&'a std::sync::atomic::AtomicBool>,
    limits: ParseLimits,
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

/// One reader with explicit accounting: consumed bytes bound the input
/// range, `work` bounds parser effort, and `retained` bounds the bytes that
/// survive into the returned summary (audit DC-08).
fn budget_error(what: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("GGUF metadata exceeds the supported {what} budget"),
    )
}

fn cancelled_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::Interrupted,
        "GGUF metadata reading was cancelled",
    )
}

impl<R: Read> Reader<'_, R> {
    fn spend_work(&mut self, units: u64) -> io::Result<()> {
        self.work = self.work.saturating_add(units);
        if self.work > self.limits.max_work_units {
            return Err(budget_error("parser work"));
        }
        Ok(())
    }

    fn retain(&mut self, bytes: u64) -> io::Result<()> {
        self.retained = self.retained.saturating_add(bytes);
        if self.retained > self.limits.max_retained_bytes {
            return Err(budget_error("retained metadata"));
        }
        Ok(())
    }

    fn check_cancelled(&self) -> io::Result<()> {
        if self
            .cancel
            .is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed))
        {
            return Err(cancelled_error());
        }
        Ok(())
    }

    fn read_array<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        checked_header_length(self.consumed, N as u64)?;
        let mut buf = [0_u8; N];
        self.inner.read_exact(&mut buf)?;
        self.consumed += N as u64;
        Ok(buf)
    }

    /// Read `len` declared bytes, discarding them in bounded chunks instead
    /// of allocating the whole declared length (audit DC-08).
    fn discard_bytes(&mut self, len: u64) -> io::Result<()> {
        let mut remaining = checked_header_length(self.consumed, len)?;
        let mut buffer = [0_u8; 16 * 1024];
        while remaining > 0 {
            let step = remaining.min(buffer.len());
            self.inner.read_exact(&mut buffer[..step])?;
            remaining -= step;
        }
        self.consumed += len;
        Ok(())
    }

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
        Ok(self.read_array::<1>()?[0])
    }
    fn u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.read_array::<2>()?))
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.read_array::<4>()?))
    }
    fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.read_array::<8>()?))
    }
    /// A string with an explicit retained-byte cap: the declared length is
    /// validated before any allocation (audit DC-08).
    fn string(&mut self, max_bytes: u64, retain: bool) -> io::Result<String> {
        let requested = self.u64()?;
        let len = checked_header_length(self.consumed, requested)?;
        if len as u64 > max_bytes {
            return Err(budget_error("string length"));
        }
        self.spend_work(1)?;
        if !retain {
            self.discard_bytes(len as u64)?;
            return Ok(String::new());
        }
        self.retain(len as u64)?;
        Ok(String::from_utf8_lossy(&self.bytes(len)?).into_owned())
    }
    fn scalar(&mut self, kind: u32, capture: bool) -> io::Result<Value> {
        self.check_cancelled()?;
        self.spend_work(1)?;
        Ok(match kind {
            0 => Value::U(self.u8()? as u64),
            1 => Value::I(self.u8()? as i8 as i64),
            2 => Value::U(self.u16()? as u64),
            3 => Value::I(self.u16()? as i16 as i64),
            4 => Value::U(self.u32()? as u64),
            5 => Value::I(self.u32()? as i32 as i64),
            6 => Value::F(f32::from_le_bytes(self.read_array::<4>()?) as f64),
            7 => Value::Bool(self.u8()? != 0),
            8 => Value::Str(self.string(self.limits.max_string_bytes, capture)?),
            10 => Value::U(self.u64()?),
            11 => Value::I(self.u64()? as i64),
            12 => Value::F(f64::from_le_bytes(self.read_array::<8>()?)),
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unknown GGUF value type {other}"),
                ))
            }
        })
    }

    /// Fixed byte width of a scalar element type, when one exists.
    fn fixed_scalar_width(kind: u32) -> Option<u64> {
        match kind {
            0..=1 | 7 => Some(1),
            2 | 3 => Some(2),
            4..=6 => Some(4),
            10..=12 => Some(8),
            _ => None,
        }
    }

    fn value(&mut self, kind: u32, capture: bool, depth: u8) -> io::Result<Value> {
        if kind != 9 {
            return self.scalar(kind, capture);
        }
        self.check_cancelled()?;
        if depth >= MAX_ARRAY_DEPTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "GGUF metadata array nesting exceeds the supported depth",
            ));
        }
        let element = self.u32()?;
        let count = self.u64()?;
        if count > self.limits.max_array_elements {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "GGUF metadata array exceeds the supported element count",
            ));
        }
        if !capture {
            // An irrelevant array of fixed-width scalars is skipped in one
            // checked byte count; only its work units are charged (audit
            // DC-08). Variable-width elements are still discarded through
            // the bounded readers above.
            if let Some(width) = Self::fixed_scalar_width(element) {
                self.spend_work(count)?;
                let total = count.checked_mul(width).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "GGUF metadata array length overflows",
                    )
                })?;
                self.discard_bytes(total)?;
                return Ok(Value::Array {
                    element_type: element,
                    count,
                    values: Vec::new(),
                    truncated: false,
                });
            }
            self.spend_work(count)?;
            for index in 0..count {
                if index % 4096 == 0 {
                    self.check_cancelled()?;
                }
                // capture=false below keeps strings discarded and nested
                // arrays skipped rather than retained.
                self.value(element, false, depth + 1)?;
            }
            return Ok(Value::Array {
                element_type: element,
                count,
                values: Vec::new(),
                truncated: false,
            });
        }
        let retained_cap = MAX_CAPTURED_ARRAY_VALUES as u64;
        let mut values = Vec::with_capacity(
            usize::try_from(count.min(retained_cap)).unwrap_or(MAX_CAPTURED_ARRAY_VALUES),
        );
        for index in 0..count {
            if index % 4096 == 0 {
                self.check_cancelled()?;
            }
            let keep = index < retained_cap;
            let value = self.value(element, keep, depth + 1)?;
            if keep {
                values.push(value);
            }
        }
        Ok(Value::Array {
            element_type: element,
            count,
            values,
            truncated: count > retained_cap,
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

#[cfg(test)]
fn parse<R: Read>(inner: R) -> io::Result<GgufSummary> {
    parse_with(inner, None)
}

#[cfg(test)]
fn parse_with_limits<R: Read>(
    inner: R,
    cancel: Option<&std::sync::atomic::AtomicBool>,
    limits: ParseLimits,
) -> io::Result<GgufSummary> {
    parse_inner(inner, cancel, limits)
}

fn parse_with<R: Read>(
    inner: R,
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> io::Result<GgufSummary> {
    parse_inner(inner, cancel, ParseLimits::default())
}

fn parse_inner<R: Read>(
    inner: R,
    cancel: Option<&std::sync::atomic::AtomicBool>,
    limits: ParseLimits,
) -> io::Result<GgufSummary> {
    let mut reader = Reader {
        inner,
        consumed: 0,
        work: 0,
        retained: 0,
        cancel,
        limits,
    };
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
    if kv_count > reader.limits.max_kv_count {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "GGUF metadata key/value count exceeds the supported limit",
        ));
    }
    let mut arch = String::new();
    let mut pairs = Vec::with_capacity(kv_count.min(4096) as usize);
    for index in 0..kv_count {
        if index % 1024 == 0 {
            reader.check_cancelled()?;
        }
        reader.spend_work(1)?;
        let key = reader.string(reader.limits.max_key_bytes, true)?;
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
    // Split metadata is optional (audit S-02): tooling that writes shard sets
    // may record the shard's own index and the total count; when the keys are
    // absent the values stay unknown instead of being guessed from names.
    summary.split_no = general("split.no").and_then(Value::as_u64);
    summary.split_count = general("split.count").and_then(Value::as_u64);
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
        let name = reader.string(reader.limits.max_key_bytes, true)?;
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
    read_summary_cancellable(path, None)
}

/// `read_summary` with an optional cancellation flag, checked between parse
/// steps so a large or malformed header cannot hold the worker hostage
/// (audit DC-08 I4).
pub fn read_summary_cancellable(
    path: &Path,
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> Result<GgufSummary, String> {
    let mut file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    file.seek(SeekFrom::Start(0))
        .map_err(|error| error.to_string())?;
    parse_with(BufReader::new(file), cancel).map_err(|error| format!("{}: {error}", path.display()))
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

    /// The same header with `split.no`/`split.count` recorded, as shard
    /// tooling writes them (audit S-02).
    fn fixture_with_split(no: u32, count: u32) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend(GGUF_MAGIC);
        out.extend(3_u32.to_le_bytes());
        out.extend(266_u64.to_le_bytes());
        out.extend(14_u64.to_le_bytes());
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
        kv_u32(&mut out, "split.no", no);
        kv_u32(&mut out, "split.count", count);
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
    fn s02_split_metadata_is_read_when_present_and_stays_unknown_when_absent() {
        let plain = parse(fixture().as_slice()).unwrap();
        assert_eq!(plain.split_no, None);
        assert_eq!(plain.split_count, None);
        let shard = parse(fixture_with_split(2, 4).as_slice()).unwrap();
        assert_eq!(shard.split_no, Some(2));
        assert_eq!(shard.split_count, Some(4));
    }

    fn gguf_header(tensor_count: u64, kv_count: u64) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend(GGUF_MAGIC);
        out.extend(3_u32.to_le_bytes());
        out.extend(tensor_count.to_le_bytes());
        out.extend(kv_count.to_le_bytes());
        out
    }

    fn tiny_limits() -> ParseLimits {
        ParseLimits {
            max_kv_count: 4,
            max_key_bytes: 64,
            max_string_bytes: 16,
            max_retained_bytes: 64,
            max_work_units: 16,
            max_array_elements: 4,
        }
    }

    #[test]
    fn dc08_a_short_file_declaring_a_giant_string_fails_before_allocating() {
        // A tiny file claims a 100 MiB string: the declared length must fail
        // the string budget while the real input is a few dozen bytes, so no
        // parser can allocate the claimed size first (audit DC-08 V1).
        let mut bytes = gguf_header(0, 1);
        put_str(&mut bytes, "general.name");
        bytes.extend(8_u32.to_le_bytes());
        bytes.extend((100 * 1024 * 1024_u64).to_le_bytes());
        bytes.extend(b"only-ten!!");
        assert!(bytes.len() < 512, "fixture must stay tiny");
        let error = parse(bytes.as_slice()).unwrap_err();
        assert!(error.to_string().contains("string length"), "{error}");
    }

    #[test]
    fn dc08_kv_key_and_count_budget_failures_are_deterministic() {
        // Oversized key: beyond its own budget (audit DC-08 V1).
        let mut bytes = gguf_header(0, 1);
        put_str(&mut bytes, &"k".repeat(20 * 1024));
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend(0_u32.to_le_bytes());
        let error = parse_with_limits(bytes.as_slice(), None, tiny_limits()).unwrap_err();
        assert!(error.to_string().contains("string length"), "{error}");
        // One kv beyond the declared budget fails before any pair work starts.
        let bytes = gguf_header(0, 5);
        let error = parse_with_limits(bytes.as_slice(), None, tiny_limits()).unwrap_err();
        assert!(error.to_string().contains("key/value count"), "{error}");
        // Exactly at the budget still parses.
        let mut bytes = gguf_header(0, 4);
        kv_u32(&mut bytes, "a", 1);
        kv_u32(&mut bytes, "b", 2);
        kv_u32(&mut bytes, "c", 3);
        kv_u32(&mut bytes, "d", 4);
        let summary = parse_with_limits(bytes.as_slice(), None, tiny_limits()).unwrap();
        assert_eq!(summary.kv_count, 4);
        assert_eq!(summary.metadata_facts.len(), 0);
    }

    #[test]
    fn dc08_arrays_are_skipped_by_shape_and_bounded_by_element_and_depth_budgets() {
        // A fixed-width irrelevant array is skipped with one checked byte
        // count and stays out of the retained facts (audit DC-08 V1).
        let mut bytes = gguf_header(0, 1);
        put_str(&mut bytes, "unrelated.arr");
        bytes.extend(9_u32.to_le_bytes());
        bytes.extend(7_u32.to_le_bytes());
        bytes.extend(4_u64.to_le_bytes());
        bytes.extend([1_u8, 0, 1, 0]);
        let summary = parse_with_limits(bytes.as_slice(), None, tiny_limits()).unwrap();
        assert!(summary.metadata_facts.is_empty());
        // One element beyond the limit fails with the explicit reason.
        let mut bytes = gguf_header(0, 1);
        put_str(&mut bytes, "unrelated.arr");
        bytes.extend(9_u32.to_le_bytes());
        bytes.extend(7_u32.to_le_bytes());
        bytes.extend(5_u64.to_le_bytes());
        bytes.extend([1_u8, 0, 1, 0, 1]);
        let error = parse_with_limits(bytes.as_slice(), None, tiny_limits()).unwrap_err();
        assert!(error.to_string().contains("element count"), "{error}");
        // A non-captured tokenizer vocabulary is stream-discarded rather
        // than retained (audit DC-08 I2).
        let mut bytes = gguf_header(0, 1);
        put_str(&mut bytes, "tokenizer.ggml.tokens");
        bytes.extend(9_u32.to_le_bytes());
        bytes.extend(8_u32.to_le_bytes());
        bytes.extend(4_u64.to_le_bytes());
        for value in ["a", "bb", "ccc", "dddd"] {
            put_str(&mut bytes, value);
        }
        let summary = parse_with_limits(bytes.as_slice(), None, tiny_limits()).unwrap();
        assert!(summary.metadata_facts.is_empty());
        // Nested arrays beyond the supported depth fail cleanly.
        let mut bytes = gguf_header(0, 1);
        put_str(&mut bytes, "lfm2.attention.nested");
        bytes.extend(9_u32.to_le_bytes()); // value kind: array
        for _ in 0..5 {
            bytes.extend(9_u32.to_le_bytes()); // element type: array
            bytes.extend(1_u64.to_le_bytes()); // count: 1
        }
        let error = parse_with_limits(bytes.as_slice(), None, tiny_limits()).unwrap_err();
        assert!(error.to_string().contains("nesting"), "{error}");
    }

    #[test]
    fn dc08_cancellation_and_truncation_produce_prompt_errors() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        // A reader that flips the cancel flag once it has supplied 8 bytes:
        // the kv loop's cancellation check must stop the parse promptly
        // (audit DC-08 V2).
        struct FlipOnRead<R: Read> {
            inner: R,
            flag: Arc<AtomicBool>,
            seen: usize,
        }
        impl<R: Read> Read for FlipOnRead<R> {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                let n = self.inner.read(buf)?;
                self.seen += n;
                if self.seen >= 8 {
                    self.flag.store(true, Ordering::Relaxed);
                }
                Ok(n)
            }
        }
        let mut bytes = gguf_header(0, 2);
        kv_u32(&mut bytes, "a", 1);
        kv_u32(&mut bytes, "b", 2);
        let flag = Arc::new(AtomicBool::new(false));
        let reader = FlipOnRead {
            inner: bytes.as_slice(),
            flag: flag.clone(),
            seen: 0,
        };
        let error = parse_with(reader, Some(&flag)).unwrap_err();
        assert!(error.to_string().contains("cancelled"), "{error}");
        // A file shortened during parsing surfaces as a read error.
        let mut truncated = gguf_header(0, 1);
        kv_u32(&mut truncated, "a", 1);
        truncated.truncate(truncated.len() - 2);
        assert!(parse(truncated.as_slice()).is_err());
    }

    #[test]
    fn dc08_truncated_prefixes_fail_cleanly_and_valid_headers_stay_useful() {
        // Fuzz every short prefix of a valid header: no panics, explicit
        // errors, and the intact fixture keeps its measured facts
        // (audit DC-08 V3).
        let valid = fixture();
        for cut in 0..64 {
            let outcome = std::panic::catch_unwind(|| parse(&valid[..cut]));
            assert!(outcome.is_ok(), "truncated prefix {cut} panicked");
            assert!(
                outcome.unwrap().is_err(),
                "truncated prefix {cut} unexpectedly parsed"
            );
        }
        let summary = parse(valid.as_slice()).unwrap();
        assert_eq!(summary.architecture, "lfm2");
        assert_eq!(summary.block_count, Some(30));
        assert_eq!(summary.context_length, Some(131072));
        assert!(summary.header_bytes < MAX_HEADER_BYTES);
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
