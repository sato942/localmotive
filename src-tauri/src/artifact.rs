use crate::evidence::EvidenceLevel;
use crate::gguf::{self, GgufSummary};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

const HASH_BUFFER_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShardName {
    pub logical_name: String,
    pub index: usize,
    pub count: usize,
    pub split: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactProblemCode {
    MissingShard,
    DuplicateShard,
    ConflictingHeader,
    UnreadableHeader,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactProblem {
    pub code: ArtifactProblemCode,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShardSet {
    pub logical_name: String,
    pub expected_shards: usize,
    pub ordered: Vec<ShardName>,
    pub complete: bool,
    pub problems: Vec<ArtifactProblem>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactFileFact {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub sha256: Option<String>,
    pub header_sha256: Option<String>,
    pub shard_index: Option<usize>,
    pub expected_shards: Option<usize>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactInspection {
    pub logical_id: String,
    pub content_id: Option<String>,
    pub logical_name: String,
    pub first_shard: String,
    pub expected_shards: usize,
    pub complete: bool,
    pub header_consistent: bool,
    pub identity_level: EvidenceLevel,
    pub shard_bytes: u64,
    pub companion_bytes: u64,
    pub shards: Vec<ArtifactFileFact>,
    pub companions: Vec<ArtifactFileFact>,
    pub summary: Option<GgufSummary>,
    pub problems: Vec<ArtifactProblem>,
}

#[derive(PartialEq)]
struct HeaderIdentity<'a> {
    version: u32,
    architecture: &'a str,
    name: &'a str,
    file_type: Option<u32>,
    block_count: Option<u64>,
    context_length: Option<u64>,
    embedding_length: Option<u64>,
    head_count: Option<u64>,
    head_count_kv: Option<u64>,
    key_length: Option<u64>,
    value_length: Option<u64>,
    expert_count: Option<u64>,
    expert_used_count: Option<u64>,
}

impl<'a> From<&'a GgufSummary> for HeaderIdentity<'a> {
    fn from(summary: &'a GgufSummary) -> Self {
        Self {
            version: summary.version,
            architecture: &summary.architecture,
            name: &summary.name,
            file_type: summary.file_type,
            block_count: summary.block_count,
            context_length: summary.context_length,
            embedding_length: summary.embedding_length,
            head_count: summary.head_count,
            head_count_kv: summary.head_count_kv,
            key_length: summary.key_length,
            value_length: summary.value_length,
            expert_count: summary.expert_count,
            expert_used_count: summary.expert_used_count,
        }
    }
}

pub fn parse_shard_name(name: &str) -> Result<ShardName, String> {
    let Some((stem, extension)) = name.rsplit_once('.') else {
        return Err("Artifact file must have a .gguf extension".into());
    };
    if !extension.eq_ignore_ascii_case("gguf") {
        return Err("Artifact file must have a .gguf extension".into());
    }
    let Some(of_position) = stem.rfind("-of-") else {
        return Ok(ShardName {
            logical_name: stem.to_string(),
            index: 1,
            count: 1,
            split: false,
        });
    };
    let count_text = &stem[of_position + 4..];
    let before = &stem[..of_position];
    let Some(index_dash) = before.rfind('-') else {
        return Err("Split GGUF name is missing its shard index".into());
    };
    let index_text = &before[index_dash + 1..];
    let logical_name = &before[..index_dash];
    let valid_number =
        |value: &str| value.len() == 5 && value.bytes().all(|byte| byte.is_ascii_digit());
    if logical_name.is_empty() || !valid_number(index_text) || !valid_number(count_text) {
        return Err("Split GGUF name must end with -00001-of-00001.gguf".into());
    }
    let index = index_text
        .parse::<usize>()
        .map_err(|_| "Split GGUF shard index is invalid")?;
    let count = count_text
        .parse::<usize>()
        .map_err(|_| "Split GGUF shard count is invalid")?;
    if index == 0 || count == 0 || index > count {
        return Err("Split GGUF shard index must be between one and the shard count".into());
    }
    Ok(ShardName {
        logical_name: logical_name.to_string(),
        index,
        count,
        split: true,
    })
}

pub fn analyze_shard_names(names: &[String]) -> Result<ShardSet, String> {
    let first_name = names.first().ok_or("A shard set cannot be empty")?;
    let first = parse_shard_name(first_name)?;
    let mut by_index = BTreeMap::new();
    let mut problems = Vec::new();
    for name in names {
        let shard = parse_shard_name(name)?;
        if shard.logical_name != first.logical_name || shard.count != first.count {
            return Err("Shard names do not describe one logical model".into());
        }
        let index = shard.index;
        if by_index.insert(index, shard).is_some() {
            problems.push(ArtifactProblem {
                code: ArtifactProblemCode::DuplicateShard,
                message: format!("Duplicate shard index {index:05}"),
            });
        }
    }
    let missing: Vec<String> = (1..=first.count)
        .filter(|index| !by_index.contains_key(index))
        .map(|index| format!("{index:05}"))
        .collect();
    if !missing.is_empty() {
        problems.push(ArtifactProblem {
            code: ArtifactProblemCode::MissingShard,
            message: format!("Missing shard indices: {}", missing.join(", ")),
        });
    }
    let ordered = by_index.into_values().collect::<Vec<_>>();
    Ok(ShardSet {
        logical_name: first.logical_name,
        expected_shards: first.count,
        complete: problems.is_empty() && ordered.len() == first.count,
        ordered,
        problems,
    })
}

#[cfg(windows)]
pub(crate) fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
pub(crate) fn is_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}

pub(crate) fn validate_no_reparse_ancestors(label: &str, path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err(format!("{label} path is empty"));
    }
    for component_path in path
        .ancestors()
        .filter(|entry| !entry.as_os_str().is_empty())
    {
        let component_metadata = match fs::symlink_metadata(component_path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "Could not inspect {label} path component {}: {error}",
                    component_path.display()
                ));
            }
        };
        if component_metadata.file_type().is_symlink() || is_reparse_point(&component_metadata) {
            return Err(format!(
                "{label} path contains a symlink or reparse-point ancestor: {}",
                component_path.display()
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_regular_non_reparse_file(label: &str, path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err(format!("{label} path is empty"));
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("{label} does not exist: {} ({error})", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("{label} is not a regular file: {}", path.display()));
    }
    validate_no_reparse_ancestors(label, path)
}

pub fn sha256_path(path: &Path) -> Result<String, String> {
    let link_metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("Could not inspect {}: {error}", path.display()))?;
    if link_metadata.file_type().is_symlink() || is_reparse_point(&link_metadata) {
        return Err(format!(
            "Artifact path cannot be a symlink or reparse point: {}",
            path.display()
        ));
    }
    if !link_metadata.is_file() {
        return Err(format!("Artifact path is not a file: {}", path.display()));
    }
    let mut file =
        File::open(path).map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    let before = file
        .metadata()
        .map_err(|error| format!("Could not inspect open file {}: {error}", path.display()))?;
    let before_modified = before.modified().ok();
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; HASH_BUFFER_BYTES];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let after = file
        .metadata()
        .map_err(|error| format!("Could not recheck open file {}: {error}", path.display()))?;
    if before.len() != after.len() || before_modified != after.modified().ok() {
        return Err(format!(
            "Artifact changed while hashing: {}",
            path.display()
        ));
    }
    Ok(hex::encode(hasher.finalize()))
}

fn sha256_prefix_path(path: &Path, byte_count: u64) -> Result<String, String> {
    let mut file =
        File::open(path).map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    let mut remaining = byte_count;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; HASH_BUFFER_BYTES];
    while remaining > 0 {
        let capacity = buffer.len();
        let requested = usize::try_from(remaining).unwrap_or(capacity).min(capacity);
        let read = file
            .read(&mut buffer[..requested])
            .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
        if read == 0 {
            return Err(format!(
                "GGUF header ended before {byte_count} bytes: {}",
                path.display()
            ));
        }
        hasher.update(&buffer[..read]);
        remaining -= read as u64;
    }
    Ok(hex::encode(hasher.finalize()))
}

fn artifact_file_fact(
    path: &Path,
    shard: Option<&ShardName>,
    summary: &GgufSummary,
    hash_file: bool,
) -> Result<ArtifactFileFact, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("Could not inspect {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) || !metadata.is_file() {
        return Err(format!(
            "Artifact must be a regular non-reparse file: {}",
            path.display()
        ));
    }
    Ok(ArtifactFileFact {
        path: path.to_string_lossy().to_string(),
        name: path
            .file_name()
            .ok_or_else(|| format!("Artifact has no file name: {}", path.display()))?
            .to_string_lossy()
            .to_string(),
        size_bytes: metadata.len(),
        sha256: hash_file.then(|| sha256_path(path)).transpose()?,
        header_sha256: Some(sha256_prefix_path(path, summary.header_bytes)?),
        shard_index: shard.map(|value| value.index),
        expected_shards: shard.map(|value| value.count),
    })
}

fn logical_id(
    logical_name: &str,
    expected_shards: usize,
    shards: &[ArtifactFileFact],
    companions: &[ArtifactFileFact],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(logical_name.as_bytes());
    hasher.update([0]);
    hasher.update(expected_shards.to_le_bytes());
    for file in shards.iter().chain(companions) {
        hasher.update(file.name.as_bytes());
        hasher.update([0]);
        hasher.update(file.size_bytes.to_le_bytes());
        if let Some(digest) = file.header_sha256.as_deref() {
            hasher.update(digest.as_bytes());
        }
    }
    hex::encode(hasher.finalize())
}

fn content_id(shards: &[ArtifactFileFact], companions: &[ArtifactFileFact]) -> Option<String> {
    let mut hasher = Sha256::new();
    for file in shards.iter().chain(companions) {
        let digest = file.sha256.as_deref()?;
        hasher.update(file.name.as_bytes());
        hasher.update([0]);
        hasher.update(file.size_bytes.to_le_bytes());
        hasher.update(digest.as_bytes());
    }
    Some(hex::encode(hasher.finalize()))
}

pub fn inspect_artifact(
    selected_shard: &Path,
    companion_paths: &[std::path::PathBuf],
    hash_files: bool,
) -> Result<ArtifactInspection, String> {
    let selected_name = selected_shard
        .file_name()
        .ok_or_else(|| format!("Artifact has no file name: {}", selected_shard.display()))?
        .to_string_lossy()
        .to_string();
    let selected = parse_shard_name(&selected_name)?;
    let directory = selected_shard
        .parent()
        .ok_or_else(|| format!("Artifact has no parent: {}", selected_shard.display()))?;
    let mut paths_by_index = BTreeMap::new();
    let mut names = Vec::new();
    for entry in fs::read_dir(directory)
        .map_err(|error| format!("Could not read {}: {error}", directory.display()))?
    {
        let path = entry.map_err(|error| error.to_string())?.path();
        let Some(name) = path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
        else {
            continue;
        };
        let Ok(shard) = parse_shard_name(&name) else {
            continue;
        };
        if shard.logical_name == selected.logical_name {
            names.push(name);
            paths_by_index.insert(shard.index, path);
        }
    }
    let shard_set = analyze_shard_names(&names)?;
    let mut problems = shard_set.problems.clone();
    let mut summaries = Vec::new();
    let mut shards = Vec::new();
    for shard in &shard_set.ordered {
        let Some(path) = paths_by_index.get(&shard.index) else {
            continue;
        };
        match gguf::read_summary(path) {
            Ok(summary) => {
                shards.push(artifact_file_fact(path, Some(shard), &summary, hash_files)?);
                summaries.push(summary);
            }
            Err(error) => problems.push(ArtifactProblem {
                code: ArtifactProblemCode::UnreadableHeader,
                message: error,
            }),
        }
    }
    let header_consistent = summaries.first().is_some_and(|first| {
        summaries
            .iter()
            .all(|item| HeaderIdentity::from(item) == HeaderIdentity::from(first))
    });
    if !summaries.is_empty() && !header_consistent {
        problems.push(ArtifactProblem {
            code: ArtifactProblemCode::ConflictingHeader,
            message: "Shard headers disagree on required model identity".into(),
        });
    }
    let mut companions = Vec::new();
    for path in companion_paths {
        let summary = gguf::read_summary(path)?;
        companions.push(artifact_file_fact(path, None, &summary, hash_files)?);
    }
    let shard_bytes = sum_file_bytes(&shards, "shard")?;
    let companion_bytes = sum_file_bytes(&companions, "companion")?;
    let first_shard = paths_by_index
        .get(&1)
        .map_or(selected_shard, |path| path.as_path())
        .to_string_lossy()
        .to_string();
    let id = logical_id(
        &shard_set.logical_name,
        shard_set.expected_shards,
        &shards,
        &companions,
    );
    let content_id = content_id(&shards, &companions);
    Ok(ArtifactInspection {
        logical_id: id,
        content_id,
        logical_name: shard_set.logical_name,
        first_shard,
        expected_shards: shard_set.expected_shards,
        complete: shard_set.complete && header_consistent && problems.is_empty(),
        header_consistent,
        identity_level: EvidenceLevel::Derived,
        shard_bytes,
        companion_bytes,
        shards,
        companions,
        summary: summaries.into_iter().next(),
        problems,
    })
}

fn sum_file_bytes(files: &[ArtifactFileFact], label: &str) -> Result<u64, String> {
    files.iter().try_fold(0_u64, |total, file| {
        total
            .checked_add(file.size_bytes)
            .ok_or_else(|| format!("{label} byte total overflowed the supported range"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn artifact_byte_totals_reject_overflow() {
        let fact = |size_bytes| ArtifactFileFact {
            path: "fixture.gguf".into(),
            name: "fixture.gguf".into(),
            size_bytes,
            sha256: None,
            header_sha256: None,
            shard_index: None,
            expected_shards: None,
        };

        let error = sum_file_bytes(&[fact(u64::MAX), fact(1)], "shard").unwrap_err();

        assert!(error.contains("shard"));
        assert!(error.contains("overflow"));
    }

    fn put_string(bytes: &mut Vec<u8>, value: &str) {
        bytes.extend((value.len() as u64).to_le_bytes());
        bytes.extend(value.as_bytes());
    }

    fn put_string_fact(bytes: &mut Vec<u8>, key: &str, value: &str) {
        put_string(bytes, key);
        bytes.extend(8_u32.to_le_bytes());
        put_string(bytes, value);
    }

    fn put_u32_fact(bytes: &mut Vec<u8>, key: &str, value: u32) {
        put_string(bytes, key);
        bytes.extend(4_u32.to_le_bytes());
        bytes.extend(value.to_le_bytes());
    }

    fn write_minimal_gguf(path: &Path, architecture: &str, split_index: u32, split_count: u32) {
        let mut bytes = Vec::new();
        bytes.extend(b"GGUF");
        bytes.extend(3_u32.to_le_bytes());
        bytes.extend(0_u64.to_le_bytes());
        bytes.extend(6_u64.to_le_bytes());
        put_string_fact(&mut bytes, "general.architecture", architecture);
        put_string_fact(&mut bytes, "general.name", "Fixture");
        put_u32_fact(&mut bytes, "general.file_type", 7);
        put_u32_fact(&mut bytes, &format!("{architecture}.block_count"), 2);
        put_u32_fact(&mut bytes, "split.no", split_index);
        put_u32_fact(&mut bytes, "split.count", split_count);
        bytes.extend(format!("PAYLOAD-{split_index}").as_bytes());
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn parses_standard_shard_identity() {
        let shard = parse_shard_name("Model-Q4_K_M-00002-of-00004.gguf").unwrap();

        assert_eq!(shard.logical_name, "Model-Q4_K_M");
        assert_eq!(shard.index, 2);
        assert_eq!(shard.count, 4);
        assert!(shard.split);
    }

    #[test]
    fn parses_a_unicode_single_file_name_without_byte_slicing() {
        let shard = parse_shard_name("Battlefield™ 6.gguf").unwrap();

        assert_eq!(shard.logical_name, "Battlefield™ 6");
        assert_eq!(shard.count, 1);
        assert!(!shard.split);
    }

    #[test]
    fn reports_a_gap_instead_of_counting_files_as_complete() {
        let names = vec![
            "Model-Q4-00001-of-00003.gguf".to_string(),
            "Model-Q4-00003-of-00003.gguf".to_string(),
        ];

        let set = analyze_shard_names(&names).unwrap();

        assert!(!set.complete);
        assert_eq!(set.expected_shards, 3);
        assert_eq!(set.ordered[0].index, 1);
        assert_eq!(set.ordered[1].index, 3);
        assert_eq!(set.problems[0].code, ArtifactProblemCode::MissingShard);
        assert!(set.problems[0].message.contains("00002"));
    }

    #[test]
    fn reports_duplicate_shard_indices() {
        let names = vec![
            "Model-Q4-00001-of-00002.gguf".to_string(),
            "Model-Q4-00001-of-00002.GGUF".to_string(),
            "Model-Q4-00002-of-00002.gguf".to_string(),
        ];

        let set = analyze_shard_names(&names).unwrap();

        assert!(!set.complete);
        assert!(set
            .problems
            .iter()
            .any(|problem| problem.code == ArtifactProblemCode::DuplicateShard));
    }

    #[test]
    fn streams_sha256_without_loading_the_artifact() {
        let directory =
            std::env::temp_dir().join(format!("localmotive-artifact-hash-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("fixture.gguf");
        fs::write(&path, b"abc").unwrap();

        let digest = sha256_path(&path).unwrap();

        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn inspects_a_complete_split_artifact_with_exact_file_facts() {
        let directory = std::env::temp_dir().join(format!(
            "localmotive-artifact-inspect-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let first = directory.join("Fixture-Q4-00001-of-00002.gguf");
        let second = directory.join("Fixture-Q4-00002-of-00002.gguf");
        write_minimal_gguf(&first, "llama", 0, 2);
        write_minimal_gguf(&second, "llama", 1, 2);

        let inspected = inspect_artifact(&second, &[], true).unwrap();

        assert!(inspected.complete);
        assert!(inspected.header_consistent);
        assert_eq!(inspected.first_shard, first.to_string_lossy());
        assert_eq!(inspected.shards.len(), 2);
        assert_eq!(inspected.shards[0].shard_index, Some(1));
        assert_eq!(inspected.shards[1].shard_index, Some(2));
        assert_eq!(
            inspected.shard_bytes,
            fs::metadata(&first).unwrap().len() * 2
        );
        assert_eq!(inspected.companion_bytes, 0);
        assert!(inspected
            .shards
            .iter()
            .all(|file| file.sha256.as_ref().is_some_and(|hash| hash.len() == 64)));
        assert_eq!(inspected.summary.as_ref().unwrap().architecture, "llama");
        assert_eq!(inspected.logical_id.len(), 64);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn logical_identity_is_stable_when_full_hashes_are_requested() {
        let directory = std::env::temp_dir().join(format!(
            "localmotive-artifact-identity-mode-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let model = directory.join("Fixture-Q4-00001-of-00001.gguf");
        write_minimal_gguf(&model, "llama", 0, 1);

        let header_only = inspect_artifact(&model, &[], false).unwrap();
        let fully_hashed = inspect_artifact(&model, &[], true).unwrap();

        assert_eq!(header_only.logical_id, fully_hashed.logical_id);
        assert_eq!(header_only.identity_level, EvidenceLevel::Derived);
        assert_eq!(fully_hashed.identity_level, EvidenceLevel::Derived);
        assert!(header_only.content_id.is_none());
        assert!(fully_hashed
            .content_id
            .as_ref()
            .is_some_and(|digest| digest.len() == 64));
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn rejects_conflicting_required_headers_across_shards() {
        let directory = std::env::temp_dir().join(format!(
            "localmotive-artifact-conflict-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let first = directory.join("Fixture-Q4-00001-of-00002.gguf");
        let second = directory.join("Fixture-Q4-00002-of-00002.gguf");
        write_minimal_gguf(&first, "llama", 0, 2);
        write_minimal_gguf(&second, "qwen2", 1, 2);

        let inspected = inspect_artifact(&first, &[], false).unwrap();

        assert!(!inspected.complete);
        assert!(!inspected.header_consistent);
        assert!(inspected
            .problems
            .iter()
            .any(|problem| problem.code == ArtifactProblemCode::ConflictingHeader));
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn keeps_companion_file_facts_and_bytes_separate() {
        let directory = std::env::temp_dir().join(format!(
            "localmotive-artifact-companion-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let model = directory.join("Fixture-Q4.gguf");
        let companion = directory.join("mmproj-Fixture-F16.gguf");
        write_minimal_gguf(&model, "llama", 0, 1);
        write_minimal_gguf(&companion, "clip", 0, 1);

        let inspected = inspect_artifact(&model, std::slice::from_ref(&companion), true).unwrap();

        assert!(inspected.complete);
        assert_eq!(inspected.shards.len(), 1);
        assert_eq!(inspected.companions.len(), 1);
        assert_eq!(
            inspected.companion_bytes,
            fs::metadata(&companion).unwrap().len()
        );
        assert_eq!(inspected.companions[0].shard_index, None);
        assert_eq!(inspected.companions[0].name, "mmproj-Fixture-F16.gguf");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn reports_an_unreadable_shard_header_without_claiming_completeness() {
        let directory = std::env::temp_dir().join(format!(
            "localmotive-artifact-unreadable-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let first = directory.join("Fixture-Q4-00001-of-00002.gguf");
        let second = directory.join("Fixture-Q4-00002-of-00002.gguf");
        write_minimal_gguf(&first, "llama", 0, 2);
        fs::write(&second, b"not a GGUF file").unwrap();

        let inspected = inspect_artifact(&first, &[], false).unwrap();

        assert!(!inspected.complete);
        assert!(inspected
            .problems
            .iter()
            .any(|problem| problem.code == ArtifactProblemCode::UnreadableHeader));
        fs::remove_dir_all(directory).unwrap();
    }
}
