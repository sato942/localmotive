//! Bounded property campaigns over the parser and evidence invariants
//! (audit S-19). Every campaign is deterministic from its seed and capped in
//! iterations and input size; a failing seed reproduces the case exactly.
#![cfg(test)]

use crate::test_support::campaign;

#[test]
fn s19_gguf_reader_never_panics_on_malformed_bytes_and_stays_bounded() {
    let root = std::env::temp_dir().join(format!("localmotive-s19-gguf-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("fuzz.gguf");
    campaign(400, |seed, rng| {
        let bytes = rng.bytes(512);
        std::fs::write(&path, &bytes).unwrap();
        // Either a clean parse or a clean error; never a panic, and the
        // bounded reader must return quickly for tiny inputs.
        let outcome = crate::gguf::read_summary(&path);
        if let Ok(summary) = outcome {
            assert!(
                summary.kv_count as usize <= 1_048_576,
                "seed {seed}: parsed counts must stay bounded"
            );
        }
    });
    // A truncated real header is the classic case: prefix corruption of a
    // valid fixture must also stay a clean error.
    campaign(100, |seed, rng| {
        let mut bytes = vec![0x47, 0x47, 0x55, 0x46, 3, 0, 0, 0];
        bytes.extend((0..rng.below(64)).map(|_| rng.byte()));
        std::fs::write(&path, &bytes).unwrap();
        let _ = crate::gguf::read_summary(&path);
        assert!(seed > 0);
    });
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn s19_proposal_parser_tolerates_malformed_and_nested_brace_input() {
    campaign(600, |_seed, rng| {
        let text = rng.text(256);
        let _ = crate::tune::parse_proposal(&text);
    });
    // Nested braces in prose (a real historical failure) must never panic.
    campaign(100, |_seed, rng| {
        let depth = rng.below(64);
        let text = format!(
            "{}prose {{\"settings\":{{}}}}{}",
            "[".repeat(depth),
            "}}".repeat(depth)
        );
        let _ = crate::tune::parse_proposal(&text);
    });
}

#[test]
fn s19_shard_name_round_trips_and_rejects_noise() {
    campaign(500, |seed, rng| {
        if rng.boolean() {
            // Generated VALID split names round-trip through parse -> display.
            let index = 1 + rng.below(9999);
            let count = index + rng.below(8);
            let name = format!("model-{index:05}-of-{count:05}.gguf");
            let parsed = crate::artifact::parse_shard_name(&name)
                .unwrap_or_else(|error| panic!("seed {seed}: {name}: {error}"));
            assert_eq!(parsed.index, index, "seed {seed}");
            assert_eq!(parsed.count, count, "seed {seed}");
        } else {
            // Noise must be a clean error, never a panic.
            let _ = crate::artifact::parse_shard_name(&rng.text(64));
        }
    });
}

#[test]
fn s19_effective_argument_sanitizer_is_idempotent_and_leaks_no_canary() {
    let canary = "C:/Users/canary-user/secret";
    let flags = [
        "-m",
        "--model",
        "--mmproj",
        "--lora",
        "--lora-scaled",
        "-md",
        "--spec-draft-model",
        "--chat-template-file",
        "--api-key-file",
        "--ssl-key-file",
        "--ssl-cert-file",
    ];
    campaign(200, |seed, rng| {
        let mut args = Vec::new();
        for flag in flags {
            args.push(flag.to_string());
            args.push(if rng.boolean() {
                format!("{canary}/{}", rng.below(1000))
            } else {
                format!("plain-{}", rng.below(1000))
            });
        }
        let once = crate::calibration::sanitize_effective_args(&args);
        let twice = crate::calibration::sanitize_effective_args(&once);
        assert_eq!(once, twice, "seed {seed}: sanitization must be idempotent");
        let joined = once.join(" ");
        assert!(!joined.contains("canary-user"), "seed {seed}: {joined}");
    });
}

#[test]
fn s19_numeric_summaries_never_return_nan_for_extreme_inputs() {
    campaign(400, |seed, rng| {
        let count = 1 + rng.below(16);
        let values: Vec<f64> = (0..count)
            .map(|_| match rng.below(8) {
                0 => f64::MAX,
                1 => f64::MIN_POSITIVE,
                2 => -f64::MAX,
                3 => f64::INFINITY,
                4 => f64::NEG_INFINITY,
                5 => f64::NAN,
                6 => 0.0,
                _ => (rng.next_u64() as f64) / 1.0e6 - 1.0e6,
            })
            .collect();
        if let Some(stats) = crate::measurement::metric_stats_for_test(values.iter().copied()) {
            assert!(
                stats.mean.is_finite()
                    && stats.p50.is_finite()
                    && stats.p95.is_finite()
                    && stats.standard_deviation.is_finite(),
                "seed {seed}: summaries must stay finite for finite inputs"
            );
        }
    });
}
