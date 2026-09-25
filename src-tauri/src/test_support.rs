//! Deterministic test-support helpers for bounded property campaigns
//!. No external proptest/fuzz dependency: a fixed splitmix64
//! generator keeps every campaign reproducible from its seed alone, and each
//! property test reports the failing seed in its panic message.
#![cfg(test)]

/// splitmix64: tiny, deterministic, good enough for input generation.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            return 0;
        }
        (self.next_u64() % bound as u64) as usize
    }

    pub fn boolean(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }

    /// A byte with no structure: the basis of malformed-input campaigns.
    pub fn byte(&mut self) -> u8 {
        (self.next_u64() & 0xFF) as u8
    }

    pub fn bytes(&mut self, max_len: usize) -> Vec<u8> {
        let len = self.below(max_len + 1);
        (0..len).map(|_| self.byte()).collect()
    }

    /// Mostly-ASCII text with occasional Unicode, braces and quotes, so the
    /// parser campaigns see realistic hostile shapes and not only noise.
    pub fn text(&mut self, max_len: usize) -> String {
        const POOL: [&str; 20] = [
            "a", "B", "9", " ", "{", "}", "\"", "'", ",", ":", "[", "]", "\n", "\\", "-", "_", "é",
            "模", "\u{2028}", "µ",
        ];
        let len = self.below(max_len + 1);
        (0..len)
            .map(|_| POOL[self.below(POOL.len())])
            .collect::<String>()
    }
}

/// Run a bounded property campaign over seeds `1..=iterations`, reporting the
/// failing seed in the panic message so the regression is reproducible.
pub fn campaign(iterations: u64, mut check: impl FnMut(u64, &mut Rng)) {
    for seed in 1..=iterations {
        let mut rng = Rng::new(seed);
        check(seed, &mut rng);
    }
}

static TEMP_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Isolated temp dir for tests: PID-only names collide across recycled PIDs
/// and leftover trees, so every call mints a fresh directory.
pub fn unique_temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);
    let root = std::env::temp_dir().join(format!(
        "localmotive-{label}-{}-{nanos}-{}",
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&root).expect("test temp dir setup failed");
    root
}

mod tests {
    use super::*;

    #[test]
    fn temp_dirs_are_unique_per_call() {
        let first = unique_temp_dir("unique");
        let second = unique_temp_dir("unique");
        assert_ne!(first, second);
        assert!(first.is_dir() && second.is_dir());
        let _ = std::fs::remove_dir_all(first);
        let _ = std::fs::remove_dir_all(second);
    }
}
