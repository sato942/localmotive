// Canonical GGUF quantisation labels (audit DC-09).
//
// The catalog builder used to take the last "-SOMETHING.gguf" suffix as the
// quant label, which captured provenance text (imatrix, MTP, dates, it) as
// if it were a quantisation. This module reads the file's own quantisation
// token instead: the leftmost known token in the basename, matching the
// `quant_weight` precedence in core.rs. Prose around the token (vendor
// prefixes, `-from-BF16` provenance, shard ids) stays outside the label.

export const CANONICAL_QUANTS = [
  "F32",
  "BF16",
  "F16",
  "Q8_0",
  "Q6_K",
  "Q6_K_XL",
  "Q5_K_M",
  "Q5_K_S",
  "Q5_K_XL",
  "Q5_0",
  "Q5_1",
  "Q4_K_M",
  "Q4_K_S",
  "Q4_K_XL",
  "Q4_K_XXL",
  "Q4_0",
  "Q4_1",
  "IQ4_NL",
  "IQ4_XS",
  "IQ3_XXS",
  "IQ3_XS",
  "IQ3_S",
  "IQ3_M",
  "Q3_K_S",
  "Q3_K_M",
  "Q3_K_L",
  "Q3_K_XL",
  "IQ2_XXS",
  "IQ2_XS",
  "IQ2_S",
  "IQ2_M",
  "Q2_K",
  "Q2_K_XL",
  "MXFP4",
  "Q6_K_M",
  "Q8_K_L",
  "Q8_0_L",
  "Q2_K_M",
  "I2_S",
  "I8_S",
  "Q1_0",
  "Q4_0_4_4",
  "Q5_0_XL",
  "IQ4_NL_L",
  "IQ1_S",
  "IQ1_M",
  "Q8_K_XL",
  "Q6_K_L",
  "Q5_K_L",
  "Q4_K_L",
  "Q2_K_L",
  "Q4_K",
  "Q5_K",
  "Q3_K",
  "Q8_K",

  "MXFP4_MOE",
  "TQ1_0",
  "TQ2_0",
];

const CANONICAL_BY_UPPER = new Map(CANONICAL_QUANTS.map((label) => [label.toUpperCase(), label]));

/// Extract the file's own quantisation label, or "UNKNOWN" when the name
/// carries no recognised token (audit DC-09).
export function quantFromFilename(filename) {
  const stem = String(filename).replace(/\.gguf$/i, "");
  // Split on '-', '.', and whitespace: '_' joins a quant token (Q4_K_M),
  // while dotted names (`model.q8_0.gguf`) separate on '.'.
  const tokens = stem.split(/[-.\s]+/).filter((token) => token.length > 0);
  for (const token of tokens) {
    // A token may glue an id to a quant (`E4B_q4_0`): try it whole and then
    // its '_'-joined suffixes, so the quant token still wins over prose.
    const parts = token.split("_");
    const candidates = [token];
    for (let start = 1; start < parts.length; start += 1) {
      candidates.push(parts.slice(start).join("_"));
    }
    for (const candidate of candidates) {
      const upper = candidate.toUpperCase();
      if (CANONICAL_BY_UPPER.has(upper)) {
        return CANONICAL_BY_UPPER.get(upper);
      }
      // Unsloth-style dynamic quants: "UD-Q4_K_XL" names the same token with
      // a UD marker token in front.
      if (upper === "UD") {
        continue;
      }
    }
    if (token.toUpperCase() === "UD") {
      continue;
    }
  }
  // "UD" marker followed by the token in the next candidate position.
  for (let index = 0; index < tokens.length; index += 1) {
    if (tokens[index].toUpperCase() !== "UD") continue;
    for (let offset = 1; offset <= 2 && index + offset < tokens.length; offset += 1) {
      const upper = tokens[index + offset].toUpperCase();
      if (CANONICAL_BY_UPPER.has(upper)) {
        return CANONICAL_BY_UPPER.get(upper);
      }
    }
  }
  return "UNKNOWN";
}

/// True when a label may appear as a catalog quant (audit DC-09 curation).
export function isCanonicalQuant(label) {
  return CANONICAL_BY_UPPER.has(String(label).toUpperCase());
}
