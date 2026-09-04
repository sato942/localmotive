# Future branding: Localmotive

Status: completed.
Date: 2026-09-04.
Move date: 2026-09-04.
Purpose: record the completed product and repository move.
Scope: product name and repository move. This file does not authorize a release.

## 1. Decision

The product name changed from GGUF Pilot to Localmotive.
Localmotive joins local and locomotive in one name.
A locomotive pulls heavy weight on fixed rails.
Localmotive pulls heavy models on local hardware.
The spelling gives a distinct search string.
The tagline is "Pull weight locally."

## 2. Reason for the change

GGUF Pilot names one model format.
The control plane must support future runtimes and future model formats.
Localmotive names local control and heavy pulling power.
Localmotive does not name any single format.
The five jobs in `AGENTS.md` stay unchanged.

## 3. Identity rules

Use Localmotive as the product name in prose.
Use LOCALMOTIVE for uppercase display text.
Use localmotive as the package slug in file names and keys.
Use LM as the two-letter brand mark.
Use `io.github.localmotive.app` as the bundle identifier.
Keep GGUF for the model format in prose and code.
Keep `llama-server` for the inference executable in prose and code.
Use the same term for the same concept in all files.

## 4. Visual direction

Keep the machine-room control cabinet in `DESIGN.md`.
Keep matte graphite panels and engraved plates.
Keep instrument-grade readouts and tabular figures.
Keep the zero-radius rule for rectangular controls.
Add locomotive cues only through language first.
Add plates such as RUNTIME, PROFILE, and MEASURE.
Do not add decorative railway illustration.
Do not add glow text or ambient animation.
If the team selects a locomotive mark, test the mark at 16 px before adoption.

## 5. Token map

Use this map for the mechanical rename.

- `GGUF Pilot` becomes `Localmotive`.
- `GGUF PILOT` becomes `LOCALMOTIVE`.
- `GGUF-Pilot` becomes `Localmotive`.
- `gguf-pilot` becomes `localmotive`.
- `gguf_pilot` becomes `localmotive`.
- `ggufpilot` becomes `localmotive`.
- `GGUF_PILOT` becomes `LOCALMOTIVE`.
- `GP` in the brand mark becomes `LM`.
- `gguf-pilot-structural-v1` becomes `localmotive-structural-v1`.
- `GGUF Pilot HF` becomes `Localmotive HF`.
- `%LOCALAPPDATA%\GGUF Pilot\runtimes` becomes `%LOCALAPPDATA%\Localmotive\runtimes`.
- `gguf-pilot` log and cache directory names become `localmotive`.

## 6. Exceptions

Do not rewrite historical release evidence.
Keep `CHANGELOG.md` entries for shipped releases unchanged.
Keep the 0.3.0 artifact names in `CHANGELOG.md` unchanged.
Keep the verification ledger in `TODO.md` for shipped builds unchanged.
Keep GGUF format references unchanged.

## 7. Compatibility

If a user upgrades, preserve credentials and data.
Read cloud keys from `Localmotive` first and from `GGUF Pilot` second.
Write cloud keys to `Localmotive` and remove the legacy entry.
Read the HF token from `Localmotive HF` first and from `GGUF Pilot HF` second.
Write the HF token to `Localmotive HF` and remove the legacy entry.
Resolve managed runtimes from the new directory first.
Fall back to the legacy directory when the new directory has no records.
Read frontend settings from the `localmotive:` prefix first.
Fall back to the `gguf-pilot:` prefix for each key.
Write frontend settings with the `localmotive:` prefix.
Keep old settings on disk to allow downgrade.
The bundle identifier changed from `io.github.ggufpilot.app` to `io.github.localmotive.app`.
Installed users may need to migrate Tauri cache and settings paths.
Catalog cache rebuilds from the bundled catalog when the old cache path misses.

## 8. Repository move

The repository moved from `sato942/gguf-pilot` to `sato942/localmotive`.
The catalog URL points at `sato942/localmotive`.
The `repository` fields point at `sato942/localmotive`.
These items moved together.

- `README.md` clone, issues, and releases links.
- `src-tauri/Cargo.toml` repository field.
- `src-tauri/src/catalog.rs` catalog URL.
- `scripts/build_catalog.mjs` source template.
- `src/App.tsx` repository fallback.
- OpenRouter referer header.
- Bundle identifier in `src-tauri/tauri.conf.json` and frontend fallback.
- Catalog signing variable `LOCALMOTIVE_CATALOG_SIGNING_KEY_PEM`.
- Catalog `source` template for future signed refreshes.
- Catalog `note` template for future signed refreshes.

The shipped `catalog/catalog.json` bytes still carry the old source and note.
A signed catalog refresh must replace the source and note together.
Do not edit `catalog/catalog.json` by hand.
Use `npm run catalog:refresh` with the signing key.

## 9. Open questions

Trademark clearance for Localmotive is unknown.
Domain availability for Localmotive is unknown.
Installer name acceptance on Windows is unknown.
Bundle identifier migration behavior on Windows is unknown.
Catalog re-signing after the note change is pending.
Human approval for signing and publication is required.

## 10. Next steps

Order trademark and domain review before public use.
Run a packaging trial before new installer names ship.
Refresh the signed catalog to update the catalog source and note.
Verify installation, launch, update, and uninstall on Windows.
Record evidence in `TODO.md` after each check.
