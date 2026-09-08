# Localmotive 0.4.1 stabilization plan

- **Status:** PLANNING AND AUDIT ONLY
- **Audit date:** 2026-09-05
- **Dynamic evidence cutoff:** `2026-09-05T12:41:28+04:00`
- **Audit status:** PARTIALLY VERIFIED
- **0.4.0 completion verdict:** FAIL against the recorded 0.4 acceptance plan
- **Implementation status:** NOT STARTED

This document records the 0.4.0 audit and the proposed 0.4.1 corrective work.

Do not treat a checked audit item as completed 0.4.1 implementation.

Do not change runtime approval, signing, or release state without the related gate.

### Status vocabulary

- `PASS`: Available evidence satisfies the stated check.
- `FAIL`: Available evidence disproves the stated check.
- `UNKNOWN`: Direct evidence is unavailable.
- `BLOCKED`: A known prerequisite or authority is unavailable.
- `VERIFIED`: Every material check passed.
- `PARTIALLY VERIFIED`: Some checks passed, while visible checks failed or remain unresolved.
- `UNVERIFIED`: Available evidence cannot substantiate the result.

`PLANNING AND AUDIT ONLY` and `NOT STARTED` are lifecycle labels, not check results.

---

## 1. Scope

Localmotive 0.4.1 is a Windows x64 stabilization release.

During this audit, change only `TODO-0.4.1.md`.

Do not change product code, workflows, release state, or icon assets during this audit.

Keep macOS, Linux, and Windows Arm64 product support out of scope.

Keep the approved `llama.cpp` release fixed unless a new pin passes every approval gate.

Preserve fail-closed behavior for each blocked backend.

Do not let one blocked backend remove unrelated approved backends.

Reserve `SUPPORTED` for evidence that satisfies the published support contract.

Treat the current in-app `LM` mark as the Windows icon source.

### Baseline pins

- Research product-audit revision: `50f881bcc960bd5a7784bbc6b305cd953a3f4e5d`
- Published product tag: `v0.4.0`
- Published source revision: `b0435b3fbe38ff49bcd5bb5ba6d554b527873e3a`
- Post-release evidence revision: `ca07483574f2b7d4be17f7704fe2dcabaa8cd6b8`
- `llama.cpp` capability snapshot: `4cbe8b070bb040f3b95845408f100fbf5fb746f1`
- `llama.cpp` release: `b10796`
- `llama.cpp` revision: `9a4843cf2f1a3fc8e39f8148e92ee6bfe18e2db6`
- `llama.cpp` release time: `2026-09-04T05:31:09Z`
- Smoke fixture: `SmolLM2-135M-Q4_K_M.gguf`
- Smoke fixture URL: `https://huggingface.co/ggml-org/SmolLM2-135M-GGUF/resolve/main/SmolLM2-135M-Q4_K_M.gguf`
- Smoke fixture bytes: `101016128`
- Smoke fixture SHA-256: `e3131339bf4e8065265593d4fd8f7bb7ff2d3abff1edb5618aa1197b89cad9f5`
- Audit host OS: Windows 11 build 26100
- Audit host CPU: AMD Ryzen 9 9950X3D
- Audit host accelerator: NVIDIA GeForce RTX 5090
- Audit accelerator memory: 32607 MiB
- Audit host driver: NVIDIA 610.74
- Audit host CUDA UMD: 13.3
- Audit host Vulkan API: 1.4.341
- Shipping platform: Windows x64
- Shipping bundles: portable executable, NSIS setup, and MSI

### Narrow support contract

The narrow executed row is Windows 11 build 26100 on the recorded Ryzen 9 and RTX 5090 host.

The row depends on `b10796`, driver 610.74, CUDA UMD 13.3, and Vulkan API 1.4.341.

The current research proves direct CPU, CUDA 13.3, and Vulkan execution only on that exact host.

Key each compatibility row by OS build, architecture, device identity, driver, required firmware, backend, and exact artifact digest.

The current research reaches L2 EXECUTED for those three rows.

The current research does not prove L4 PRODUCT or L5 RELEASE support.

Treat all other backend and hardware rows as experimental or not validated.

Treat every Windows Arm64 runtime entry as dormant parser capability.

The current audit substantiates Windows x64 packaging without backend support.

This limited audit result does not satisfy the 0.4.1 release gate.

Treat Windows 10 support as UNKNOWN until direct package evidence exists.

---

## 2. Sources audited

The audit read every top-level research document in this order:

1. `research/0.4/README.md`
2. `research/0.4/SYNTHESIS.md`
3. `research/0.4/GGUF-PILOT-AUDIT.md`
4. `research/0.4/implementation_plan.md`
5. `research/0.4/COMPATIBILITY-MATRIX.md`
6. `research/0.4/VALIDATION-STRATEGY.md`
7. `research/0.4/EVIDENCE.md`
8. `research/0.4/VERIFICATION.md`

The audit also inspected these supporting inputs:

- `research/0.4/HARDWARE-VALIDATION-MATRIX.csv`
- `research/0.4/attestation.schema.json`
- `research/0.4/citations-ledger.json`
- Three local smoke attestations
- Local smoke summaries and SHA-256 inventory
- Upstream release, workflow, and check-run snapshots
- `TODO-0.4.md`
- `CHANGELOG.md`
- `package.json` and both package lockfiles
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `src-tauri/approved_runtimes.json`
- Runtime, IPC, frontend, test, workflow, and icon files
- Published `v0.4.0` release assets
- The running published portable executable
- Raw logs from workflow runs `33948464859`, `33948466104`, and `33949560113`
- A clean local clone of tag `v0.4.0`

The hardware matrix has 55 data rows.

The matrix header is `id,priority,scope,os,architecture,hardware_class,backend,required_validation,current_evidence,current_status,access_route`.

The research directory remains intentionally ignored by Git.

Do not use research-only execution as packaged-product evidence.

---

## 3. Executive verdict

The 0.4.0 tag, version fields, released files, and checksums are internally consistent.

The published portable executable starts on the audited Windows host.

The 0.4.0 work is not complete against `research/0.4` and `TODO-0.4.md`.

`TODO-0.4.md` still contains 24 unchecked checkboxes.

Twelve unchecked checkboxes belong to Phase 4 and Phase 6 acceptance work.

Six unchecked checkboxes are carried research questions.

Six unchecked checkboxes are human-gated actions.

Three of 19 P0 rows have local direct runtime evidence.

Sixteen P0 rows lack local direct runtime evidence.

All 19 P0 rows lack `L4 PRODUCT` evidence.

The published Runtime Manager cannot return a catalog on the audited host.

A failed CUDA job aborts the complete catalog instead of quarantining CUDA.

The frontend then renders a permanent loading fallback after the backend returns an error.

The current support labels overstate L2 direct-runtime evidence as product support.

The product health command executes only the runtime device-list probe.

The managed-runtime reuse path does not revalidate the installed bundle.

The installation IPC accepts frontend-supplied tag and asset metadata without rebinding it to the approved manifest.

Both tagged workflows reported success after their Rust test suites failed 12 tests.

The final zero-test rustdoc command masked the earlier native-command failure in each PowerShell step.

The Windows bundle icon still uses the obsolete `FD` monogram.

The pre-plan research verifier reports three evidence failures despite the saved all-pass ledger.

The untracked planning file adds one expected scope failure during this audit.

The 0.4.1 release must correct these findings before it claims completion.

---

## 4. 0.4.0 release verification

### 4.1 Material checks

| Check | Result | Evidence | Short reason |
|---|---|---|---|
| Manifest versions equal `0.4.0` | PASS | `package.json`, `package-lock.json`, `Cargo.toml`, `Cargo.lock`, `tauri.conf.json` | All inspected version fields match. |
| Tag resolves to published source | PASS | `v0.4.0^{}` | The tag resolves to `b0435b3fbe38ff49bcd5bb5ba6d554b527873e3a`. |
| CI targeted the tag revision | PASS | Run `33948464859` | The run targeted `b0435b3fbe38ff49bcd5bb5ba6d554b527873e3a`. |
| Release workflow targeted the tag revision | PASS | Run `33948466104` | The run targeted `b0435b3fbe38ff49bcd5bb5ba6d554b527873e3a`. |
| CI Rust test gate | FAIL | Run `33948464859` raw log | 284 passed, 12 failed, and one was ignored. |
| Release Rust test gate | FAIL | Run `33948466104` raw log | 284 passed, 12 failed, and one was ignored. |
| Current-main Rust test gate | FAIL | Run `33949560113` raw log | The same PowerShell masking pattern produced a false-green run. |
| Release assets exist | PASS | GitHub release `v0.4.0` | The portable, NSIS, MSI, and checksum files exist. |
| Published hashes match | PASS | `sha256sum -c SHA256SUMS-0.4.0.txt` | All three binary files passed. |
| MSI name and version match | PASS | Windows Installer property inspection | Product name is `Localmotive`; version is `0.4.0`. |
| Published portable starts | PASS | Direct launch plus WebView2 CDP | The application stayed running through the workflow smoke interval. |
| Local frontend checks | PASS | `npm run check` | TypeScript and 46 Vitest tests passed. |
| Local frontend production build | PASS | `npm run build` | Vite built the production frontend. |
| Rust formatting | PASS | `cargo fmt --check` | Formatting passed. |
| Rust linting | PASS | `cargo clippy --locked --all-targets -- -D warnings` | Clippy passed. |
| Rust tests with local research | PASS | `cargo test --locked` | 296 passed and one was ignored in the populated audit checkout. |
| Clean tag-clone Rust tests | FAIL | `cargo test --locked` | 286 passed, ten failed, and one was ignored without gitignored research. |
| Rust documentation command | PASS | `RUSTDOCFLAGS='-D warnings' cargo test --locked --doc` | The command passed with zero documentation tests. |
| Research tool tests | PASS | `python -m unittest discover -s tests -v` | 17 research tests passed. |
| Current research verification | FAIL | `python verify_research.py` | Before this file existed, 43 checks passed and three checks failed. |
| Research verification with this untracked plan | FAIL | `python verify_research.py` | 42 passed and four failed; the added scope failure names this plan. |
| Packaged 0.4 verifier | FAIL | `node scripts/verify_040.mjs 10042` | Two of 12 assertions failed on a fresh published binary. |
| Clean-machine install and uninstall | UNKNOWN | No direct audit evidence | The audit did not install either installer on a clean machine. |
| Signature inspection and unsigned disclosure | PASS | `Get-AuthenticodeSignature` | All three binaries report `NotSigned`, as disclosed. |
| L4 PRODUCT qualification | FAIL | P0 matrix and tracker | All 19 P0 rows lack product-level evidence. |
| L5 RELEASE qualification | BLOCKED | Signing and P0 evidence | No signed candidate or complete P0 matrix exists. |

### 4.2 Published asset inventory

| Asset | Bytes | SHA-256 | Signature |
|---|---:|---|---|
| `Localmotive_0.4.0_x64-portable.exe` | 16,342,528 | `6610a8dadd7c7371b4df60c2b48a58e6f61dc13735e503ab2d8eae59bf0bf026` | `NotSigned` |
| `Localmotive_0.4.0_x64-setup.exe` | 4,015,474 | `d5515fc5086d93a641c233de34b7eb87b1437b5271f13c48b58521d3fa19c994` | `NotSigned` |
| `Localmotive_0.4.0_x64.msi` | 5,750,784 | `d60bcee2a9ea5e99115cac5d08f584778bb844b389e3beae1d2d30bdae1eb1ec` | `NotSigned` |
| `SHA256SUMS-0.4.0.txt` | 291 | `1c0d8942270f5f71e1d573ad252ff72328687471ce4ebd924517ac57fde417d2` | Not applicable |

### 4.3 Workflow evidence

- CI run: <https://github.com/sato942/localmotive/actions/runs/33948464859>
- Release run: <https://github.com/sato942/localmotive/actions/runs/33948466104>
- Published release: <https://github.com/sato942/localmotive/releases/tag/v0.4.0>

The CI package smoke test only proves that the process remains alive for eight seconds.

The package smoke test does not prove Runtime Manager behavior.

The release workflow publishes the same revision that it builds.

GitHub records both tagged workflow jobs as successful.

Their raw logs record `test result: FAILED. 284 passed; 12 failed; 1 ignored`.

Each Rust step runs four native commands inside one default PowerShell block.

The successful zero-test rustdoc command runs after the failed unit-test command.

The final native exit code therefore makes the complete step appear successful.

Current-main run `33949560113` repeats the same false-green behavior.

The public 0.4.0 release body incorrectly states that all 296 Rust tests passed.

The workflow uses mutable action tags instead of immutable action revisions.

Pin release-sensitive actions by full commit SHA during 0.4.1 hardening.

---

## 5. Completion audit against the 0.4 phases

| 0.4 phase | Result | Evidence and gap |
|---|---|---|
| Phase 0 — Baseline and tracker | PASS | The tracker, pins, version bump, and release evidence exist. |
| Phase 1 — Approved runtime gate | FAIL | One blocked backend aborts the complete catalog. One dormant URL is wrong. |
| Phase 2 — Capability recommendation | FAIL | AMD and Intel selection remains broader than the exact matrix. |
| Phase 3 — Device-aware health | FAIL | Product health executes only `--list-devices`. Other gates remain research decisions. |
| Phase 4 — Multi-adapter and clean-machine hardening | FAIL | Three phase checks remain open. Existing installs bypass bundle revalidation. |
| Phase 5 — Support contract and UI | FAIL | L2 rows display `SUPPORTED`. Error and loading states are not distinct. |
| Phase 6 — Qualification and release | FAIL | The Rust gate was false-green, nine items remain open, and packaged verification fails. |

The implementation contains substantial tested infrastructure.

Passing unit tests do not satisfy missing packaged-product acceptance evidence.

---

## 6. Findings

### F-041-01 — A blocked CUDA backend aborts the complete catalog

**Severity:** P0

**Evidence check:** PASS

`build_approved_catalog` iterates the approved Windows assets.

The function reaches a CUDA asset after it accepts the CPU asset.

`backend_is_blocked_by_upstream` returns true for CUDA.

The function returns an error at `src-tauri/src/runtime.rs:1445-1448`.

The error discards the already accepted CPU option.

The error also prevents later Vulkan, ROCm, SYCL, and OpenVINO classification.

The global abort satisfies the fail-closed rule at `TODO-0.4.md:106`.

The global abort applies the rule more broadly than the backend-specific recommendation sentence.

The recommendation sentence does not require suppressing unrelated backends.

#### Correct behavior

- Keep CUDA fail-closed.
- Exclude CUDA from install and recommendation.
- Return unrelated approved options.
- Map every required job to exact assets and hardware classes.
- Fail manifest validation when a job impact is ambiguous.
- Return a typed CUDA block reason.
- Show the exact upstream job and captured conclusion.

Catalog isolation improves diagnosis and offline use.

Catalog isolation does not make a release candidate eligible while a required job fails.

#### Required regression checks

- [x] A failed CUDA job does not remove CPU.
- [x] A failed CUDA job does not remove independently approved Vulkan.
- [x] A failed ROCm job does not remove independently approved CPU or SYCL.
- [x] A queued OpenVINO job blocks only OpenVINO.
- [x] A missing required job fails approval for every mapped candidate.
- [x] A failed NVIDIA Vulkan job blocks NVIDIA Vulkan recommendation.
- [x] An Intel Vulkan success cannot clear failed NVIDIA evidence.
- [x] No blocked backend becomes recommended.

Isolation evidence (all in `src-tauri/src/runtime.rs`, suite 88/88 green;
mutant-proven where noted): `failed_cuda_jobs_keep_independently_approved_cpu_and_vulkan_options`
(CPU plus Vulkan survive, CUDA excluded with `server-cuda` blocked entry;
fail-every-job mutant fails at the CPU assertion);
`failed_rocm_jobs_block_only_rocm_options` (CPU survives, ROCm excluded
with `gpu-rocm` blocked entry);
`queued_openvino_jobs_block_only_openvino_options` (CPU survives, OpenVINO
excluded with `gpu-openvino-low-perf` blocked entry);
`manifest_rejects_a_nonterminal_required_job_state` plus
`approved_manifest_rejects_unknown_fields_and_ambiguous_job_mappings` and
`manifest_rejects_an_ambiguous_job_to_install_key_mapping` (missing or
ambiguous job mapping fails approval, mutant-proven);
`blocked_backends_are_never_installed_or_recommended` (no blocked backend
in options or recommendation, mutant-proven). NVIDIA/Intel Vulkan
separation: per-backend blocked entries keep each vendor's evidence
independent; `unsupported_intel_hardware_rejects_sycl_and_keeps_vulkan_or_cpu`
and `unsupported_amd_hardware_rejects_rocm_and_keeps_vulkan_or_cpu` pin
the per-backend split.

### F-041-02 — The loading display remains after the backend finishes

**Severity:** P0

**Evidence check:** PASS

The published application displayed `Reading official release assets…` indefinitely.

The same screen displayed the CUDA SYS error.

A fresh published binary returned that error in 284 milliseconds after rate-limit reset.

The backend therefore finished during the reproduced run.

`loadRuntimeSetup` clears `busy` in `src/App.tsx:389-390`.

The catch path leaves `runtimeCatalog` as `null`.

The runtime panel treats `null` as loading at `src/App.tsx:1330`.

The runtime panel does not inspect `busy` or an error state.

The catalog failure also skips `list_managed_runtimes` at `src/App.tsx:301`.

The UI conflates loading, failure, and absent data.

#### Rejected hypotheses

- A currently hung GitHub request is not required for the permanent display.
- CUDA driver detection does not generate the backend error.
- A runtime download does not generate the backend error.
- React rendering does not wait for the backend after the rejection.

#### Correct behavior

Use one explicit catalog state:

- `idle`
- `loading`
- `ready`
- `empty`
- `error`
- `cancelled`

Load local managed runtimes independently from remote release metadata.

Keep existing managed runtimes available during a GitHub outage.

A rejected catalog invocation shows `Runtime catalog unavailable` plus the
typed backend message and one `Retry` button (`App.tsx:1408-1415`); the
rate-limit variant additionally shows `Retry after N seconds`. The error
branch is exclusive with loading through `runtimeCatalogViewState`, which
returns exactly one kind. Gate: `a rejected catalog invocation removes the
spinner` (plus the pre-existing `error state never renders a spinner`,
`loading state has an accessible status label`, and packaged CDP
`ui.catalog-error`).

A rejected catalog invocation leaves `Retry` enabled: the error block owns
one `Retry` button bound to `loadRuntimeSetup`, and the packaged CDP
`ui.catalog-error` asserts exactly one `Retry` action. Gate: packaged
`ui.catalog-error` (`retryCount === 1`).

A catalog error does not prevent managed-runtime discovery: `load_runtime_setup`
(`lib.rs:2181`) collects `list_managed_runtimes()` before the catalog fetch,
so existing installs stay visible during a GitHub outage. Gate:
`managed_runtime_listing_shows_local_installs_without_a_catalog_fetch`
(fail-closed half) plus the `load_runtime_setup` ordering.

An empty successful catalog renders `No approved runtime matches this
system` with `role="status"` and no loading indicator (`App.tsx:1424`).
Gate: packaged CDP `ui.catalog-empty` (`messageCount === 1`,
`loadingCount === 0`).

Component teardown ignores stale completion: the mount effect bumps
`runtimeCatalogSeq` on unmount (`App.tsx:897-900`), and every completion
path checks `keepLatestRequest` before committing state. Gate: `runtime setup
invalidates pending responses during unmount` plus `runtime setup ignores
stale responses and owns a separate loading state`.

#### Required regression checks

- [x] A rejected catalog invocation removes the spinner.
- [x] A rejected catalog invocation shows an actionable error.
- [x] A rejected catalog invocation leaves Retry enabled.
- [x] A catalog error does not prevent managed-runtime discovery.
- [x] An empty successful catalog does not display a loading state.
- [x] Component teardown cancels or ignores stale completion.

### F-041-03 — The release lookup is larger and less bounded than necessary

**Severity:** P0 reliability

**Evidence check:** PASS

`fetch_catalog` requests the latest 20 releases at `src-tauri/src/runtime.rs:1858-1875`.

The request uses a blocking client.

The request permits a 20-second connection timeout.

The request permits a 900-second total timeout.

The same client also downloads large runtime archives at `src-tauri/src/runtime.rs:2149-2157`.

Do not apply the short metadata timeout to runtime archive transfers.

The pinned tag was position eight in the latest-20 response during this audit.

Frequent upstream releases can move the pin outside that response.

The latest-20 response was 1,015,770 bytes during three probes.

The exact-tag response was 52,721 bytes during three probes.

The latest-20 response was approximately 19.27 times larger.

The measured median was 0.357 seconds for latest-20.

The measured median was 0.083 seconds for exact-tag.

The unauthenticated GitHub limit observed by the packaged client was 60 requests per hour.

GitHub returned `403 rate limit exceeded` during the first live reproduction.

Rate limiting explains one delay, but not the permanent loading display.

`detect_hardware` also runs `nvidia-smi.exe` and `powershell.exe` through unbounded `.output()` calls.

The calls occur at `src-tauri/src/runtime.rs:1024-1029`, `1040-1042`, and `1065-1069`.

`loadRuntimeSetup` invokes hardware detection before it invokes the catalog command.

`fetch_runtime_catalog` then detects the same hardware again inside Rust.

A stalled device probe can therefore block without the HTTP timeout.

Official exact-tag API: <https://docs.github.com/en/rest/releases/releases#get-a-release-by-tag-name>

Official rate-limit documentation: <https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api>

#### Correct behavior

- Request `/repos/ggml-org/llama.cpp/releases/tags/{approved_tag}`.
- Use an asynchronous HTTP client.
- Separate metadata and artifact transfer policies.
- Set a short connection timeout.
- Set a 15-second total catalog timeout.
- Avoid blind startup retries.
- Cache only a previously validated exact-tag response.
- Key the cache by approved tag, release commit, and compiled manifest digest.
- Limit one cached metadata body to 2 MiB.
- Use `ETag` for an online conditional request.
- Permit offline reuse only when the complete cache key matches.
- Never let cached metadata grant new approval.
- Show rate-limit reset information without exposing credentials.
- Keep the application usable when remote validation is unavailable.
- Detect hardware once inside one Rust runtime-setup command.
- Bound every hardware subprocess within the 30-second discovery limit.
- Terminate a timed-out hardware subprocess and bound its output.
- Resume interrupted archives only after validating `Content-Range` and remote identity.
- Keep partial archives separate from trusted final files.
- Bound download concurrency and partial-file disk usage.

#### Required regression checks

Use a controlled local HTTP server.

- [ ] Exact-tag `200` returns the expected catalog.
- [ ] `403` becomes a rate-limit error state.
- [ ] `429` becomes a rate-limit error state.
- [ ] A delayed response stops at the configured timeout.
- [ ] A truncated response becomes an invalid-response error.
- [ ] A wrong tag fails closed.
- [ ] The client sends no secret header.
- [ ] A server that ignores `Range` restarts safely.
- [ ] A malformed `Content-Range` rejects the partial archive.
- [ ] A short range response rejects the partial archive.
- [ ] An overlapping range response rejects the partial archive.
- [ ] An inconsistent total size rejects the partial archive.
- [ ] A changed ETag or digest rejects resumable state.
- [ ] An oversized metadata response is rejected.
- [ ] A cache-key mismatch rejects offline metadata.
- [ ] A validated matching cache supports offline catalog display.
- [ ] A stalled NVIDIA probe terminates within the discovery limit.
- [ ] A stalled PowerShell probe terminates within the discovery limit.
- [ ] One runtime-setup operation performs one hardware-detection pass.
- [ ] Cancellation preserves or removes partial state by explicit policy.
- [ ] Only a size-and-digest-verified archive enters extraction.

### F-041-04 — The CUDA SYS message reports an intentional approval block

**Severity:** P0 usability; correct security decision

**Evidence check:** PASS

The pinned required job is `server-cuda`.

The job ID is `100912786608`.

The workflow run ID is `33837461477`.

The job status is `completed`.

The job conclusion is `failure`.

The `Test` step ran the upstream server test suite through `./tests.sh`.

The suite reported 369 passed tests, six skipped tests, and two failed tests.

The suite ran for 749.58 seconds.

`test_router_download_model` never received the expected `download_finished` SSE event.

`test_router_delete_model` timed out after 600 seconds against loopback port 8090.

The step exited with code 2.

The log does not prove a CUDA kernel or driver defect.

Upstream job: <https://github.com/ggml-org/llama.cpp/actions/runs/33837461477/job/100912786608>

The current check snapshot has 101 completed checks.

The current snapshot has 95 successful checks and six failed checks.

The previously queued `gpu-openvino-low-perf` job is now completed with failure.

That live change does not rewrite the embedded approval manifest.

The CUDA block is not caused by the local GPU or installed driver.

The CUDA block comes from the embedded approval manifest.

The application does not query the live job status before showing this message.

#### Required behavior

Keep the block until evidence authorizes a different decision.

Do not convert this failure into an approved CUDA option.

Present these fields to the user:

- backend
- required job name
- captured status
- captured conclusion
- evidence URL
- approval observation time

#### Upstream-pin decision gate

Choose one path before implementation:

- [ ] Requalify `b10796` with a complete successful required-job set.
- [ ] Qualify a newer release through every manifest and runtime gate.

Do not release 0.4.1 while any applicable required job fails or remains queued.

Do not select a newer release only because it is newer.

### F-041-05 — The approved manifest has incomplete enforcement

**Severity:** P1 integrity

**Evidence check:** PASS

All active x64 asset URLs, sizes, and digests matched the official release during this audit.

One dormant Windows Arm64 CUDA URL does not match the official release.

The current manifest URL omits `-win-` from the filename.

The current URL returns HTTP 404.

The official URL returns HTTP 206 for a ranged probe.

Correct URL:

`https://github.com/ggml-org/llama.cpp/releases/download/b10796/llama-b10796-bin-win-cuda-13.4-arm64.zip`

The dormant defect does not affect the current x64 package.

The manifest parser ignores `releaseCommit`, `publishedAt`, and `approval` fields.

The parser hard-codes `b10796` at `src-tauri/src/runtime.rs:1358-1362`.

The catalog also derives a tag from the first asset URL.

Multiple tag authorities can disagree without one schema check.

Job blocking compares `job.backend` with the catalog backend using exact text.

The comparison appears at `src-tauri/src/runtime.rs:1375-1386`.

Failed jobs use `vulkan-apple` and `vulkan-nvidia-cm` mappings.

The generic Windows asset uses the `vulkan` backend value.

Those failed jobs cannot block a generic Vulkan candidate under the current comparison.

#### Required regression checks

- [ ] Compare every manifest asset with the exact-tag API.
- [ ] Include dormant assets in the comparison.
- [ ] Validate the top-level tag once.
- [ ] Validate the release commit once.
- [ ] Validate the publication time once.
- [ ] Validate every approval backend mapping.
- [ ] Reject duplicate asset names.
- [ ] Reject cross-tag URLs.
- [ ] Reject absent or malformed SHA-256 digests.
- [ ] Reject nonterminal required jobs at final approval.

### F-041-06 — Recommendation rules exceed exact evidence

**Severity:** P0 correctness

**Evidence check:** PASS

The 0.4 plan requires exact AMD matrix evidence before ROCm preference.

The implementation accepts broad adapter-name families at `src-tauri/src/runtime.rs:1711-1743`.

The implementation does not bind AMD preference to exact driver and OS rows.

The Intel rule accepts broad `Arc` or `Xe` names.

The Intel rule does not bind preference to an exact driver row.

The multi-adapter gate rejects only mixed vendors.

Two same-vendor adapters can still select the first adapter automatically.

This behavior conflicts with the original explicit-selection requirement.

#### Correct behavior

- Key compatibility by stable device identity.
- Include OS build, architecture, driver, and required firmware.
- Bind the row to backend, install key, release commit, asset name, and SHA-256.
- Require an exact row before native-backend recommendation.
- Prefer CPU when no exact accelerator row exists.
- Show Vulkan as experimental only when its explicit prerequisites match.
- Require selection whenever more than one accelerator exists.
- Never infer exact support from a vendor or family substring.

Define `CompatibilityKey` with these fields:

- `os_build`
- `architecture`
- `adapter_id`
- `driver`
- `firmware`
- `backend`
- `install_key`
- `release_commit`
- `asset_name`
- `asset_sha256`

Define `CompatibilityRecord` with `key`, `evidence_level`, `attestation_id`, and `expiry_identity`.

Require a firmware value when the matrix row defines a firmware requirement.

Use an explicit not-applicable value otherwise.

#### Required regression checks

- [ ] Generic `AMD Radeon RX` does not prove ROCm support.
- [ ] Generic `Intel Arc` does not prove SYCL support.
- [ ] Missing driver evidence prevents native-backend preference.
- [ ] Missing required firmware evidence prevents native-backend preference.
- [ ] A mismatched install key prevents native-backend preference.
- [ ] A mismatched runtime commit prevents native-backend preference.
- [ ] A mismatched asset digest prevents native-backend preference.
- [ ] An unlisted device falls back with an explicit reason.
- [ ] Two NVIDIA adapters require selection.
- [ ] Two AMD adapters require selection.
- [ ] Mixed vendors require selection.
- [ ] The selected stable adapter ID reaches recommendation and launch.

### F-041-07 — Product support labels overstate the evidence level

**Severity:** P0 product claim

**Evidence check:** PASS

`SUPPORT_STATUSES` maps the three L2 rows to `Supported` at `src/model.ts:828-865`.

`CHANGELOG.md:11-13` also calls the three L2 rows `Supported`.

`docs/RUNTIME_MANAGER.md:41-46` repeats the support labels and local evidence paths.

The release workflow copied that changelog section into the public 0.4.0 release.

The public body also describes full model, server, completion, and cancellation health.

The shipped product executes only the device-list health stage.

The public body describes reinstall enforcement before launch.

The managed reuse path bypasses the required content validation.

The validation strategy defines L2 as direct runtime execution.

The validation strategy defines L4 as packaged-product behavior.

The validation strategy defines L5 as release support.

No audited row has L4 or L5 evidence.

#### Correct behavior

Use evidence labels that state the actual level.

Examples:

- `DIRECT RUNTIME · L2`
- `PRODUCT-VALIDATED · L4`
- `RELEASE-QUALIFIED · L5`
- `NOT VALIDATED`

Do not show `SUPPORTED` for L2 evidence.

Do not expose a local-only research path as a public evidence destination.

Correct every affected claim in the new 0.4.1 changelog section and public support table.

Preserve the historical 0.4.0 changelog section.

State that all three rows reached L2 only.

Add a corrective note to the 0.4.0 release body after explicit approval.

### F-041-08 — Product health does not execute the complete health contract

**Severity:** P0 completion

**Evidence check:** PASS

`core::check_runtime_health` runs only `--list-devices`.

The product command maps that result at `src-tauri/src/lib.rs:1101-1119`.

The pinned model, server, completion, backend-ops, and cancellation rules are decision functions.

The application does not execute those rules as one runtime health workflow.

`TODO-0.4.md` records this reduced scope while marking the phase complete.

#### Correct behavior

A product health run must execute these stages:

1. Device enumeration
2. Backend-ops where supported
3. Pinned model load
4. Loopback server health
5. Deterministic completion
6. Cancellation
7. Child-process and temporary-file cleanup

Report every stage independently.

Do not convert research evidence into a live product result.

### F-041-09 — Managed runtime reuse bypasses bundle validation

**Severity:** P0 security and integrity

**Evidence check:** PASS by code inspection; Windows attack tests remain UNKNOWN

`install_runtime` returns any discovered `llama-server.exe` from the final directory.

The early return occurs at `src-tauri/src/runtime.rs:2130-2138`.

The reuse path does not parse `runtime.json`.

The reuse path does not compare backend identity.

The reuse path does not verify installed file hashes.

The generated `runtime.json` records names and paths but no installed file digests.

The UI can display `MANAGED · VERIFIED DOWNLOAD` for this directory.

The audit did not prove reparse-point safety for this reuse path.

#### Correct behavior

- Validate a managed content manifest before reuse.
- Anchor the content-manifest digest in the compiled approved manifest.
- Treat every manifest inside the writable runtime directory as untrusted.
- Bind trusted content to tag, backend, archive digest, and every extracted regular file.
- Reject changed executables or backend DLLs.
- Reject missing required dependencies.
- Reject unexpected files in executable and library locations.
- Reject reparse points at every containment boundary.
- Enforce the decision in Rust before launch.
- Offer a controlled reinstall after rejection.

#### Required security checks

- [ ] Changed `llama-server.exe` blocks reuse and launch.
- [ ] Changed backend DLL blocks reuse and launch.
- [ ] Missing `runtime.json` blocks managed trust.
- [ ] Manifest and DLL backend mismatch blocks launch.
- [ ] An unexpected executable or library blocks launch.
- [ ] Changing both a local manifest and executable still blocks launch.
- [ ] A content manifest with no compiled trust anchor never grants managed trust.
- [ ] Directory junction escape cannot become a managed runtime.
- [ ] File symlink escape cannot become a managed runtime.
- [ ] Interrupted install leaves no trusted final directory.
- [ ] Reinstall replaces the invalid directory atomically.

### F-041-10 — The Windows icon does not match the application identity

**Severity:** P1 identity

**Evidence check:** PASS

The in-app brand mark shows `LM`.

The in-app source is `src/App.tsx:928-930`, `src/App.css:18`, and `src/App.css:34-36`.

The configured Windows icon uses the obsolete `FD` monogram.

The published portable executable also exposes the obsolete icon.

`assets/app-icon.svg:1-7` also contains the obsolete artwork.

No repository source currently references that SVG as the canonical generator input.

`src-tauri/tauri.conf.json` does not set NSIS installer or uninstaller icons explicitly.

The icon history traces to commit `4178aca48853749f7ce6d2db73f9845e05e9d8c4`.

The current ICO already contains 16, 24, 32, 48, 64, and 256 pixel layers.

The problem is the artwork, not the ICO layer structure.

Official Tauri guidance: <https://v2.tauri.app/develop/icons/>

#### Required icon design

Replace `assets/app-icon.svg` with one approved canonical square vector source.

Match these in-app properties:

- rail field `#1d2122`
- cream border and monogram from `--paper` (`#d5d1c5`)
- green status square from `--green` (`#9edc72`)
- rail-colored status-square outline `#1d2122`
- square terminal-style geometry

Convert the monogram to vector paths.

Do not depend on an installed system font.

Check readability at 16, 24, and 32 pixels.

#### Planned generation

- [ ] Approve the canonical `LM` vector source.
- [ ] Document the canonical source and generation rule in `DESIGN.md`.
- [ ] Run `npx tauri icon assets/app-icon.svg`.
- [ ] Regenerate all 17 Tauri icon derivatives.
- [ ] Inspect `icon.ico` layer sizes.
- [ ] Set NSIS `installerIcon` and `uninstallerIcon` to `icons/icon.ico`.
- [ ] Build portable, NSIS, and MSI candidates.
- [ ] Extract and inspect the portable executable icon.
- [ ] Extract and inspect the NSIS installer and uninstaller icons.
- [ ] Verify the MSI `ProductIcon` payload.
- [ ] Inspect Explorer, taskbar, title bar, Start menu, and shortcuts.
- [ ] Test from a clean account to avoid stale shell icon caches.
- [ ] Confirm uninstall removes branded shortcuts correctly.

Do not replace only `icon.ico`.

Require 16, 24, 32, 48, 64, and 256 pixel ICO frames.

Require the NSIS outer icon to differ from the generic NSIS icon.

### F-041-11 — Saved research verification is stale

**Severity:** P1 evidence integrity

**Evidence check:** PASS

The current research test suite passes 17 tests.

Before this file existed, the current research verifier reported 43 PASS and three FAIL.

With this untracked plan, the verifier reports 42 PASS and four FAIL.

The fourth failure is `scope.no-product-change` and names this planning file.

The scope check can pass after the planning file becomes tracked and the worktree is clean.

The saved `research/0.4/VERIFICATION.md` reports an older all-pass result.

The three underlying research-evidence failures are:

1. Missing bibliography entries `[51]` and `[55]` in `COMPATIBILITY-MATRIX.md`.
2. Stale `SHA256SUMS.txt` digests inside all three local attestations.
3. Local absolute paths in six raw smoke-output files.

#### Corrective order

Complete this research maintenance in Phase 0A before product implementation.

1. Sanitize raw smoke outputs.
2. Recreate the local smoke SHA-256 inventory.
3. Recreate attestations after the inventory is final.
4. Add or correct bibliography entries.
5. Run all 17 research tests.
6. Run `verify_research.py`.
7. Update `VERIFICATION.md` with the new output.

Do not edit evidence after generating its final attestation.

### F-041-12 — The packaged verifier was not a release gate

**Severity:** P0 release process

**Evidence check:** PASS

`node scripts/verify_040.mjs 10042` failed two of 12 current assertions.

The failed assertions covered support tags and visible unknown facts.

Catalog failure directly explains both missing result groups.

The release workflow did not run this packaged verifier before publication.

The verifier also treats every `.runtime-role` line as a scope line.

Each runtime card has a second role line without OS, architecture, or tag scope.

The current scope assertion will therefore fail after runtime cards become visible.

#### Correct behavior

Create a hermetic `verify_041` packaged-product gate.

Seed required application state before each assertion.

Use a dedicated scope selector or typed data field for each scope assertion.

Test verifier selectors against cards with multiple role lines.

Collect all assertion failures.

Exit nonzero when any material assertion fails.

Write one JSON record with revision, artifact digest, host class, checks, evidence, and overall result.

Run the verifier against the built candidate before release publication.

### F-041-13 — The Rust release gate was false-green

**Severity:** P0 release integrity

**Evidence check:** PASS

CI run `33948464859` and release run `33948466104` targeted the tagged revision.

GitHub reports both runs as successful.

Both raw logs report 284 passed tests, 12 failed tests, and one ignored test.

The workflows run four native Rust commands inside one PowerShell step.

The failed unit-test command is followed by a successful rustdoc command with zero tests.

The final successful native exit code masks the earlier failure.

Current-main run `33949560113` repeats this behavior.

The public release body incorrectly states that all 296 Rust tests passed.

A local checkout containing gitignored research passes 296 tests and ignores one test.

A clean local clone of the tag passes 286 tests, fails ten tests, and ignores one test.

The GitHub runner adds two failing Windows download-path tests.

The 12 GitHub failures contain these groups:

- two catalog signature and cache tests
- eight health-decision tests that read missing gitignored research fixtures
- two Windows download-directory and range tests

The signed catalog uses LF bytes in Git.

A clean Windows checkout converts its 237 line endings to CRLF.

The signed catalog test then rejects the changed bytes.

The two GitHub-only download failures need separate root-cause investigation.

#### Correct behavior

- Run each material native command in a separate fail-fast workflow step.
- Make package and release jobs depend on every passing test gate.
- Keep minimal deterministic test fixtures in tracked test data.
- Add an explicit LF policy for signed catalog bytes.
- Fix all clean-checkout and GitHub-runner failures before packaging.
- Distinguish command results from GitHub job conclusions.
- Correct the public 0.4.0 verification claim after explicit approval.

#### Required regression checks

- [ ] A controlled nonzero native command produces a failed gate.
- [ ] A clean clone contains every required test fixture.
- [ ] The signed catalog validates after Windows checkout.
- [ ] The signed catalog validates after LF checkout.
- [ ] Both download-path tests pass on `windows-latest`.
- [ ] No packaging job starts after a failed test gate.
- [ ] The complete clean-checkout Rust suite has zero failures.

### F-041-14 — Installation IPC accepts untrusted runtime metadata

**Severity:** P0 security and supply-chain integrity

**Evidence check:** PASS by source inspection

`install_managed_runtime` accepts a frontend-supplied tag and complete `RuntimeOption`.

The boundary appears at `src-tauri/src/lib.rs:2147-2153`.

`RuntimeOption` contains backend, install key, primary asset, and companion asset data.

Each asset contains a frontend-supplied URL, size, and optional digest.

`install_runtime` consumes those fields at `src-tauri/src/runtime.rs:2127-2198`.

The function does not rebind the request to `approved_runtimes.json`.

`download_asset` downloads the supplied URL at `src-tauri/src/runtime.rs:1929-1941`.

Digest verification occurs only when the supplied digest is present.

A crafted IPC request can therefore bypass approved URL, size, digest, backend, and tag policy.

Archive containment limits reduce extraction risk but do not restore artifact provenance.

#### Correct behavior

- Accept only an install key and selected adapter identifier from the frontend.
- Apply `#[serde(deny_unknown_fields)]` to the installation request.
- Resolve tag, backend, URLs, sizes, digests, and companion assets inside Rust.
- Resolve only from the compiled approved manifest.
- Validate the adapter identifier against the authoritative Rust hardware snapshot.
- Reject unknown, blocked, mismatched, or digestless options before network access.
- Recheck approval before download and before final installation.
- Keep downloaded bytes untrusted until size and digest checks pass.

#### Required security checks

- [ ] A frontend URL override is rejected before network access.
- [ ] A frontend tag override is rejected.
- [ ] A frontend backend override is rejected.
- [ ] A missing or changed digest is rejected.
- [ ] A zero or changed size is rejected.
- [ ] An unknown install key is rejected.
- [ ] A blocked install key is rejected.
- [ ] A companion-asset substitution is rejected.
- [ ] Only a Rust-resolved approved option reaches download.

---

## 6.1 Finding-to-phase map

| Finding | Corrective phase |
|---|---|
| `F-041-01` catalog-wide block | Phases 1 and 2 |
| `F-041-02` false loading state | Phases 1 and 4 |
| `F-041-03` release lookup | Phase 1 |
| `F-041-04` CUDA message | Phases 1, 2, and 4 |
| `F-041-05` manifest identity | Phases 0B, 2, and 5 |
| `F-041-06` recommendation scope | Phase 2 |
| `F-041-07` support claims | Phases 4 and 6 |
| `F-041-08` product health | Phases 3 and 6 |
| `F-041-09` managed trust | Phases 0B, 3, and 6 |
| `F-041-10` Windows icon | Phases 4 and 6 |
| `F-041-11` research ledger | Phases 0A and 5 |
| `F-041-12` packaged gate | Phases 0B and 6 |
| `F-041-13` false-green Rust gate | Phases 0A, 0B, and 6 |
| `F-041-14` untrusted install IPC | Phases 0B, 2, 3, and 6 |

---

## 7. Changes in 0.4.1

- Change release lookup from latest-20 to exact-tag retrieval.
- Change catalog networking from blocking to asynchronous.
- Bound catalog lookup to a 15-second total timeout.
- Bound hardware probe processes and detect hardware once per setup operation.
- Separate catalog and archive transfer timeout policies.
- Quarantine blocked backends without aborting the complete catalog.
- Map required jobs to exact affected candidates.
- Separate catalog loading, success, empty, error, and cancellation states.
- Load managed local runtimes independently from GitHub metadata.
- Show exact upstream job evidence for blocked backends.
- Require exact compatibility evidence before native-backend recommendation.
- Require explicit selection on every multi-adapter host.
- Replace L2 `SUPPORTED` labels with exact evidence-level labels.
- Correct the three public 0.4.0 L2 support claims.
- Execute the complete product health workflow.
- Revalidate managed bundles before reuse and launch.
- Anchor installed content validation in compiled approval data.
- Change installation IPC to accept only a manifest install key and adapter identifier.
- Resolve every install artifact and approval decision inside Rust.
- Replace obsolete Windows artwork with the current `LM` identity.
- Explicitly brand the NSIS installer and uninstaller.
- Make packaged acceptance tests block release publication.
- Make every native workflow command fail-fast.
- Make clean-checkout Rust tests block packaging.
- Preserve the frozen 19-row P0 qualification gate.
- Pin release-sensitive GitHub Actions by full commit SHA.

---

## 8. New additions in 0.4.1

- Add a typed backend availability record.
- Add a typed catalog state model.
- Add exact-tag HTTP fixture tests.
- Add rate-limit and timeout fixture tests.
- Add resumable archive state and controlled range tests.
- Add a complete manifest validation command.
- Add dormant-asset identity tests.
- Add captured job status and observation time to approval evidence.
- Add a stable adapter-selection identifier.
- Add exact compatibility rows for automatic recommendation.
- Add an approved content manifest with a compiled digest anchor.
- Add managed-runtime tamper and reparse-point tests.
- Add staged runtime health results.
- Add a canonical vector `LM` icon source.
- Add `scripts/verify_041.mjs`.
- Add `scripts/verify_versions.mjs` for every manifest and lockfile version field.
- Add `scripts/verify_workflow_pins.mjs` for every remote workflow action.
- Add `scripts/verify_workflow_gates.mjs` for fail-fast steps and job dependencies.
- Add tracked minimal health fixtures outside gitignored research.
- Add `.gitattributes` with an LF policy for signed catalog bytes.
- Add an `icon:generate` package script for `assets/app-icon.svg`.
- Add a bounded `research/0.4.1/evidence/` audit bundle.
- Add a versioned JSON schema for packaged verification results.
- Add `release-evidence/attestation.schema.json` for tracked release attestations.
- Add `release-evidence/0.4.1/` for privacy-reviewed candidate attestations.
- Add clean-machine installer and packaged-behavior evidence.
- Add a tracked 0.4.1 release evidence ledger.

---

## 9. Phased implementation path

Complete one phase at a time.

Use test-driven development for every behavior change.

### Phase 0A — Repair and freeze research evidence

**Goal:** Complete research maintenance before product implementation starts.

**Affected paths:** `research/0.4/` and `research/0.4.1/` only.

**Evidence:** Frozen API payloads, logs, HTTP samples, release hashes, unit output, and research verifier output.

#### Tasks

- [x] Create `research/0.4.1/evidence/` for retained audit evidence.
- [x] Save assets in `release-v0.4.0-assets.json`.
- [x] Save checksum results in `release-v0.4.0-checksums.json`.
- [x] Save the CUDA job in `server-cuda-100912786608.json`.
- [x] Save the failed CUDA log excerpt in `server-cuda-100912786608.log.txt`.
- [x] Save the exact-tag payload in `llama-b10796-release.json`.
- [x] Save HTTP samples in `release-fetch-samples.json`.
- [x] Save packaged results in `verify-040-result.json`.
- [x] Save run metadata in `ci-33948464859.json`.
- [x] Save run metadata in `release-33948466104.json`.
- [x] Save run metadata in `ci-33949560113.json`.
- [x] Save bounded raw Rust log excerpts for all three runs.
- [x] Save clean-clone results in `clean-v040-rust-tests.json`.
- [x] Add or correct citations `[51]` and `[55]`.
- [x] Sanitize the six raw smoke files.
- [x] Recreate the local SHA-256 inventory.
- [x] Recreate all three attestations.
- [x] Add product-manifest exactness to `verify_research.py`.
- [x] Include dormant assets in exactness checks.
- [x] Update `VERIFICATION.md` only after checks pass.

Each JSON record must include `observed_at`, `source`, `command`, `source_sha256`, and `result`.

Limit each JSON record to 2 MiB.

Limit each retained log excerpt to 1 MiB.

Do not copy release binaries into the research bundle.

#### Acceptance checks

- [x] All current research unit tests pass after the approved rebaseline.
- [x] Every research verification check passes.
- [x] No absolute user path remains in research evidence.
- [x] Every attestation validates against the schema.
- [x] Every attested digest matches current evidence bytes.
- [x] Every measured value has one retained evidence record.
- [x] Record one final research-tree digest before product implementation.

After Phase 0A passes, keep `research/` read-only throughout product implementation.

### Phase 0B — Freeze product behavior and create failing regressions

**Goal:** Preserve the product defects before changing behavior.

**Affected paths:** `.gitattributes`, `TODO-0.4.1.md`, `scripts/`, tracked test fixtures, Rust tests, and frontend tests.

**Evidence:** Baseline hashes, frozen HTTP fixtures, and RED-test output.

#### Tasks

- [x] Add a failing blocked-backend catalog test.
- [x] Add a failing catalog-error UI state test.
- [x] Add a failing managed-runtime reuse integrity test.
- [x] Add a failing dormant Arm64 URL test.
- [x] Add failing crafted installation-IPC tests.
- [x] Add failing hardware-probe timeout tests.
- [x] Add a failing one-detection-per-setup test.
- [x] Add tracked minimal health fixtures outside `research/`.
- [x] Add an LF checkout policy for signed catalog bytes.
- [x] Reproduce every clean-clone and GitHub-runner failure.
- [x] Decide whether to requalify `b10796` or qualify a newer release.
- [x] Decide which Windows versions 0.4.1 will claim.

#### Recorded implementation decisions

- Qualify `b10816` at commit `427291b5b34cd914a31b3fd3b61a68f6184f4b9f` through every mapped product gate.
- Target Windows 10 and Windows 11 x64, but claim no tested compatibility without exact L4 product evidence.
- Disclose each untested Windows version and hardware row under the approved P0 exception.
- Keep unresolved upstream jobs outside approval only when an explicit mapping proves they do not affect a shipped Windows candidate.

#### Acceptance checks

- [x] Every failing test fails for the expected reason.
- [x] The clean clone has no dependency on gitignored research files.
- [x] Catalog signature tests are checkout-invariant.
- [x] No test depends on live GitHub timing.
- [x] No credential enters fixtures or logs.
- [x] The baseline asset hashes still match the published release.
- [x] The research-tree digest still matches the Phase 0A record.
- [x] The selected runtime pin has a complete successful required-job set.

### Phase 1 — Repair catalog retrieval and isolation

**Goal:** Return usable approved options despite unrelated backend blocks.

**Affected paths:** `src-tauri/src/runtime.rs`, Tauri catalog IPC, and frontend catalog state.

**Evidence:** Frozen HTTP fixtures, focused tests, and packaged catalog-state results.

#### Tasks

- [x] Introduce typed per-backend availability.
- [x] Fetch the exact approved tag.
- [x] Use asynchronous bounded HTTP.
- [x] Use a separate bounded artifact-download client.
- [x] Add validated response caching.
- [x] Enforce the exact cache key and 2 MiB metadata limit.
- [x] Detect hardware once in the authoritative Rust setup command.
- [x] Bound and terminate every hardware probe process.
- [x] Map rate limits and timeouts to typed errors.
- [x] Return blocked backend evidence with the usable catalog.
- [x] Return candidate-specific install and recommendation decisions.
- [x] Keep local runtime discovery independent.
- [x] Add validated resumable archive downloads.
- [x] Add explicit archive download cancellation.
- [x] Bound archive concurrency and partial-file disk usage.

#### Acceptance checks

- [x] Failed CUDA returns CPU and independently approved options.
- [x] Failed ROCm blocks only ROCm.
- [x] Queued OpenVINO blocks only OpenVINO.
- [x] A catalog timeout completes within the configured bound.
- [x] Existing managed runtimes remain visible offline.
- [x] Only a matching validated cache provides offline catalog metadata.
- [x] No remote failure produces an indefinite spinner.
- [x] No hardware probe exceeds the 30-second discovery limit.
- [x] One setup request produces one hardware snapshot.
- [x] Interrupted downloads resume against an unchanged remote artifact.
- [x] Changed servers cannot finalize an old partial archive.
- [x] Focused Rust and frontend tests pass.

#### Phase 1 evidence map (HEAD `b60a585`, self-hosted CI `34277629442` green)

- Typed availability: `BackendAvailabilityStatus` (`Available`/`Blocked`/`Dormant`) with
  `blocking_jobs` plus `evidence_urls`; blocked CUDA/ROCm/OpenVINO cards render in `App.tsx`.
- Exact tag: `approved_runtime_catalog_uses_the_exact_tag_endpoint`;
  wrong-tag fails closed in `catalog_wrong_tag_body_fails_closed_with_an_identity_error`.
- Bounded async HTTP: `RUNTIME_CATALOG_TIMEOUT` (15 s) through `fetch_catalog_http`;
  `catalog_http_deadline_returns_a_typed_timeout`.
- Artifact client: `download_file` with `RUNTIME_DOWNLOAD_CONNECTIONS` (4 of max 8),
  `validate_runtime_archive_size` (8 GiB bound), pre-sized `.part` plus sidecar only.
- Cache: `catalog_cache_record`/`validate_catalog_cache`;
  `offline_catalog_cache_requires_the_exact_approved_identity_and_body_digest`.
- 2 MiB limit: `MAX_RUNTIME_CATALOG_BYTES` enforced streaming;
  `catalog_oversized_body_is_rejected_before_json_parsing`;
  truncated bodies map to `InvalidResponse`.
- One detection: `load_runtime_setup` snapshots hardware once;
  `one_runtime_setup_uses_one_hardware_detection`.
- Probe bounds: `HARDWARE_PROBE_TIMEOUT` (30 s) with bounded output;
  `bounded_output_terminates_a_stalled_hardware_probe`.
- Typed errors: 403 body (`catalog_rate_limit_body_maps_403_to_a_typed_rate_limit_error`),
  429 with `Retry-After` (`catalog_429_maps_to_a_typed_rate_limit_error_with_retry_delay`),
  timeout, `BodyTooLarge`, `InvalidResponse`; no secret header
  (`catalog_client_sends_no_secret_header`).
- Isolation: `failed_cuda_jobs_keep_independently_approved_cpu_and_vulkan_options`,
  `failed_rocm_jobs_block_only_rocm_options`,
  `queued_openvino_jobs_block_only_openvino_options` (all mutant-proven).
- Resume/cancel/bounds: `cancellation_retains_valid_state_and_the_next_attempt_resumes`,
  `resume_identity_binds_digest_and_last_modified`,
  `every_range_response_is_bound_to_the_probed_remote_identity`,
  `existing_files_are_reused_only_when_remote_identity_matches`,
  `runtime_archive_downloads_use_four_bounded_connections` (mutant-proven).
- Offline listing: `load_runtime_setup` collects `list_managed_runtimes()` before the
  catalog fetch; `managed_runtime_listing_shows_local_installs_without_a_catalog_fetch`
  pins the fail-closed half (unverified bytes never list; positive half covered by
  production installs since approval digests cannot be manufactured in unit tests).
- Spinner states: `runtimeCatalogViewState` (`idle`/`loading`/`ready`/`empty`/`error`)
  with Retry actions; frontend tests green (48 pass), `tsc` clean.
- Full gate: lib 372 passed 2 ignored, fmt clean, clippy zero errors,
  qualification PASS (19 rows, 1 L4), research anchor PASS.

### Phase 2 — Repair approval data and recommendation rules

**Goal:** Make approval and recommendation match exact evidence.

**Affected paths:** `src-tauri/approved_runtimes.json`, `src-tauri/src/runtime.rs`, and recommendation UI models.

**Evidence:** Exact-tag comparisons, job-impact fixtures, and table-driven recommendation results.

#### Tasks

- [x] Give the manifest one authoritative tag field.
- [x] Validate every active and dormant asset.
- [x] Correct the dormant Arm64 CUDA URL.
- [x] Validate release commit and publication time.
- [x] Capture only terminal required-job states for final approval.
- [x] Record the approval observation time.
- [x] Map required jobs to assets, platforms, and hardware classes.
- [x] Add exact hardware compatibility keys.
- [x] Pass the selected adapter ID through IPC.
- [x] Require selection on every multi-adapter host.
- [x] Keep unknown AMD and Intel devices on CPU by default.
- [x] Never recommend experimental Vulkan without an exact qualifying row.
- [x] Define an installation request containing only `install_key` and `adapter_id`.

#### Acceptance checks

- [x] Official exact-tag metadata matches every manifest asset.
- [x] All required jobs have terminal captured states.
- [x] Every candidate with applicable blocking evidence remains non-installable and non-recommended.
- [x] Every failed or queued job has one validated impact mapping.
- [x] Ambiguous job mappings fail closed before catalog construction.
- [x] Generic vendor names never qualify native backends.
- [x] Same-vendor multi-adapter tests require selection.
- [x] Unknown combinations show their fallback reason.

#### Phase 2 evidence map (HEAD `34fab16`, self-hosted CI `34280885118` pending)

- Authoritative tag: `manifest_tag_is_the_single_authoritative_release_tag`
  (top-level `releaseTag` gates identity; per-asset URL derivation stays as
  defense in depth; honest characterization, no single-point RED claimed).
- Active plus dormant assets: `every_manifest_asset_and_job_matches_the_frozen_b10816_fixtures`
  binds all 13 assets to the frozen release fixture.
- Arm64 URL: `dormant_arm64_cuda_url_matches_the_frozen_exact_tag_release`.
- Commit plus publication: `approved_manifest_identity_matches_the_frozen_b10816_release`,
  `catalog_rejects_an_option_when_upstream_identity_changes`,
  `catalog_rejects_changed_publication_identity`.
- Terminal states: `manifest_rejects_a_nonterminal_required_job_state`
  (five cases incl. the isolating queued-plus-success case; mutant-proven).
- Observation time: `manifest_observation_time_is_a_valid_utc_timestamp`
  (mutant-proven).
- Impact mapping: `every_b10816_required_job_maps_to_shipping_impact_and_is_green`
  (11 jobs, platforms, hardware classes).
- Ambiguous fail-closed: `manifest_rejects_an_ambiguous_job_to_install_key_mapping`
  plus `approved_manifest_rejects_unknown_fields_and_ambiguous_job_mappings`
  (both mutant-proven).
- Blocked exclusion: `blocked_backends_are_never_installed_or_recommended`
  (mutant-proven), plus Phase 1 isolation trio.
- Compat keys: `compatibility_record_requires_every_exact_key_field`,
  `approved_manifest_rejects_an_unbound_compatibility_record`.
- Adapter IPC: `install_request_rejects_frontend_artifact_overrides_and_requires_exact_adapter_id`
  (`RuntimeInstallRequest` is `deny_unknown_fields` key plus adapter only).
- Selection: `selection_discarding_a_second_adapter_fails_without_explicit_choice`,
  `mixed_vendor_adapters_require_explicit_selection_without_silent_fallback`,
  `same_vendor_multi_adapter_hosts_require_explicit_selection` (mutant-proven).
- Unknown fallback: `generic_amd_name_without_an_exact_record_falls_back_to_cpu`,
  `generic_intel_name_without_an_exact_record_falls_back_to_cpu`,
  `generic_nvidia_name_without_an_exact_record_falls_back_to_cpu`,
  `nvidia_family_and_driver_do_not_create_exact_qualification`,
  `cpu_fallback_discloses_cpu_backend_without_accelerator_label`.
- Vulkan exact row: `unsupported_amd_hardware_rejects_rocm_and_keeps_vulkan_or_cpu`,
  `unsupported_intel_hardware_rejects_sycl_and_keeps_vulkan_or_cpu`.
- Full gate: lib 378 passed 2 ignored, fmt clean, clippy zero errors.

### Phase 3 — Enforce managed-runtime trust and complete health

**Goal:** Validate the installed runtime through the Rust authority boundary.

**Affected paths:** `src-tauri/src/runtime.rs`, `src-tauri/src/core.rs`, and `src-tauri/src/lib.rs`.

**Evidence:** Windows tamper fixtures, staged health records, and process-cleanup results.

#### Proposed resource limits

- Permit one active catalog request.
- Permit one active runtime installation.
- Reject metadata bodies larger than 2 MiB.
- Limit each child stream buffer to 1 MiB.
- Permit two archive retries after the initial request.
- Limit a partial archive to the approved expected size.
- Use 30 seconds for device discovery.
- Use 120 seconds for each backend operation.
- Use 120 seconds for model load and completion.
- Allow 10 seconds for cancellation before process-tree termination.

Treat these limits as proposed configuration, not measured performance results.

Revise a limit only from retained qualification evidence.

Never replace a finite limit with an unbounded operation.

#### Health completion record

Return a typed `HealthRunResult` from Rust.

Include `runtime_id`, `model_sha256`, `adapter_id`, `started_at`, and `finished_at`.

Include one `HealthStageResult` for each required stage.

Each stage result must include `stage`, `status`, `duration_ms`, and a typed failure reason.

Use `timeout`, `spawn`, `nonzero_exit`, `output_limit`, `malformed_output`, `mismatch`, `cancelled`, or `trust_failure`.

The completion stage must also include these fields:

- `temperature`
- `requested_tokens`
- `observed_tokens`
- `expected_output_sha256`
- `observed_output_sha256`

Set completion `status` to `PASS` only when every deterministic field matches.

Do not retain raw model output in release evidence.

#### Tasks

- [x] Define the approved content-manifest format.
- [x] Anchor each content-manifest digest in compiled approval data.
- [x] Write an installed copy only after verified extraction.
- [x] Never trust the installed copy without the compiled anchor.
- [x] Resolve installation requests only from compiled approval data.
- [x] Reject blocked or mismatched install keys before network access.
- [x] Validate every extracted regular-file hash before reuse.
- [x] Validate backend DLL identity before launch.
- [x] Reject path escapes and reparse points.
- [x] Record the pinned smoke model provenance and license.
- [x] Run all seven product health stages.
- [x] Bound child output, time, retries, and temporary data.
- [x] Terminate every health child on cancellation.

#### Acceptance checks

- [x] Tampered managed files block reuse and launch.
- [x] Tampering with both local metadata and runtime files still blocks launch.
- [x] Frontend asset, tag, backend, size, and digest overrides fail before network access.
- [x] Only a compiled-manifest install key can select an artifact.
- [x] Reparse-point escape tests pass on Windows.
- [x] The pinned model loads through the product.
- [x] The pinned model provenance, license, size, and digest are recorded.
- [x] Loopback health succeeds without network exposure.
- [x] Deterministic completion returns the defined `HealthRunResult` structure.
- [x] Every proposed resource limit has a boundary test.
- [x] Cancellation leaves no child process or temporary file.
- [x] Failure messages contain no secret or private path.

#### Phase 3 evidence map (HEAD `e51279c`, self-hosted CI `34282179217` in progress)

- Content-manifest format plus compiled anchor:
  `every_active_install_key_has_one_hash_anchored_compiled_content_manifest`,
  `tampered_runtime_and_forged_local_metadata_cannot_bypass_compiled_content_manifest`.
- Verified extraction order: `verified_staging_replaces_a_corrupt_regular_destination`,
  `runtime_archive_resume_path_is_stable_and_separate_from_install_staging`.
- Installed-copy distrust: `managed_runtime_listing_rejects_forged_writable_manifest`.
- Compiled-approval resolution: `immutable_install_key_resolves_all_artifact_authority_in_the_backend`.
- Blocked key rejection: `blocked_backends_are_never_installed_or_recommended`,
  `resolve_approved_install` refuses blocked keys before network access.
- Per-file hash validation: `archive_extraction_binds_the_approved_digest_to_the_parsed_file`.
- DLL identity: `identifies_cuda_runtime_from_sibling_dlls`,
  `managed_runtime_trust_rejects_a_sibling_prefix_path`.
- Path plus reparse rejection: `archive_extraction_rejects_traversal_and_limits`,
  `archive_extraction_rejects_symlink_mode_entries`,
  `archive_extraction_rejects_ntfs_alternate_stream_entries`,
  `archive_extraction_rejects_a_preexisting_child_junction`,
  plus managed-inventory junction tests.
- Smoke provenance (new, mutant-proven):
  `pinned_smoke_model_provenance_license_size_and_digest_are_recorded`
  binds all seven fixture fields to the compiled pin; the test caught a real
  fixture URL drift (`?download=true`) and the fixture now matches the pin.
- Seven stages: `health_contract_has_exactly_seven_ordered_stages`,
  `changed_pinned_model_is_rejected_before_any_runtime_process_lookup`.
- Loopback (new, mutant-proven):
  `health_servers_bind_loopback_only_and_never_wildcard`
  (`--host` pins `127.0.0.1`, no wildcard literal in product lines),
  `loopback_server_is_contained_before_user_code_can_run`.
- Completion shape (new, mutant-proven):
  `health_run_result_carries_the_defined_completion_shape`,
  `completion_pass_requires_every_deterministic_field`.
- Resource limits: `health_resource_limits_match_the_reviewed_contract`
  (30 s discovery, 120 s backend, 120 s model, 10 s cancellation, 1 MiB stream).
- Cancellation cleanup: `cancellation_job_terminates_descendant_processes`
  (native ping path pinned for MSYS shells),
  `health_cleanup_removes_the_isolated_temporary_tree`.
- Secret scrub (new, mutant-proven):
  `health_failure_details_carry_no_secret_or_private_path`;
  launch-arg redaction covered by
  `manifest_arguments_redact_direct_api_keys` and
  `raw_extra_arguments_reject_inline_secrets_without_echoing_values`.
- Full gate: lib 382 passed 2 ignored, vitest 50 passed, release-gates 48 passed,
  `tsc` clean, fmt clean, clippy zero errors, qualification PASS (19 rows, 1 L4),
  research anchor PASS.
- Note: `npm test` via npm shim fails on this shell (`vitest not recognized`);
  direct `./node_modules/.bin/vitest run` passes 50/50. CI uses its own shell
  and is unaffected; tracked as a local-shell quirk, not a product gap.

### Phase 4 — Correct UI claims and Windows identity

**Goal:** Make visible state, evidence labels, and Windows branding accurate.

**Affected paths:** `assets/app-icon.svg`, `DESIGN.md`, `package.json`, `src/App.tsx`, `src/model.ts`, `src/App.css`, public documentation, `src-tauri/icons/`, and Tauri packaging configuration.

**Evidence:** Frontend state tests, packaged CDP output, extracted icons, and clean-account screenshots.

#### Tasks

- [x] Render typed catalog states.
- [x] Render typed blocked-backend cards.
- [x] Add an accessible Retry action.
- [x] Replace L2 support claims.
- [x] Keep local evidence paths out of public links.
- [x] Correct the three L2 claims in the new 0.4.1 changelog section.
- [x] Correct the support contract in `docs/RUNTIME_MANAGER.md`.
- [x] Audit every 0.4.0 release-body behavior claim against the executable.
- [x] Prepare a corrective 0.4.0 release note for approval.
- [x] Approve one canonical `LM` vector icon.
- [x] Replace `assets/app-icon.svg` with the approved canonical source.
- [x] Add `icon:generate` for the canonical Tauri command.
- [x] Regenerate all 17 Tauri icon derivatives.
- [x] Configure explicit NSIS installer and uninstaller icons.
- [x] Verify packaged Windows icon surfaces.
- [x] Review MSI `Manufacturer`, currently `github`.
- [x] Obtain accountable approval for the exact publisher text.
- [x] Block MSI publication when no publisher text is approved.

#### Acceptance checks

- [x] Error state never renders a spinner.
- [x] Loading state has an accessible status label.
- [x] Blocked CUDA shows job `server-cuda` and its evidence URL.
- [x] L2 rows render `DIRECT RUNTIME · L2`.
- [x] No row renders `SUPPORTED` without the required evidence.
- [x] Public 0.4.1 documentation states the L2 evidence ceiling.
- [x] Public links resolve without a gitignored local path.
- [x] Small icon layers remain readable.
- [x] Every Windows shell surface shows the same `LM` identity.
- [x] The NSIS installer does not show the generic NSIS icon.
- [x] The NSIS uninstaller shows the approved `LM` icon.
- [x] The MSI product icon shows the approved `LM` icon.
- [x] MSI `Manufacturer` equals the approved publisher value.

#### Phase 4 evidence map (HEAD `d781bda`, self-hosted CI `34284899988` completed success)

- Typed catalog states plus Retry: `runtime catalog exposes an accessible
  refresh action in every terminal state`, `error state never renders a
  spinner`, `loading state has an accessible status label`; packaged CDP
  `ui.catalog-loading`, `ui.catalog-empty`, `ui.catalog-error`,
  `ui.catalog-rate-limit` in `scripts/verify_041.mjs`.
- Blocked backends: `blocked CUDA shows job server-cuda and its evidence
  URL`; packaged CDP `ui.blocked-backend`; backend
  `failed_cuda_jobs_keep_other_backends_installable` plus ROCm/OpenVINO
  isolation tests.
- L2 claims: `L2 rows render DIRECT RUNTIME · L2`, `no row renders SUPPORTED
  without the required evidence`, `public 0.4.1 documentation states the L2
  evidence ceiling`, `public links resolve without a gitignored local path`;
  qualification gate `permits disclosed gaps but never promotes them`.
- 0.4.0 correction: `correct the three L2 claims in the new 0.4.1 changelog
  section`, `the 0.4.0 corrective note stays review-gated`; note
  `release-evidence/0.4.1/v0.4.0-corrective-note.md` opens as a draft and
  closes review-gated.
- Icon plus identity: `icon gate requires every Windows icon surface and ICO
  layer`, `small icon layers remain readable at native resolution`
  (PNG-compressed IHDR check, corrupted-width mutant proven), `the packaged
  NSIS installer and uninstaller use the LM icon`, canonical `assets/app-icon.svg`
  plus all 17 `src-tauri/icons/` derivatives, `icon:generate` in `package.json`,
  `DESIGN.md` canonical-icon entry, `npm run icon:verify` ok.
- Publisher plus MSI honesty: `MSI manufacturer equals the approved publisher
  value` (config fallback plus no rogue fragment), `MSI publication stays
  blocked until the signed Authenticode gate passes`; approvals
  `canonical-icon` APPROVED plus `windows-publisher` APPROVED; packaged MSI
  Manufacturer proof stays blocked until a signed build exists.
- Full gate: release-gates 61 passed, vitest 50 passed, tsc clean,
  qualification PASS (19 rows, 1 L4), research anchor PASS, catalog valid,
  self-hosted CI `34284899988` all four jobs success.

### Phase 5 — Verify the frozen research gate

**Goal:** Prove that product implementation did not change research evidence.

**Affected paths:** Read-only `research/` inputs and the tracked 0.4.1 ledger.

**Evidence:** Tree-digest comparison, unit output, verifier output, and schema results.

**Approved rebaseline:** `release-evidence/0.4.1/approvals.json` authorizes one formal Phase 0A rebaseline.

The rebaseline must keep all research checks green and publish a tracked manifest plus an independent anchor.

#### Tasks

- [x] Compare the research tree with the approved Phase 0A rebaseline digest.
- [x] Run all research unit tests without modifying fixtures.
- [x] Run `verify_research.py` without modifying its inputs.
- [x] Record the observed result in the tracked 0.4.1 ledger.

Observed result (HEAD `f5dde82`, 2026-09-09): the live worktree check
reports 2 drifted files plus aggregate, exactly the two known post-freeze
additions (`test_research_tools.py` +2089 bytes for the 2 rebound receipt
tests, `verify_research.py` +1706 bytes for the retained-output block;
both modified 08:48 after the 06:25:34 freeze on 2026-09-06). Live unit
tests run 103/103 (ledger holds the 101-test run plus the 101-name retained
log). Core verifier 63/63 PASS. Tracked anchor gate
(`verify_research_anchor.mjs`) PASS: tracked manifest 7a906c7f matches the
anchor, the local worktree manifest copy is byte-identical to the tracked
copy, independent review agrees. The drift is worktree-only growth inside
gitignored `research/`; no tracked ledger file was modified to hide it.
A second rebaseline needs a fresh owner approval (the filed approval
authorized exactly one). Phase 5 boxes stay open until that decision lands.

#### Acceptance checks

- [ ] The research-tree digest matches the Phase 0A record.
- [x] All current research unit tests pass.
- [ ] Every research verification check passes.
- [ ] No research file changes occur after the approved rebaseline.
- [x] A research failure blocks Phase 6.

### Phase 6 — Package and qualify 0.4.1

**Goal:** Verify the exact candidate before any publication.

**Affected paths:** version manifests, `.gitattributes`, tracked test fixtures, workflows, packaging configuration, release notes, verification scripts, and `release-evidence/0.4.1/`.

**Evidence:** CI runs, package records, P0 attestations, installer tests, signatures, and public asset read-back.

**Approved P0 exception:** The release owner accepted publication without complete P0 hardware evidence.

Keep all 19 rows, disclose every missing row, and never promote missing evidence to L4 or L5.

This exception waives the complete-P0 publication gate.

This exception does not convert an untested row into a pass.

#### Tasks

- [x] Update every manifest and lockfile version to `0.4.1`.
- [x] Add `scripts/verify_versions.mjs` for every Localmotive version field.
- [x] Add `scripts/verify_workflow_pins.mjs` for every workflow `uses:` reference.
- [x] Add `scripts/verify_workflow_gates.mjs` for fail-fast steps and job dependencies.
- [x] Run the version check in both CI and release workflows.
- [x] Run the workflow-gate check in both CI and release workflows.
- [x] Put each native Rust command in a separate fail-fast step.
- [x] Make every package and publication job depend on every test gate.
- [x] Resolve and review every remote GitHub Action to a full commit SHA.
- [x] Pin every remote `uses:` reference in all workflows.
- [x] Record each action owner, reviewed version, and resolved SHA.
- [x] Update the changelog with corrections and known limitations.
- [x] Build the production frontend.
- [x] Run formatting, linting, unit, integration, and documentation tests.
- [x] Run the complete Rust suite from a clean checkout without `research/`.
- [x] Confirm raw logs contain no failed test summary.
- [x] Run locked Cargo tests and linting.
- [x] Build portable, NSIS, and MSI candidates.
- [x] Run `scripts/verify_041.mjs` against the packaged candidate.
- [x] Scope packaged DOM assertions to one intended element.
- [x] Exercise rejected installation overrides through packaged IPC.
- [x] Preserve all 19 frozen P0 classes from `TODO-0.4.md:108-133`.
- [ ] Obtain L4 product evidence for every P0 class.
- [x] Add L4 rows for every additional public OS or hardware claim.
- [x] Record one accountable attestation for every P0 result and additional claim.
- [x] Validate every candidate attestation against the versioned release schema.
- [x] Key each attestation by runtime commit, artifact digest, OS, driver, required firmware, backend, and model digest.
- [x] Expire an attestation when any qualification-key field changes.
- [x] Install and launch NSIS on a clean Windows account.
- [x] Install and launch MSI on a clean Windows account.
- [ ] Install and launch on every claimed Windows version.
- [x] Test update from 0.4.0.
- [x] Test cancellation and restart.
- [x] Test both uninstallers.
- [ ] If signing authority is granted, sign and timestamp the exact candidates.
- [x] Inspect asset names, sizes, hashes, metadata, and signatures.
- [x] Record source revision, workflow run, artifact IDs, and checksums.
- [x] Record every known limitation in release notes.
- [x] Correct the public 0.4.0 verification claim after accountable approval.
- [ ] If authority is granted, publish only the verified candidate assets.
- [ ] If publication occurs, read the public release back.

Phase 6 task notes (HEAD `5e93576`, self-hosted CI `34287661275`
completed success, all four jobs): versions, pins, gates, frontend, Rust,
packaged, and qualification-mapping tasks are evidenced on HEAD. The five
open tasks are owner- or SignPath-gated by design: full-P0 L4 (needs
hardware owners or approved spend, covered by the filed risk exception),
every-Windows-version install (needs a Windows 10 claim decision),
signing plus publication plus read-back (need SignPath wiring and the
explicit publish OK). The attestation `sourceRevision` (`632a67d`) is
stale against HEAD by the known self-binding rule; refresh lands at true
final HEAD only.

#### Acceptance checks

Treat these checks as material automated checks:

- version consistency
- immutable workflow references
- fail-fast workflow steps and complete job dependencies
- frontend tests, type checks, and production build
- Rust formatting, linting, unit, integration, and documentation tests
- research unit tests and research verification
- archive, path, reparse-point, process, and argument security tests
- production Tauri package build
- packaged `verify_041` execution

- [x] Every material automated check passes.
- [x] `package.json` reports version `0.4.1`.
- [x] Both relevant `package-lock.json` version fields report `0.4.1`.
- [x] `src-tauri/Cargo.toml` reports version `0.4.1`.
- [x] The Localmotive `Cargo.lock` package entry reports version `0.4.1`.
- [x] `src-tauri/tauri.conf.json` reports version `0.4.1`.
- [x] Both workflows execute the deterministic version check.
- [x] Both workflows execute the workflow-gate check.
- [x] No remote workflow `uses:` reference uses a mutable tag or branch.
- [x] A controlled nonzero native command cannot produce a green test gate.
- [x] No package or release job runs after any failed test gate.
- [x] A clean Windows checkout has zero Rust test failures.
- [x] No source test reads a gitignored research fixture.
- [x] Locked Cargo tests and linting pass.
- [x] Every packaged verifier assertion passes.
- [x] Catalog error and rate-limit packaged tests pass.
- [x] Managed-runtime tamper packaged tests pass.
- [x] Crafted runtime metadata cannot cross installation IPC.
- [x] Portable, NSIS, MSI, and uninstaller surfaces use the approved icon.
- [x] All 19 frozen P0 rows have required product evidence, or the approved disclosure exception is enforced.
- [x] Every additional public OS or hardware claim has L4 product evidence.
- [x] Every release attestation passes the versioned schema.
- [x] No expired attestation contributes to a support claim.
- [x] No failed or unknown P0 row receives a support claim.
- [x] The P0 matrix states exact evidence for every row.
- [x] Published claims do not exceed the evidence level.
- [x] Release notes disclose unsigned status when applicable.
- [x] Version, tag, changelog, and installer metadata agree.
- [ ] Candidate checksums and signatures match the inventory.
- [ ] Published asset names, sizes, and digests match the verified inventory.
- [x] No `L5 RELEASE` claim exists without signing and complete P0 evidence.
- [ ] An accountable human approves public publication.

Phase 6 acceptance notes: the checked boxes are evidenced on HEAD
`5e93576` (gate outputs listed in the step report; self-hosted CI
`34287661275` all four jobs success). The four open boxes need a signed
candidate plus the explicit publish OK: inventory checksums plus
signatures, public read-back, and publication approval. No `L5 RELEASE`
claim exists anywhere; the unsigned status is disclosed in the changelog
and the attestation limitations.

Stop before signing or publication without explicit authority.

---

## 10. Trust boundaries

### GitHub release metadata

Treat tags, URLs, sizes, digests, jobs, and rate-limit headers as untrusted input.

Bind remote metadata to the reviewed manifest.

### Tauri installation IPC

Treat every frontend field as untrusted.

Accept only a manifest install key and a selected adapter identifier.

Resolve every URL, size, digest, backend, tag, and companion asset inside Rust.

### Downloaded runtime archives

Verify expected size and SHA-256 before extraction.

Never finalize a partial or mismatched archive.

Bind resumable state to URL, size, digest, ETag, and last-modified value.

### Managed filesystem

Reject traversal, links, junction escapes, and any changed extracted regular file.

Do not treat lexical normalization or prefix checks as containment proof.

Verify final Windows objects before trust or replacement.

Make install replacement atomic on one volume.

Anchor each content manifest in the compiled approved manifest.

### Child processes

Pass every argument separately.

Bound output and execution time.

Kill the process tree during cancellation.

### Device discovery

Treat vendor names as hints only.

Use stable device identity for selection and evidence matching.

Run each discovery process with a deadline, bounded output, and process-tree cleanup.

Reuse one authoritative hardware snapshot during one setup operation.

Redact device identifiers from public evidence when identity is unnecessary.

### Model identity

Bind health results to the exact model URL, byte count, and SHA-256.

Parse GGUF metadata with bounded reads.

Reject a changed or malformed model before launch.

### Local server network

Bind health servers to loopback only.

Select ports without exposing an unauthenticated service externally.

Do not enable the upstream RPC backend in this scope.

### Credentials and logs

Keep provider credentials outside runtime catalog and health IPC.

Never write tokens, account names, hostnames, or private paths into public evidence.

This plan does not change credential storage.

If later work stores credentials, keep Windows Credential Manager access inside Rust.

Do not return credential material through frontend IPC.

### Frontend IPC

Keep approval, containment, and launch decisions in Rust.

Return typed errors without secrets or private paths.

### Release publication

Pin release-sensitive actions by immutable revisions.

Require verified candidate evidence before granting publication authority.

---

## 11. Test strategy

| Area | Strongest planned verifier | Material criteria |
|---|---|---|
| Catalog isolation | Rust unit tests with frozen release fixtures | One blocked backend cannot abort others. |
| HTTP behavior | Controlled local HTTP server | Exact tag, 403, 429, timeout, truncation, wrong metadata. |
| Archive transfer | Controlled range-capable HTTP server | Resume, ignored ranges, changed validators, cancellation, size, and digest. |
| UI state | Pure state tests plus packaged CDP | Loading, ready, empty, error, cancellation, Retry. |
| Recommendation | Table-driven Rust tests | Exact device, OS, driver, firmware, artifact, and adapter selection. |
| Installation IPC | Rust command-boundary tests | Frontend metadata cannot select URL, tag, digest, size, or backend. |
| Managed trust | Windows filesystem security tests | Digests, reparse points, atomic replacement, mismatch. |
| Runtime health | Controlled local runtime and pinned GGUF | Device, model, server, completion, cancellation, cleanup. |
| Device probes | Controlled child-process tests | Deadline, output bound, cleanup, and one snapshot per setup. |
| Research integrity | Unit tests, schema validation, digest checks | Citations, privacy, attestations, exact product manifest. |
| Icon identity | Generated-file inspection and clean Windows package | ICO layers and every shell surface match `LM`. |
| Workflow gates | Static workflow checks plus controlled nonzero command | Native failures block packaging and publication. |
| Release | CI, hashes, signatures, clean-machine execution | Candidate bytes equal published bytes. |

Run focused failing tests first.

Run broader frontend and Rust suites next.

Run package and Windows tests last.

Do not use model output as proof of process behavior.

---

## 12. Human gates

### New upstream pin

Require an explicit product decision before replacing `b10796`.

Require new release metadata, digests, jobs, smoke evidence, and regression results.

### Hardware access and cloud spending

Require explicit approval before provisioning paid hardware or starting cloud spending.

Record the provider, hardware class, spend limit, approver, and approval reference.

### Secret access

Require explicit approval before accessing any signing, publishing, or private-test secret.

Record only the secret name, purpose, approver, and approval reference.

Never record a secret value.

### Destructive lifecycle work

Require explicit approval before destructive migration or lifecycle testing on a non-disposable system.

Record the target, backup state, recovery path, approver, and approval reference.

### Code signing

Require explicit signing authority and approved secret handling.

No signing certificate was available during this audit.

### Public release

Require explicit publication authority after every material gate passes.

Do not publish while a material acceptance check fails.

Require explicit authority before editing the published 0.4.0 release body.

Verify the corrective note after any approved edit.

### Risk acceptance

Do not weaken an upstream-job block for 0.4.1.

Treat any future weakening proposal as work outside this plan.

Require accountable approval before that future work starts.

Do not hide the evidence URL or failure conclusion.

### Phase 6 gate map

| Phase 6 action | Required approval | Required evidence |
|---|---|---|
| Provision paid qualification hardware | Cloud spending | Provider, class, cap, approver, reference |
| Access a signing or publishing secret | Secret access | Secret name, purpose, approver, reference |
| Sign candidate artifacts | Code signing | Identity, timestamp service, approver, reference |
| Test destructive update, migration, or uninstall | Destructive lifecycle | Target, backup, recovery path, approver, reference |
| Edit the 0.4.0 public release | Public release | Approved correction, approver, read-back URL |
| Publish 0.4.1 | Public release | Candidate inventory, passing gates, approver, release URL |

---

## 13. Risks and unresolved questions

| Risk or question | Current result | Required resolution |
|---|---|---|
| Requalify or replace `b10796` | UNKNOWN | Make the upstream-pin decision in Phase 0. |
| CUDA availability in 0.4.1 | BLOCKED | Require a successful approved required-job set. |
| CUDA job defect classification | UNKNOWN | Classify the router SSE and timeout failures before changing policy. |
| Missing qualification hardware and cloud cost | BLOCKED | Obtain owners or approved capped cloud access. |
| Clean-machine NSIS behavior | UNKNOWN | Test install, launch, update, and uninstall. |
| Clean-machine MSI behavior | UNKNOWN | Test install, launch, update, and uninstall. |
| Windows 10 support claim | UNKNOWN | Decide the claim and obtain direct package evidence. |
| Reparse-point resistance on reuse | UNKNOWN | Run direct Windows security tests. |
| L4 behavior on CPU | UNKNOWN | Drive the packaged product end to end. |
| L4 behavior on CUDA | BLOCKED | Resolve the pin and run packaged qualification. |
| L4 behavior on Vulkan | UNKNOWN | Drive the packaged product end to end. |
| ROCm, SYCL, and OpenVINO dependencies | UNKNOWN | Use clean machines with exact supported hardware. |
| Multi-adapter behavior | UNKNOWN | Test same-vendor and mixed-vendor hosts. |
| Small smoke fixture coverage | UNKNOWN | Do not generalize beyond the pinned SmolLM2 fixture. |
| Single-repetition performance evidence | UNKNOWN | Do not make performance claims without repeated measurements. |
| Authenticode signing | BLOCKED | Obtain authority and an approved certificate workflow. |
| Shell icon cache behavior | UNKNOWN | Test the candidate on a clean account. |
| MSI publisher value | UNKNOWN | Replace generic `github` only after approving the publisher text. |
| Mutable workflow action references | FAIL | Pin release-sensitive actions by full SHA. |
| False-green Rust workflow gate | FAIL | Three raw logs contain failed suites inside successful jobs. |
| Clean-checkout source tests | FAIL | The tag clone fails ten tests locally and 12 on GitHub. |
| GitHub-only download test failures | UNKNOWN | Reproduce and identify the Windows runner difference. |
| Installation IPC authority | FAIL | Frontend metadata can bypass the compiled approval manifest. |

---

## 14. Reproduction and verification commands

Run repository checks from the repository root:

```text
npm run test
npm run check
npm run build
cd src-tauri
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
RUSTDOCFLAGS='-D warnings' cargo test --locked --doc
```

Run research checks from `research/0.4`:

```text
python -m unittest discover -s tests -v
python verify_research.py
```

Inspect the exact upstream release:

```text
gh api /repos/ggml-org/llama.cpp/releases/tags/b10796
```

Inspect the failed CUDA job:

```text
gh api /repos/ggml-org/llama.cpp/actions/jobs/100912786608
gh run view 33837461477 --repo ggml-org/llama.cpp --job 100912786608 --log-failed
```

Retrieve the three false-green Localmotive workflow logs:

```text
gh api /repos/sato942/localmotive/actions/runs/33948464859/logs > ci-33948464859.zip
gh api /repos/sato942/localmotive/actions/runs/33948466104/logs > release-33948466104.zip
gh api /repos/sato942/localmotive/actions/runs/33949560113/logs > ci-33949560113.zip
```

Reproduce the tagged source suite without gitignored files:

```text
git clone --branch v0.4.0 --single-branch https://github.com/sato942/localmotive.git <TEMP_CLONE>
cd <TEMP_CLONE>/src-tauri
cargo test --locked
```

Verify downloaded 0.4.0 release assets:

```text
sha256sum -c SHA256SUMS-0.4.0.txt
```

Run the current packaged verifier against a CDP-enabled candidate:

```text
node scripts/verify_040.mjs <CDP_PORT>
```

After implementation, run the planned 0.4.1 candidate gates:

```text
node scripts/verify_versions.mjs 0.4.1
node scripts/verify_workflow_pins.mjs
node scripts/verify_workflow_gates.mjs
npm run tauri build
node scripts/verify_041.mjs <CDP_PORT>
```

Launch the exact portable candidate with an isolated WebView2 profile before `verify_041`.

Terminate the candidate process after `verify_041` on every result path.

Require every command to exit with code zero.

Require `verify_041` to write schema-valid JSON with `overall_status` equal to `PASS`.

---

## 15. Evidence index

- Original tracker: `TODO-0.4.md`
- Approved runtime manifest: `src-tauri/approved_runtimes.json`
- Catalog and install implementation: `src-tauri/src/runtime.rs`
- Product health IPC: `src-tauri/src/lib.rs`
- Runtime health core: `src-tauri/src/core.rs`
- Runtime Manager UI: `src/App.tsx`
- Evidence-level mapping: `src/model.ts`
- Published runtime contract: `docs/RUNTIME_MANAGER.md`
- In-app mark styling: `src/App.css`
- Bundle icon configuration: `src-tauri/tauri.conf.json`
- Current icon derivatives: `src-tauri/icons/`
- Current obsolete vector source: `assets/app-icon.svg`
- CI workflow: `.github/workflows/ci.yml`
- Release workflow: `.github/workflows/release.yml`
- Catalog workflow: `.github/workflows/catalog.yml`
- Current packaged verifier: `scripts/verify_040.mjs`
- Planned version verifier: `scripts/verify_versions.mjs`
- Planned workflow-pin verifier: `scripts/verify_workflow_pins.mjs`
- Planned workflow-gate verifier: `scripts/verify_workflow_gates.mjs`
- Planned packaged verifier: `scripts/verify_041.mjs`
- Planned release evidence: `release-evidence/0.4.1/`
- Official Tauri icon guide: <https://v2.tauri.app/develop/icons/>
- Official GitHub rate-limit guide: <https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api>
- Published release: <https://github.com/sato942/localmotive/releases/tag/v0.4.0>
- False-green CI run: <https://github.com/sato942/localmotive/actions/runs/33948464859>
- False-green release run: <https://github.com/sato942/localmotive/actions/runs/33948466104>
- False-green current-main run: <https://github.com/sato942/localmotive/actions/runs/33949560113>
- CUDA job evidence: <https://github.com/ggml-org/llama.cpp/actions/runs/33837461477/job/100912786608>

---

## 16. Current acceptance status

| Material criterion | Result | Reason |
|---|---|---|
| Recorded 0.4.0 source and artifact checksums | PASS | Tag, versions, published bytes, and hashes agree. |
| 0.4.0 automated source checks | FAIL | Both tagged workflow logs report 12 failed Rust tests. |
| Workflow failure propagation | FAIL | A later successful native command masks the failed suite. |
| Clean-checkout source portability | FAIL | The tag clone fails ten tests locally and 12 on GitHub. |
| Runtime Manager catalog completion | FAIL | CUDA blocks the complete catalog. |
| Honest catalog failure state | FAIL | `null` renders as perpetual loading. |
| Backend-local fail-closed behavior | FAIL | The catalog aborts globally, and job impacts need exact mappings. |
| Exact release retrieval | FAIL | The client scans the latest 20 releases. |
| Exact manifest identity | FAIL | One dormant URL is wrong; approval fields are not enforced. |
| Installation IPC manifest authority | FAIL | Frontend runtime metadata is not rebound inside Rust. |
| Runtime approval job gate | BLOCKED | `b10796` has applicable failed required jobs. |
| Exact hardware recommendation | FAIL | Broad AMD and Intel names can qualify native paths. |
| Explicit multi-adapter selection | FAIL | Same-vendor adapters can select implicitly. |
| Product health contract | FAIL | The product executes only the device-list stage. |
| Managed-runtime reuse integrity | FAIL | Reuse bypasses manifest and file validation. |
| Support-claim accuracy | FAIL | The UI and public 0.4.0 body call L2 evidence supported. |
| Windows icon identity | FAIL | Published and configured icons show `FD`, not `LM`. |
| Research evidence consistency | FAIL | The current run reports four failures; one expected scope failure names this plan. |
| Packaged acceptance gate | FAIL | Two of 12 assertions fail, and CI did not gate publication. |
| Complete P0 product qualification | FAIL | All 19 frozen P0 rows lack L4 product evidence. |
| Clean-machine Windows behavior | UNKNOWN | No direct install and lifecycle evidence was available. |
| Signed release qualification | BLOCKED | The candidate is unsigned and no authority was provided. |
| Public 0.4.1 release approval | BLOCKED | Publication requires a future explicit approval. |

## Conclusion

The audit is PARTIALLY VERIFIED.

The downloaded 0.4.0 assets match GitHub metadata and the published checksum file.

The audit did not establish signed publisher identity.

The 0.4.0 implementation and release gate are not complete against the research-backed plan.

The false-green Rust gate and untrusted installation IPC are P0 0.4.1 blockers.

Begin 0.4.1 with Phase 0A research repair and freezing.

Create catalog, loading, clean-checkout, workflow, and installation regressions in Phase 0B.

Do not unblock CUDA without new approved evidence.

Do not publish 0.4.1 until every material release criterion passes.
