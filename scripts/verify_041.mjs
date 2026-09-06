import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import {
  mkdir,
  readFile,
  stat,
  writeFile,
} from "node:fs/promises";
import { basename, dirname, resolve, sep } from "node:path";
import os from "node:os";
import process from "node:process";
import Ajv from "ajv";
import WebSocket from "ws";

const port = Number.parseInt(process.argv[2] ?? "", 10);
const artifactPath = process.argv[3] ? resolve(process.argv[3]) : "";
const outputPath = resolve(
  process.argv[4] ?? "release-evidence/0.4.1/packaged-verification.json",
);
const isolationRoot = process.env.LOCALMOTIVE_VERIFY_ISOLATED_ROOT
  ? resolve(process.env.LOCALMOTIVE_VERIFY_ISOLATED_ROOT)
  : "";
const cdpConnectTimeoutMs = Math.max(
  500,
  Number.parseInt(process.env.LOCALMOTIVE_VERIFY_CDP_TIMEOUT_MS ?? "45000", 10) || 45_000,
);
const startedAt = new Date().toISOString();
const checks = [];
let client;
let catalogHarnessResponses;
let baselineSetup;
let installedRuntime;
let originalRuntimeBytes;
let runtimeWasRepaired = false;

function safeGit(args, fallback = "") {
  try {
    return execFileSync("git", args, {
      cwd: process.cwd(),
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  } catch {
    return fallback;
  }
}

const sourceRevision = (
  process.env.LOCALMOTIVE_SOURCE_REVISION ?? safeGit(["rev-parse", "HEAD"])
).toLowerCase();
const sourceStatus = safeGit(["status", "--porcelain", "--untracked-files=all"], "UNKNOWN");
const sourceDirty = sourceStatus !== "";

function redactString(value) {
  let result = value;
  if (isolationRoot) {
    result = result.replaceAll(isolationRoot, "[ISOLATED_ROOT]");
    result = result.replaceAll(isolationRoot.replaceAll("\\", "/"), "[ISOLATED_ROOT]");
  }
  result = result.replace(/[A-Za-z]:[\\/]Users[\\/][^\\/\s]+/gi, "[USER_HOME]");
  return result;
}

function sanitize(value) {
  if (typeof value === "string") return redactString(value);
  if (Array.isArray(value)) return value.map(sanitize);
  if (value && typeof value === "object") {
    return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, sanitize(item)]));
  }
  return value;
}

function addCheck(id, criterion, status, evidence = {}, reason = "") {
  checks.push({
    id,
    criterion,
    status,
    evidence: sanitize(evidence),
    reason: redactString(reason),
  });
}

async function runCheck(id, criterion, verifier) {
  try {
    const evidence = (await verifier()) ?? {};
    addCheck(id, criterion, "PASS", evidence, "");
    return true;
  } catch (error) {
    addCheck(id, criterion, "FAIL", {}, error instanceof Error ? error.message : String(error));
    return false;
  }
}

function requireCondition(condition, message) {
  if (!condition) throw new Error(message);
}

const HEALTH_STAGES = [
  "device_enumeration",
  "backend_operations",
  "pinned_model_load",
  "loopback_server_health",
  "deterministic_completion",
  "cancellation",
  "process_and_temporary_file_cleanup",
];

function successfulHealthEvidence(health) {
  requireCondition(health?.passed === true, "The managed runtime health contract did not pass");
  requireCondition(health.modelSha256 === "e3131339bf4e8065265593d4fd8f7bb7ff2d3abff1edb5618aa1197b89cad9f5", "Health used an unexpected model digest");
  requireCondition(Array.isArray(health.stages) && health.stages.length === HEALTH_STAGES.length, "Health did not return seven stages");
  requireCondition(
    health.stages.every((stage, index) => stage.stage === HEALTH_STAGES[index] && stage.status === "PASS"),
    "Health stage order or status did not match the seven-stage contract",
  );
  const completion = health.stages[4]?.completion;
  requireCondition(completion?.temperature === 0, "Deterministic completion did not use temperature zero");
  requireCondition(completion?.requestedTokens === completion?.observedTokens, "Deterministic completion token counts differ");
  requireCondition(completion?.expectedOutputSha256 === completion?.observedOutputSha256, "Deterministic completion digest differs");
  return {
    model_sha256: health.modelSha256,
    stages: health.stages.map((stage) => ({ stage: stage.stage, status: stage.status })),
    completion_sha256: completion.observedOutputSha256,
  };
}

async function sha256File(path) {
  const bytes = await readFile(path);
  return createHash("sha256").update(bytes).digest("hex");
}

class CdpClient {
  constructor(socket, defaultTimeoutMs = 30_000) {
    this.socket = socket;
    this.defaultTimeoutMs = defaultTimeoutMs;
    this.nextId = 1;
    this.pending = new Map();
    socket.addEventListener("message", (event) => {
      const message = JSON.parse(event.data);
      if (!message.id) return;
      const pending = this.pending.get(message.id);
      if (!pending) return;
      this.pending.delete(message.id);
      clearTimeout(pending.timer);
      if (message.error) pending.reject(new Error(message.error.message));
      else pending.resolve(message.result);
    });
    socket.addEventListener("close", () => {
      for (const pending of this.pending.values()) {
        clearTimeout(pending.timer);
        pending.reject(new Error("The CDP connection closed"));
      }
      this.pending.clear();
    });
  }

  send(method, params = {}, timeoutMs = this.defaultTimeoutMs) {
    const id = this.nextId++;
    return new Promise((resolvePromise, rejectPromise) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        rejectPromise(new Error(`CDP ${method} exceeded ${timeoutMs} ms`));
      }, timeoutMs);
      this.pending.set(id, { resolve: resolvePromise, reject: rejectPromise, timer });
      this.socket.send(JSON.stringify({ id, method, params }));
    });
  }

  async evaluate(expression, timeoutMs = this.defaultTimeoutMs) {
    const response = await this.send(
      "Runtime.evaluate",
      {
        expression,
        awaitPromise: true,
        returnByValue: true,
        userGesture: true,
      },
      timeoutMs,
    );
    if (response.exceptionDetails) {
      const description =
        response.exceptionDetails.exception?.description ??
        response.exceptionDetails.text ??
        "Page evaluation failed";
      throw new Error(description);
    }
    return response.result?.value;
  }

  close() {
    this.socket.close();
  }
}

async function connectToCandidate() {
  const deadline = Date.now() + cdpConnectTimeoutMs;
  let lastError = "No CDP page was found";
  while (Date.now() < deadline) {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/json/list`);
      requireCondition(response.ok, `CDP endpoint returned HTTP ${response.status}`);
      const pages = await response.json();
      const page = pages.find(
        (entry) =>
          entry.type === "page" &&
          typeof entry.webSocketDebuggerUrl === "string" &&
          (/tauri\.localhost/i.test(entry.url ?? "") || /Localmotive/i.test(entry.title ?? "")),
      );
      requireCondition(page, "The CDP endpoint did not list the Localmotive page");
      const socket = new WebSocket(page.webSocketDebuggerUrl);
      await new Promise((resolvePromise, rejectPromise) => {
        const timer = setTimeout(() => rejectPromise(new Error("CDP WebSocket connection timed out")), 10_000);
        socket.addEventListener("open", () => {
          clearTimeout(timer);
          resolvePromise();
        }, { once: true });
        socket.addEventListener("error", () => {
          clearTimeout(timer);
          rejectPromise(new Error("CDP WebSocket connection failed"));
        }, { once: true });
      });
      const connected = new CdpClient(socket);
      await connected.send("Runtime.enable");
      return connected;
    } catch (error) {
      lastError = error instanceof Error ? error.message : String(error);
      await new Promise((resolvePromise) => setTimeout(resolvePromise, 500));
    }
  }
  throw new Error(lastError);
}

async function waitFor(expression, message, timeoutMs = 30_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await client.evaluate(`Boolean(${expression})`)) return;
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 100));
  }
  throw new Error(message);
}

async function clickUnique(selector, expectedText = null) {
  const result = await client.evaluate(`(() => {
    const nodes = [...document.querySelectorAll(${JSON.stringify(selector)})]
      .filter((node) => ${expectedText === null ? "true" : `node.textContent.trim() === ${JSON.stringify(expectedText)}`});
    if (nodes.length !== 1) return { clicked: false, count: nodes.length };
    nodes[0].click();
    return { clicked: true, count: 1 };
  })()`);
  requireCondition(result?.clicked, `${selector} matched ${result?.count ?? 0} intended elements`);
  return result;
}

async function invoke(command, args = {}, timeoutMs = 120_000) {
  const result = await client.evaluate(`(async () => {
    try {
      const value = await window.__TAURI_INTERNALS__.invoke(
        ${JSON.stringify(command)},
        ${JSON.stringify(args)}
      );
      return { ok: true, value };
    } catch (error) {
      return { ok: false, error: String(error) };
    }
  })()`, timeoutMs);
  if (!result?.ok) throw new Error(result?.error ?? `Tauri command ${command} failed`);
  return result.value;
}

async function rejectedInvoke(command, args) {
  return client.evaluate(`(async () => {
    try {
      const value = await window.__TAURI_INTERNALS__.invoke(
        ${JSON.stringify(command)},
        ${JSON.stringify(args)}
      );
      return { rejected: false, value };
    } catch (error) {
      const structured = error && typeof error === 'object'
        ? error
        : { kind: 'unknown', message: String(error), retryAfterSeconds: null };
      return {
        rejected: true,
        error: structured,
        errorText: typeof structured.message === 'string' ? structured.message : String(error),
      };
    }
  })()`, 120_000);
}

const fixtureCatalog = {
  tag: "b10816",
  publishedAt: "2026-09-04T00:00:00Z",
  options: [
    {
      id: "verify-cpu",
      label: "CPU verification fixture",
      backend: "cpu",
      installKey: "cpu",
      description: "Hermetic packaged-verifier fixture.",
      compatibility: "x64 CPU fixture",
      asset: {
        name: "llama-b10816-bin-win-cpu-x64.zip",
        browserDownloadUrl: "https://example.invalid/cpu.zip",
        size: 1,
        digest: `sha256:${"0".repeat(64)}`,
      },
      companionAsset: null,
      recommended: true,
    },
    {
      id: "verify-vulkan",
      label: "Vulkan verification fixture",
      backend: "vulkan",
      installKey: "vulkan",
      description: "Hermetic packaged-verifier fixture.",
      compatibility: "x64 Vulkan fixture",
      asset: {
        name: "llama-b10816-bin-win-vulkan-x64.zip",
        browserDownloadUrl: "https://example.invalid/vulkan.zip",
        size: 1,
        digest: `sha256:${"1".repeat(64)}`,
      },
      companionAsset: null,
      recommended: false,
    },
  ],
  availability: [
    {
      installKey: "cuda-13.3",
      backend: "cuda",
      status: "blocked",
      reason: "Verifier-injected upstream CUDA job failure.",
      blockingJobs: ["server-cuda"],
      evidenceUrls: ["https://github.com/ggml-org/llama.cpp/actions/runs/33906999326/job/101134163906"],
    },
  ],
  origin: "cache",
  warning: "Hermetic packaged-verifier fixture",
  recommendationReason: "CPU fallback for packaged UI verification.",
};

async function installCatalogHarness(setup) {
  catalogHarnessResponses = {
    ready: { ...setup, catalog: fixtureCatalog, catalogError: null },
    empty: {
      ...setup,
      catalog: { ...fixtureCatalog, options: [] },
      catalogError: null,
    },
    error: {
      ...setup,
      catalog: null,
      catalogError: {
        kind: "invalid_response",
        message: "Verifier-injected catalog failure",
        retryAfterSeconds: null,
      },
    },
    rate: {
      ...setup,
      catalog: null,
      catalogError: {
        kind: "rate_limited",
        message: "Verifier-injected GitHub rate limit; retry after 60 seconds",
        retryAfterSeconds: 60,
      },
    },
  };
  const result = await client.evaluate(`(() => {
    const host = document.querySelector('.app-shell');
    const fiberKey = host && Object.keys(host).find((key) => key.startsWith('__reactFiber$'));
    let fiber = fiberKey ? host[fiberKey] : null;
    while (fiber) {
      const hooks = [];
      let hook = fiber.memoizedState;
      while (hook && hooks.length < 100) {
        hooks.push(hook);
        hook = hook.next;
      }
      const hardwareIndex = hooks.findIndex((entry) =>
        entry?.memoizedState?.architecture &&
        Array.isArray(entry.memoizedState.adapters) &&
        typeof entry?.queue?.dispatch === 'function'
      );
      if (hardwareIndex >= 0) {
        const catalogHook = hooks[hardwareIndex + 2];
        const errorHook = hooks[hardwareIndex + 3];
        const loadingHook = hooks[hardwareIndex + 4];
        if (
          Array.isArray(catalogHook?.memoizedState?.options) &&
          typeof catalogHook?.queue?.dispatch === 'function' &&
          typeof errorHook?.queue?.dispatch === 'function' &&
          typeof loadingHook?.memoizedState === 'boolean' &&
          typeof loadingHook?.queue?.dispatch === 'function'
        ) {
          window.__LM_VERIFY_SET_CATALOG = catalogHook.queue.dispatch;
          window.__LM_VERIFY_SET_CATALOG_ERROR = errorHook.queue.dispatch;
          window.__LM_VERIFY_SET_CATALOG_LOADING = loadingHook.queue.dispatch;
          return { ok: true, hardwareIndex, hookCount: hooks.length };
        }
      }
      fiber = fiber.return;
    }
    return { ok: false, reason: 'The App catalog state hooks were not found' };
  })()`);
  requireCondition(result?.ok, result?.reason ?? "The React catalog harness failed");
}

async function setCatalogScenario(scenario) {
  requireCondition(catalogHarnessResponses, "The catalog harness responses are unavailable");
  const response = scenario === "pending" ? catalogHarnessResponses.ready : catalogHarnessResponses[scenario];
  requireCondition(response, `Unknown catalog harness scenario: ${scenario}`);
  const catalog = scenario === "pending" ? fixtureCatalog : response.catalog;
  const error = scenario === "pending" ? null : response.catalogError;
  const loading = scenario === "pending";
  const emptyCatalog = catalogHarnessResponses.empty.catalog;
  const result = await client.evaluate(`(() => {
    if (
      typeof window.__LM_VERIFY_SET_CATALOG !== 'function' ||
      typeof window.__LM_VERIFY_SET_CATALOG_ERROR !== 'function' ||
      typeof window.__LM_VERIFY_SET_CATALOG_LOADING !== 'function'
    ) {
      return { ok: false, reason: 'The React catalog harness is not installed' };
    }
    window.__LM_VERIFY_SET_CATALOG(${JSON.stringify(catalog)});
    window.__LM_VERIFY_SET_CATALOG_ERROR(${JSON.stringify(error)});
    window.__LM_VERIFY_SET_CATALOG_LOADING(${JSON.stringify(loading)});
    if (${JSON.stringify(scenario)} === 'pending') {
      window.__LM_VERIFY_RESOLVE = () => {
        delete window.__LM_VERIFY_RESOLVE;
        window.__LM_VERIFY_SET_CATALOG(${JSON.stringify(emptyCatalog)});
        window.__LM_VERIFY_SET_CATALOG_ERROR(null);
        window.__LM_VERIFY_SET_CATALOG_LOADING(false);
      };
    } else {
      delete window.__LM_VERIFY_RESOLVE;
    }
    return { ok: true };
  })()`);
  requireCondition(result?.ok, result?.reason ?? "The catalog scenario could not be selected");
  await new Promise((resolvePromise) => setTimeout(resolvePromise, 50));
  return result;
}

async function setScenarioAndRefresh(scenario) {
  return setCatalogScenario(scenario);
}

async function artifactRecord() {
  requireCondition(artifactPath, "Pass the packaged artifact path as the second argument");
  const details = await stat(artifactPath);
  requireCondition(details.isFile(), "The packaged artifact path is not a file");
  return {
    name: basename(artifactPath),
    size_bytes: details.size,
    sha256: await sha256File(artifactPath),
  };
}

async function writeResult(artifact, hostClass) {
  const result = {
    schema_version: "1.0.0",
    verifier: "scripts/verify_041.mjs",
    source_revision: sourceRevision,
    source_dirty: sourceDirty,
    artifact,
    host_class: hostClass,
    started_at: startedAt,
    finished_at: new Date().toISOString(),
    checks,
    overall_status: checks.every((check) => check.status === "PASS") ? "PASS" : "FAIL",
  };
  const schema = JSON.parse(
    await readFile(resolve("release-evidence/packaged-verification.schema.json"), "utf8"),
  );
  const validate = new Ajv({ allErrors: true, strict: true }).compile(schema);
  if (!validate(result)) {
    result.overall_status = "FAIL";
    addCheck(
      "result.schema",
      "The verification record matches packaged-verification.schema.json.",
      "FAIL",
      { errors: validate.errors ?? [] },
      "The generated record did not match its schema",
    );
    result.checks = checks;
    result.finished_at = new Date().toISOString();
  }
  await mkdir(dirname(outputPath), { recursive: true });
  await writeFile(outputPath, `${JSON.stringify(sanitize(result), null, 2)}\n`, "utf8");
  return result;
}

let artifact = { name: "UNKNOWN", size_bytes: 1, sha256: "0".repeat(64) };
let hostClass = {
  platform: os.platform(),
  release: os.release(),
  architecture: os.arch(),
  cpu_model: os.cpus()[0]?.model ?? "",
  logical_cpus: Math.max(1, os.cpus().length),
  memory_bytes: Math.max(1, os.totalmem()),
  adapters: [],
};

try {
  await runCheck(
    "candidate.artifact",
    "The verifier binds its result to the exact packaged artifact digest.",
    async () => {
      artifact = await artifactRecord();
      return artifact;
    },
  );

  await runCheck(
    "candidate.revision",
    "The source revision is a full Git commit and matches the requested revision.",
    async () => {
      requireCondition(/^[0-9a-f]{40}$/.test(sourceRevision), "The source revision is not a full Git commit");
      const repositoryRevision = safeGit(["rev-parse", "HEAD"]).toLowerCase();
      requireCondition(repositoryRevision === sourceRevision, "The requested revision differs from repository HEAD");
      return { source_revision: sourceRevision };
    },
  );

  await runCheck(
    "candidate.clean-source",
    "The packaged candidate comes from a clean checkout.",
    async () => {
      requireCondition(!sourceDirty, "The source checkout contains tracked or untracked changes");
      return { source_dirty: false };
    },
  );

  const connected = await runCheck(
    "candidate.cdp-connect",
    "The exact packaged candidate exposes one Localmotive WebView through the configured CDP port.",
    async () => {
      requireCondition(Number.isInteger(port) && port > 0 && port <= 65_535, "Pass a valid CDP port as the first argument");
      client = await connectToCandidate();
      const title = await client.evaluate("document.title");
      return { title, port };
    },
  );

  if (connected) {
    await runCheck(
      "ui.branding",
      "The packaged page presents Localmotive without the obsolete product name.",
      async () => {
        const content = await client.evaluate(`({ title: document.title, text: document.body.innerText })`);
        requireCondition(content.title === "Localmotive", `Unexpected document title: ${content.title}`);
        requireCondition(content.text.includes("Localmotive"), "Localmotive is not visible");
        requireCondition(!content.text.includes("GGUF Pilot"), "The obsolete product name is visible");
        return { title: content.title, localmotive_visible: true, obsolete_name_visible: false };
      },
    );

    await runCheck(
      "ipc.runtime-setup",
      "The packaged backend returns hardware and managed-runtime state independently from catalog success.",
      async () => {
        baselineSetup = await invoke("load_runtime_setup", {}, 120_000);
        requireCondition(baselineSetup?.hardware, "Runtime setup omitted hardware data");
        requireCondition(typeof baselineSetup.runtimeRoot === "string", "Runtime setup omitted the runtime root");
        requireCondition(Array.isArray(baselineSetup.managedRuntimes), "Runtime setup omitted managed runtimes");
        hostClass.adapters = (baselineSetup.hardware.adapters ?? []).map((adapter) => ({
          name: String(adapter.name ?? ""),
          vendor: adapter.vendor == null ? null : String(adapter.vendor),
          driver_version: adapter.driver?.value == null ? null : String(adapter.driver.value),
        }));
        return {
          architecture: baselineSetup.hardware.architecture,
          adapter_count: baselineSetup.hardware.adapters?.length ?? 0,
          catalog_result: baselineSetup.catalog ? "ready" : "terminal-error",
          managed_runtime_count: baselineSetup.managedRuntimes.length,
        };
      },
    );

    if (baselineSetup) {
      await runCheck(
        "ipc.runtime-recommendation",
        "The selected backend adapter reaches backend recommendation without creating an unsupported accelerator claim.",
        async () => {
          const adapterId = baselineSetup.hardware.adapters?.[0]?.adapterId ?? null;
          const catalog = await invoke("fetch_runtime_catalog", { adapterId }, 120_000);
          const recommended = catalog.options.filter((option) => option.recommended);
          requireCondition(recommended.length === 1, "The backend did not return one recommendation");
          requireCondition(recommended[0].backend === "cpu", "An accelerator was recommended without an exact L4 record");
          requireCondition(/exact L4 .*compatibility record|no accelerator/i.test(catalog.recommendationReason), "The CPU fallback reason was not explicit");
          return {
            selected_adapter: adapterId === null ? "none" : "one detected adapter",
            backend: recommended[0].backend,
            reason: catalog.recommendationReason,
          };
        },
      );

      await runCheck(
        "ipc.reject-unknown-adapter",
        "Runtime recommendation rejects an adapter identifier absent from the backend hardware snapshot.",
        async () => {
          const result = await rejectedInvoke("fetch_runtime_catalog", {
            adapterId: "luid:ffffffffffffffff:ffffffffffffffff",
          });
          requireCondition(result.rejected, "The backend accepted an unknown adapter identifier");
          requireCondition(result.error?.kind === "invalid_response", "The rejection did not preserve the invalid-response kind");
          requireCondition(/not (?:present )?in the current hardware snapshot/i.test(result.errorText), "The rejection did not identify the hardware-snapshot mismatch");
          return { rejected: true, error: result.error };
        },
      );

      await runCheck(
        "ui.catalog-harness",
        "The verifier seeds catalog state without network dependence.",
        async () => {
          await installCatalogHarness(baselineSetup);
          await clickUnique("button.nav-item", "Runtime");
          await setScenarioAndRefresh("ready");
          await waitFor(
            "document.querySelectorAll('.runtime-option:not(.blocked)').length === 2",
            "The seeded runtime cards did not render",
          );
          return { fixture_tag: fixtureCatalog.tag, option_count: 2 };
        },
      );

      await runCheck(
        "ui.catalog-loading",
        "The packaged UI shows one bounded loading state and disables refresh during retrieval.",
        async () => {
          await setScenarioAndRefresh("pending");
          await waitFor(
            "document.querySelectorAll('.runtime-loading').length === 1",
            "The loading state did not appear",
          );
          const state = await client.evaluate(`({
            loadingCount: document.querySelectorAll('.runtime-loading').length,
            refreshCount: document.querySelectorAll('[aria-label="Refresh approved runtime catalog"]').length,
            refreshDisabled: document.querySelector('[aria-label="Refresh approved runtime catalog"]')?.disabled === true
          })`);
          requireCondition(state.loadingCount === 1, "The loading selector did not match one intended element");
          requireCondition(state.refreshCount === 1, "The refresh selector did not match one intended element");
          requireCondition(state.refreshDisabled, "Refresh remained enabled while loading");
          await client.evaluate("window.__LM_VERIFY_RESOLVE()")
          await waitFor(
            "document.querySelectorAll('.runtime-catalog-message').length === 1 && document.querySelectorAll('.runtime-loading').length === 0",
            "The loading state did not terminate",
          );
          return state;
        },
      );

      await runCheck(
        "ui.catalog-empty",
        "The packaged UI shows an explicit empty state without a loading indicator.",
        async () => {
          const state = await client.evaluate(`({
            messageCount: document.querySelectorAll('.runtime-catalog-message').length,
            loadingCount: document.querySelectorAll('.runtime-loading').length,
            text: document.querySelector('.runtime-catalog-message')?.innerText ?? ''
          })`);
          requireCondition(state.messageCount === 1, "The empty selector did not match one intended element");
          requireCondition(state.loadingCount === 0, "The loading indicator remained visible");
          requireCondition(state.text.includes("No approved runtime"), "The empty message was not explicit");
          return state;
        },
      );

      await runCheck(
        "ui.catalog-error",
        "The packaged UI shows a terminal catalog error and one retry action.",
        async () => {
          await setScenarioAndRefresh("error");
          await waitFor(
            "document.querySelectorAll('.runtime-catalog-message.error').length === 1",
            "The catalog error did not render",
          );
          const state = await client.evaluate(`(() => {
            const message = document.querySelector('.runtime-catalog-message.error');
            return {
              errorCount: document.querySelectorAll('.runtime-catalog-message.error').length,
              loadingCount: document.querySelectorAll('.runtime-loading').length,
              retryCount: message ? [...message.querySelectorAll('button')].filter((button) => button.textContent.trim() === 'Retry').length : 0,
              text: message?.innerText ?? ''
            };
          })()`);
          requireCondition(state.errorCount === 1, "The error selector did not match one intended element");
          requireCondition(state.loadingCount === 0, "The error state still showed loading");
          requireCondition(state.retryCount === 1, "The error state did not expose one retry action");
          requireCondition(state.text.includes("Verifier-injected catalog failure"), "The backend error was not visible");
          return state;
        },
      );

      await runCheck(
        "ui.catalog-rate-limit",
        "The packaged UI terminates loading and reports a rate-limit retry condition.",
        async () => {
          await setScenarioAndRefresh("rate");
          await waitFor(
            "document.querySelector('.runtime-catalog-message.error')?.innerText.includes('rate limit')",
            "The rate-limit state did not render",
          );
          const state = await client.evaluate(`({
            errorCount: document.querySelectorAll('.runtime-catalog-message.error').length,
            loadingCount: document.querySelectorAll('.runtime-loading').length,
            retryCount: document.querySelectorAll('.runtime-catalog-message.error button').length,
            text: document.querySelector('.runtime-catalog-message.error')?.innerText ?? ''
          })`);
          requireCondition(state.errorCount === 1, "The rate-limit selector did not match one intended element");
          requireCondition(state.loadingCount === 0, "The rate-limit state still showed loading");
          requireCondition(state.retryCount === 1, "The rate-limit state did not expose one retry action");
          requireCondition(/retry after 60 seconds/i.test(state.text), "The retry delay was not visible");
          return state;
        },
      );

      await runCheck(
        "ui.runtime-scope",
        "Each runtime card has one dedicated scope element despite multiple role lines.",
        async () => {
          await setScenarioAndRefresh("ready");
          await waitFor(
            "document.querySelectorAll('.runtime-option:not(.blocked)').length === 2",
            "The ready catalog did not render",
          );
          const cards = await client.evaluate(`[...document.querySelectorAll('.runtime-option:not(.blocked)')].map((card) => {
            const scope = card.querySelector('.runtime-scope');
            return {
              label: card.querySelector('strong')?.textContent ?? '',
              roleCount: card.querySelectorAll('.runtime-role').length,
              scopeCount: card.querySelectorAll('.runtime-scope').length,
              level: scope?.dataset.supportLevel ?? '',
              os: scope?.dataset.supportOs ?? '',
              text: scope?.textContent ?? '',
              architecture: scope?.dataset.supportArchitecture ?? '',
              backend: scope?.dataset.supportBackend ?? '',
              revision: scope?.dataset.runtimeRevision ?? ''
            };
          })`);
          requireCondition(cards.length === 2, "The card selector did not match two fixture cards");
          for (const card of cards) {
            requireCondition(card.roleCount >= 2, `${card.label} did not contain multiple role lines`);
            requireCondition(card.scopeCount === 1, `${card.label} did not contain one scope element`);
            requireCondition(card.level === "", `${card.label} added a frontend-owned support level`);
            requireCondition(card.os === "", `${card.label} added a frontend-owned operating-system claim`);
            requireCondition(card.text.includes("UPSTREAM EVIDENCE ONLY"), `${card.label} omitted the upstream-evidence limit`);
            requireCondition(card.text.includes("exact product configuration untested"), `${card.label} omitted the product-qualification limit`);
            requireCondition(card.architecture === "x64", `${card.label} omitted exact architecture scope`);
            requireCondition(card.revision === "b10816", `${card.label} omitted exact runtime scope`);
          }
          return { cards };
        },
      );

      await runCheck(
        "ui.blocked-backend",
        "A blocked CUDA card names server-cuda and exposes its public evidence action.",
        async () => {
          const blocked = await client.evaluate(`(() => {
            const cards = [...document.querySelectorAll('article[aria-label="cuda runtime blocked"]')];
            const card = cards[0];
            return {
              count: cards.length,
              text: card?.innerText ?? '',
              evidenceButtons: card ? [...card.querySelectorAll('button')]
                .filter((button) => button.textContent.includes('View server-cuda evidence')).length : 0
            };
          })()`);
          requireCondition(blocked.count === 1, "The blocked CUDA selector did not match one intended card");
          requireCondition(blocked.text.includes("server-cuda"), "The blocked CUDA card omitted server-cuda");
          requireCondition(blocked.evidenceButtons === 1, "The blocked CUDA card omitted one public evidence action");
          return blocked;
        },
      );

      const forgedFields = {
        browserDownloadUrl: "file:///C:/untrusted/runtime.zip",
        digest: `sha256:${"a".repeat(64)}`,
        size: 1,
        backend: "cpu",
        tag: "attacker-controlled",
      };
      for (const [field, value] of Object.entries(forgedFields)) {
        await runCheck(
          `ipc.reject-${field.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`)}`,
          `Installation IPC rejects frontend-supplied ${field} metadata.`,
          async () => {
            const result = await rejectedInvoke("install_managed_runtime", {
              request: { installKey: "cpu", adapterId: null, [field]: value },
            });
            requireCondition(result.rejected, `The backend accepted forged ${field} metadata`);
            return { rejected: true, error: result.error };
          },
        );
      }

      await runCheck(
        "ipc.reject-legacy-option",
        "Installation IPC rejects the legacy frontend-controlled artifact object.",
        async () => {
          const result = await rejectedInvoke("install_managed_runtime", {
            tag: "attacker-controlled",
            option: {
              installKey: "cpu",
              browserDownloadUrl: "file:///C:/untrusted/runtime.zip",
              digest: `sha256:${"b".repeat(64)}`,
            },
          });
          requireCondition(result.rejected, "The backend accepted the legacy artifact object");
          return { rejected: true, error: result.error };
        },
      );

      const isolationPassed = await runCheck(
        "tamper.isolated-root",
        "Destructive tamper verification uses only the configured isolated application-data root.",
        async () => {
          requireCondition(isolationRoot, "Set LOCALMOTIVE_VERIFY_ISOLATED_ROOT before launching the candidate");
          const runtimeRoot = resolve(baselineSetup.runtimeRoot);
          const prefix = `${isolationRoot.endsWith(sep) ? isolationRoot : `${isolationRoot}${sep}`}`.toLowerCase();
          requireCondition(
            runtimeRoot.toLowerCase().startsWith(prefix),
            "The packaged runtime root is outside LOCALMOTIVE_VERIFY_ISOLATED_ROOT",
          );
          return { isolated: true };
        },
      );

      const installPassed = isolationPassed && await runCheck(
        "tamper.install-approved",
        "The packaged backend installs the compiled CPU runtime into the isolated root.",
        async () => {
          installedRuntime = await invoke(
            "install_managed_runtime",
            { request: { installKey: "cpu", adapterId: null } },
            600_000,
          );
          requireCondition(installedRuntime?.runtimePath, "The install result omitted runtimePath");
          const runtimePath = resolve(installedRuntime.runtimePath);
          const prefix = `${isolationRoot.endsWith(sep) ? isolationRoot : `${isolationRoot}${sep}`}`.toLowerCase();
          requireCondition(runtimePath.toLowerCase().startsWith(prefix), "The runtime was installed outside the isolated root");
          originalRuntimeBytes = await readFile(runtimePath);
          requireCondition(originalRuntimeBytes.length > 1024, "The runtime executable is unexpectedly small");
          return {
            tag: installedRuntime.tag,
            backend: installedRuntime.backend,
            runtime_name: basename(runtimePath),
            reused: installedRuntime.reused,
            sha256: createHash("sha256").update(originalRuntimeBytes).digest("hex"),
          };
        },
      );

      let tamperWritten = false;
      if (installPassed) {
        await runCheck(
          "tamper.write",
          "The verifier changes one byte in the isolated managed runtime before the trust check.",
          async () => {
            const changed = Buffer.from(originalRuntimeBytes);
            const offset = Math.floor(changed.length / 2);
            changed[offset] ^= 0x01;
            await writeFile(installedRuntime.runtimePath, changed);
            const originalHash = createHash("sha256").update(originalRuntimeBytes).digest("hex");
            const changedHash = createHash("sha256").update(changed).digest("hex");
            requireCondition(changedHash !== originalHash, "The tamper operation did not change the digest");
            tamperWritten = true;
            return { byte_offset: offset, original_sha256: originalHash, tampered_sha256: changedHash };
          },
        );
      } else {
        addCheck(
          "tamper.write",
          "The verifier changes one byte in the isolated managed runtime before the trust check.",
          "FAIL",
          {},
          "The approved isolated runtime was not available",
        );
      }

      if (tamperWritten) {
        await runCheck(
          "tamper.health-rejection",
          "Managed health rejects the changed runtime before model download or process execution.",
          async () => {
            const health = await invoke(
              "check_managed_runtime_health",
              { request: { installKey: "cpu", adapterId: null } },
              120_000,
            );
            requireCondition(health?.passed === false, "Managed health accepted the changed runtime");
            requireCondition(health.stages?.length === 7, "Managed health did not return the seven-stage contract");
            const first = health.stages[0];
            requireCondition(first.stage === "device_enumeration", "Trust failure was attributed to the wrong first stage");
            requireCondition(first.status === "FAIL", "The first stage did not fail");
            requireCondition(first.failureReason === "trust_failure", "The failure reason was not trust_failure");
            return {
              passed: health.passed,
              stage_count: health.stages.length,
              first_stage: first.stage,
              first_status: first.status,
              first_failure_reason: first.failureReason,
            };
          },
        );

        const repairPassed = await runCheck(
          "tamper.reinstall-repair",
          "A second approved installation detects and repairs the changed runtime instead of reusing it.",
          async () => {
            const repaired = await invoke(
              "install_managed_runtime",
              { request: { installKey: "cpu", adapterId: null } },
              600_000,
            );
            requireCondition(repaired.reused === false, "The changed runtime was reported as reused");
            const repairedBytes = await readFile(repaired.runtimePath);
            const originalHash = createHash("sha256").update(originalRuntimeBytes).digest("hex");
            const repairedHash = createHash("sha256").update(repairedBytes).digest("hex");
            requireCondition(repairedHash === originalHash, "Reinstallation did not restore the approved executable bytes");
            runtimeWasRepaired = true;
            return { reused: repaired.reused, repaired_sha256: repairedHash };
          },
        );
        if (repairPassed) {
          await runCheck(
            "health.seven-stage-pass",
            "The repaired packaged runtime completes the exact seven-stage health contract.",
            async () => successfulHealthEvidence(await invoke(
              "check_managed_runtime_health",
              { request: { installKey: "cpu", adapterId: null } },
              600_000,
            )),
          );

          const cancellationPassed = await runCheck(
            "health.cancellation",
            "External cancellation stops an active packaged health run and returns its bounded contract.",
            async () => {
              const outcome = await client.evaluate(`(async () => {
                try {
                  const running = window.__TAURI_INTERNALS__.invoke(
                    "check_managed_runtime_health",
                    { request: { installKey: "cpu", adapterId: null } }
                  );
                  await new Promise((resolvePromise) => setTimeout(resolvePromise, 250));
                  const accepted = await window.__TAURI_INTERNALS__.invoke(
                    "cancel_managed_runtime_health",
                    {}
                  );
                  return { ok: true, accepted, health: await running };
                } catch (error) {
                  return { ok: false, error: String(error) };
                }
              })()`, 600_000);
              requireCondition(outcome?.ok, outcome?.error ?? "Cancelled health invocation failed");
              requireCondition(outcome.accepted === true, "The backend did not accept health cancellation");
              requireCondition(outcome.health?.passed === false, "The cancelled health run reported success");
              requireCondition(outcome.health?.stages?.length === 7, "The cancelled health run lost the seven-stage contract");
              const cancelledStages = outcome.health.stages
                .filter((stage) => stage.failureReason === "cancelled")
                .map((stage) => stage.stage);
              requireCondition(cancelledStages.length >= 1, "The cancelled run did not attribute cancellation to a stage");
              return { accepted: true, cancelled_stages: cancelledStages };
            },
          );

          if (cancellationPassed) {
            await runCheck(
              "health.restart",
              "A packaged health run succeeds after cancellation cleanup.",
              async () => successfulHealthEvidence(await invoke(
                "check_managed_runtime_health",
                { request: { installKey: "cpu", adapterId: null } },
                600_000,
              )),
            );
          } else {
            addCheck(
              "health.restart",
              "A packaged health run succeeds after cancellation cleanup.",
              "FAIL",
              {},
              "Cancellation did not complete safely",
            );
          }
        } else {
          for (const [id, criterion] of [
            ["health.seven-stage-pass", "The repaired packaged runtime completes the exact seven-stage health contract."],
            ["health.cancellation", "External cancellation stops an active packaged health run and returns its bounded contract."],
            ["health.restart", "A packaged health run succeeds after cancellation cleanup."],
          ]) {
            addCheck(id, criterion, "FAIL", {}, "Runtime repair did not complete");
          }
        }
      } else {
        for (const [id, criterion] of [
          ["tamper.health-rejection", "Managed health rejects the changed runtime before model download or process execution."],
          ["tamper.reinstall-repair", "A second approved installation detects and repairs the changed runtime instead of reusing it."],
          ["health.seven-stage-pass", "The repaired packaged runtime completes the exact seven-stage health contract."],
          ["health.cancellation", "External cancellation stops an active packaged health run and returns its bounded contract."],
          ["health.restart", "A packaged health run succeeds after cancellation cleanup."],
        ]) {
          addCheck(id, criterion, "FAIL", {}, "The isolated tamper operation did not complete");
        }
      }
    }
  }
} catch (error) {
  addCheck(
    "verifier.unhandled-error",
    "The packaged verifier completes without an unhandled error.",
    "FAIL",
    {},
    error instanceof Error ? error.message : String(error),
  );
} finally {
  if (installedRuntime?.runtimePath && originalRuntimeBytes && !runtimeWasRepaired) {
    try {
      await writeFile(installedRuntime.runtimePath, originalRuntimeBytes);
    } catch (error) {
      addCheck(
        "tamper.emergency-restore",
        "The verifier restores changed bytes after an incomplete repair.",
        "FAIL",
        {},
        error instanceof Error ? error.message : String(error),
      );
    }
  }
  if (client) client.close();
}

const result = await writeResult(artifact, hostClass);
console.log(JSON.stringify(sanitize(result), null, 2));
process.exitCode = result.overall_status === "PASS" ? 0 : 1;
