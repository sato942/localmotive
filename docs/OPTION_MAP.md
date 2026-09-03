# llama-server Option Map

Source: `LLAMA-SERVER-README.md`. GGUF Pilot promotes options according to how often they matter, how safely they can be changed, and whether a wrong value can make a model fail to load.

## Shown immediately

### Identity and endpoint

- Runtime executable
- Host, port, and API alias
- Profile name

These define what process is started and how clients address it.

### Everyday performance

- Context length and parallel slots
- GPU layers and CPU MoE layers
- Flash attention: automatic/on/off
- Model loading mode
- KV cache K/V types and GPU offload
- Smart memory fitting

These are the highest-value controls for fit, latency, and throughput. Their labels include short operational guidance.

### Acceleration and model companions

- Speculative method reported by the selected runtime
- Draft/DSpark/DFlash/EAGLE/MTP companion
- Maximum draft depth
- Vision projector

These are model relationships, not generic tuning knobs, so they remain visible.

### Chat behavior

- Jinja templates
- Reasoning mode and effort
- Prompt cache reuse

These directly affect compatibility with chat, tool-calling, and reasoning clients.

## Advanced, grouped by job

### CPU, batching, and throughput

Generation/prompt threads, batch/uBatch, dense FFN CPU placement, HTTP threads, continuous batching, and warmup.

### GPU placement and memory fitting

Split mode, tensor split proportions, main GPU, device list, fit margin/minimum context, and lazy tensor loading.

### Prompt cache and server lifecycle

Cache RAM, reuse chunk, context checkpoints, context shift, sleep-on-idle, request timeout, SSE pings, slots endpoint, metrics, and WebUI.

### Speculative fine tuning

Draft minimum/maximum, probability thresholds, draft GPU layers and cache types, plus n-gram lookup, draft size, and hit thresholds.

### Sampling defaults

Temperature, top-k, top-p, min-p, repeat penalty/window, seed, and DRY controls. These are advanced because API clients commonly override them per request.

### Vision and multimodal

Projector offload/device and image token budgets.

### Network and security

CORS origins, API key file, and TLS key/certificate. LAN binding is rejected unless authentication and restricted CORS are configured. Loopback-only wildcard CORS is rejected without an API key file. DNS hostnames are rejected as bind targets; use an IP literal or `localhost`. Secrets are referenced by file and are not placed directly in the process command.

### Templates, adapters, and low-level overrides

Reasoning budget/history, custom chat template, LoRAs, tensor/model-metadata overrides, logging, and raw extra arguments.

## Deliberately kept out of dedicated controls

The raw-extra-arguments field accepts only self-contained `--flag` or
`--flag=value` tokens. GGUF Pilot checks each token against the selected
runtime's current `--help` output before process creation.

The backend rejects positional values and overrides of typed profile fields.
The backend also rejects these privileged capability families:

- inline credentials;
- model and projector downloads;
- RPC workers and router directories;
- built-in tools and agent mode;
- MCP process definitions and the WebUI MCP proxy;
- arbitrary static, UI-configuration, and slot-save paths.

Other advertised experimental options can use the raw field. Use dedicated
typed fields whenever GGUF Pilot provides them.
