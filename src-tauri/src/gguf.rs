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

#[derive(Clone, Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GgufSummary {
    pub architecture: String,
    pub name: String,
    pub size_label: String,
    pub file_type: Option<u32>,
    pub block_count: Option<u64>,
    pub context_length: Option<u64>,
    pub embedding_length: Option<u64>,
    pub head_count: Option<u64>,
    pub head_count_kv: Option<u64>,
    pub expert_count: Option<u64>,
    pub expert_used_count: Option<u64>,
    pub vocab_size: Option<u64>,
    pub rope_freq_base: Option<f64>,
    pub tensor_count: u64,
    pub kv_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
enum Value {
    U(u64),
    I(i64),
    F(f64),
    Bool(bool),
    Str(String),
    Array(u32, u64),
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
}

struct Reader<R: Read> {
    inner: R,
    consumed: u64,
}

impl<R: Read> Reader<R> {
    fn bytes(&mut self, n: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0_u8; n];
        self.inner.read_exact(&mut buf)?;
        self.consumed += n as u64;
        if self.consumed > MAX_HEADER_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "GGUF metadata exceeds the supported header size",
            ));
        }
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
        let len = self.u64()? as usize;
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
    fn value(&mut self, kind: u32) -> io::Result<Value> {
        if kind != 9 {
            return self.scalar(kind);
        }
        let element = self.u32()?;
        let count = self.u64()?;
        for _ in 0..count {
            self.value(element)?;
        }
        Ok(Value::Array(element, count))
    }
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
        tensor_count,
        kv_count,
        ..Default::default()
    };
    let mut arch = String::new();
    let mut pairs = Vec::with_capacity(kv_count.min(4096) as usize);
    for _ in 0..kv_count {
        let key = reader.string()?;
        let kind = reader.u32()?;
        let value = reader.value(kind)?;
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
    summary.expert_count = get("expert_count").and_then(Value::as_u64);
    summary.expert_used_count = get("expert_used_count").and_then(Value::as_u64);
    summary.vocab_size = get("vocab_size").and_then(Value::as_u64);
    summary.rope_freq_base = get("rope.freq_base").and_then(Value::as_f64);
    Ok(summary)
}

pub fn read_summary(path: &Path) -> Result<GgufSummary, String> {
    let mut file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    file.seek(SeekFrom::Start(0))
        .map_err(|error| error.to_string())?;
    parse(io::BufReader::new(file)).map_err(|error| format!("{}: {error}", path.display()))
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

    fn fixture() -> Vec<u8> {
        let mut out = Vec::new();
        out.extend(GGUF_MAGIC);
        out.extend(3_u32.to_le_bytes());
        out.extend(266_u64.to_le_bytes());
        out.extend(9_u64.to_le_bytes());
        kv_str(&mut out, "general.architecture", "lfm2");
        kv_str(&mut out, "general.name", "LFM Fixture");
        kv_str(&mut out, "general.size_label", "2.7B");
        kv_u32(&mut out, "general.file_type", 7);
        kv_u32(&mut out, "lfm2.block_count", 30);
        kv_u32(&mut out, "lfm2.context_length", 131072);
        kv_u32(&mut out, "lfm2.embedding_length", 2048);
        kv_i32_array(&mut out, "lfm2.attention.head_count_kv", &[8; 30]);
        kv_f32(&mut out, "lfm2.rope.freq_base", 10_000_000.0);
        out.extend(b"TENSOR DATA THAT MUST NEVER BE READ");
        out
    }

    #[test]
    fn reads_architecture_facts_from_metadata_only() {
        let summary = parse(fixture().as_slice()).unwrap();
        assert_eq!(summary.architecture, "lfm2");
        assert_eq!(summary.name, "LFM Fixture");
        assert_eq!(summary.size_label, "2.7B");
        assert_eq!(summary.file_type, Some(7));
        assert_eq!(summary.block_count, Some(30));
        assert_eq!(summary.context_length, Some(131072));
        assert_eq!(summary.embedding_length, Some(2048));
        assert_eq!(
            summary.head_count_kv, None,
            "array-valued kv head counts are not a scalar"
        );
        assert_eq!(summary.rope_freq_base, Some(10_000_000.0));
        assert_eq!(summary.tensor_count, 266);
        assert_eq!(summary.kv_count, 9);
        assert_eq!(summary.expert_count, None);
    }

    #[test]
    fn rejects_non_gguf_files() {
        let error = parse(b"RIFF....".as_slice()).unwrap_err();
        assert!(error.to_string().contains("bad magic"));
    }

    #[test]
    fn reads_from_disk_without_touching_tensor_bytes() {
        let path = std::env::temp_dir().join(format!("gguf-pilot-hdr-{}.gguf", std::process::id()));
        let mut file = File::create(&path).unwrap();
        file.write_all(&fixture()).unwrap();
        drop(file);
        let summary = read_summary(&path).unwrap();
        assert_eq!(summary.block_count, Some(30));
        std::fs::remove_file(path).unwrap();
    }
}
