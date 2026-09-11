//! Deterministic test-support helpers for bounded property campaigns
//! (audit S-19). No external proptest/fuzz dependency: a fixed splitmix64
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
