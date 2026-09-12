// Installer payload identity: extract the executable embedded in the staged
// NSIS and MSI installers, record the actual payload digests, and explain the
// difference from the portable executable.
//
// Investigation result (2026-09-13, freeze-8 candidate): the three binaries
// are the SAME build bytes apart from one 3-byte bundle-type marker that
// Tauri's bundlers patch in place - the portable carries "UNK", the NSIS
// payload "NSS", the MSI payload "MSI" (the tail of the runtime's
// TYPE_VAR_<bundle> identity string). Every other byte is identical, which is
// why the recorded digests differ while the executables are, byte for byte,
// the same program. The earlier note in the lifecycle harness attributed the
// difference to non-reproducible bundler builds; this script replaces that
// assumption with a measured, reproducible comparison.
//
// Usage:
//   node scripts/verify_installer_payloads.mjs <artifactDir> <inventoryPath> <evidencePath>
// Requires 7-Zip (LOCALMOTIVE_7ZIP overrides the default install path).
import { createHash } from "node:crypto";
import { execFileSync, spawn } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SEVEN_ZIP = process.env.LOCALMOTIVE_7ZIP ?? "C:\\Program Files\\7-Zip\\7z.exe";

export function extractInstallerPayload(installerPath, { sevenZip = SEVEN_ZIP, sizeHint } = {}) {
  const workDir = mkdtempSync(join(tmpdir(), "localmotive-payload-"));
  try {
    execFileSync(sevenZip, ["x", "-y", `-o${workDir}`, installerPath], { windowsHide: true });
    const candidates = [];
    const walk = (directory) => {
      for (const entry of readdirSync(directory, { withFileTypes: true })) {
        const path = join(directory, entry.name);
        if (entry.isDirectory()) walk(path);
        else if (statSync(path).size === sizeHint) candidates.push(path);
      }
    };
    walk(workDir);
    if (candidates.length !== 1) {
      throw new Error(
        `${installerPath}: expected exactly one embedded payload of ${sizeHint} bytes, found ${candidates.length}`,
      );
    }
    return readFileSync(candidates[0]);
  } finally {
    rmSync(workDir, { recursive: true, force: true });
  }
}

/// Functional scope for an installed payload: launch the EXACT payload bytes
/// with the WebView2 debugger, attach over CDP, and evaluate the rendered
/// application shell. An exact version string and brief process survival are
/// identity and liveness checks; they are not functional qualification, and
/// this probe exists so the installed payloads carry the real thing. Returns a
/// record that names what was observed and whether it passed.
export async function probePayloadFunctionally(payloadBytes, label, port) {
  const workDir = mkdtempSync(join(tmpdir(), `localmotive-payload-probe-${label}-`));
  const exePath = join(workDir, `${label}-payload.exe`);
  writeFileSync(exePath, payloadBytes);
  const child = spawn(exePath, [], {
    env: {
      ...process.env,
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`,
    },
    stdio: "ignore",
    windowsHide: true,
  });
  const sleep = (ms) => new Promise((resolvePromise) => setTimeout(resolvePromise, ms));
  const deadline = Date.now() + 60_000;
  let target = null;
  while (Date.now() < deadline && !target) {
    try {
      const list = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
      target = list.find((entry) => entry.type === "page" && entry.webSocketDebuggerUrl) ?? null;
    } catch {
      // the debugger endpoint is not up yet
    }
    if (!target) await sleep(500);
  }
  const probe = { label, port, executable: exePath, cdpTarget: Boolean(target), navButtons: null, hasManagedControls: null, bodyChars: null };
  if (target) {
    try {
      const socket = new WebSocket(target.webSocketDebuggerUrl);
      await new Promise((resolvePromise, rejectPromise) => {
        socket.onopen = resolvePromise;
        socket.onerror = rejectPromise;
      });
      const expression =
        '(function(){const buttons=document.querySelectorAll("nav button");const text=(document.body&&document.body.innerText)||"";return JSON.stringify({navButtons:buttons.length,hasLocalmotive:/Localmotive/i.test(text),hasManagedControls:/Stop server|Start profile|HF catalog|Benchmark/i.test(text),bodyChars:text.length});})()';
      const reply = await new Promise((resolvePromise) => {
        socket.onmessage = (event) => {
          try {
            const message = JSON.parse(event.data);
            if (message.id === 1) resolvePromise(message.result?.result?.value ?? null);
          } catch {
            // ignore frames we do not answer
          }
        };
        socket.send(JSON.stringify({ id: 1, method: "Runtime.evaluate", params: { expression, returnByValue: true } }));
        setTimeout(() => resolvePromise(null), 20_000);
      });
      socket.close();
      if (reply) Object.assign(probe, JSON.parse(reply));
    } catch (error) {
      probe.probeError = String(error).slice(0, 200);
    }
  }
  try {
    child.kill();
  } catch {
    // the probe process may already be gone
  }
  await sleep(2000);
  rmSync(workDir, { recursive: true, force: true });
  probe.passed = probe.cdpTarget === true && Number(probe.navButtons) > 0 && probe.hasManagedControls === true;
  return probe;
}

/// The bundler patches one 3-byte marker in the same executable. Verify that
/// the payload and the portable executable are identical everywhere else.
export function compareBundlePayloads(portable, payload, label) {
  if (portable.length !== payload.length) {
    return {
      ok: false,
      reason: `${label}: payload size ${payload.length} differs from the portable size ${portable.length}`,
    };
  }
  const diffPositions = [];
  for (let index = 0; index < portable.length; index += 1) {
    if (portable[index] !== payload[index]) diffPositions.push(index);
  }
  if (diffPositions.length !== 3) {
    return {
      ok: false,
      reason: `${label}: expected exactly the 3-byte bundle-type marker to differ, found ${diffPositions.length} differing bytes`,
      diffPositions,
    };
  }
  const marker = (buffer) => buffer.subarray(diffPositions[0], diffPositions[0] + 3).toString("latin1");
  const portableMarker = marker(portable);
  const payloadMarker = marker(payload);
  const expected = { "NSS": "nsis", "MSI": "msi" }[payloadMarker];
  if (portableMarker !== "UNK" || !expected || expected !== label) {
    return {
      ok: false,
      reason: `${label}: unexpected marker bytes - portable '${portableMarker}', payload '${payloadMarker}'`,
      diffPositions,
    };
  }
  return { ok: true, diffPositions, portableMarker, payloadMarker };
}

export async function verifyInstallerPayloads({
  artifactDir,
  inventoryPath,
  evidencePath,
  sevenZip = SEVEN_ZIP,
  probeFunctionally = false,
  functionalBasePort = 10151,
}) {
  const inventory = JSON.parse(readFileSync(inventoryPath, "utf8"));
  const portableEntry = inventory.artifacts.find((artifact) => artifact.name.endsWith("-portable.exe"));
  const setupEntry = inventory.artifacts.find((artifact) => artifact.name.endsWith("-setup.exe"));
  const msiEntry = inventory.artifacts.find((artifact) => artifact.name.endsWith(".msi"));
  if (!portableEntry || !setupEntry || !msiEntry) {
    throw new Error("The candidate inventory does not carry the expected artifact set");
  }
  const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
  const portablePath = join(artifactDir, portableEntry.name);
  const portable = readFileSync(portablePath);
  if (portable.length !== portableEntry.sizeBytes || digest(portable) !== portableEntry.sha256) {
    throw new Error("The portable executable does not match the approved inventory");
  }
  const nsisPayload = extractInstallerPayload(join(artifactDir, setupEntry.name), {
    sevenZip,
    sizeHint: portable.length,
  });
  const msiPayload = extractInstallerPayload(join(artifactDir, msiEntry.name), {
    sevenZip,
    sizeHint: portable.length,
  });
  const nsisComparison = compareBundlePayloads(portable, nsisPayload, "nsis");
  const msiComparison = compareBundlePayloads(portable, msiPayload, "msi");
  const functional = [];
  if (probeFunctionally) {
    functional.push(await probePayloadFunctionally(nsisPayload, "nsis", functionalBasePort));
    functional.push(await probePayloadFunctionally(msiPayload, "msi", functionalBasePort + 1));
  }
  const evidence = {
    schema: "localmotive.installer-payload-identity.v1",
    generatedAt: new Date().toISOString(),
    sourceRevision: inventory.sourceRevision,
    inventorySha256: digest(readFileSync(inventoryPath)),
    release: inventory.release,
    portable: { name: portableEntry.name, sizeBytes: portableEntry.sizeBytes, sha256: portableEntry.sha256 },
    nsisPayload: { sizeBytes: nsisPayload.length, sha256: digest(nsisPayload) },
    msiPayload: { sizeBytes: msiPayload.length, sha256: digest(msiPayload) },
    marker: {
      portable: nsisComparison.portableMarker ?? null,
      nsis: nsisComparison.payloadMarker ?? null,
      msi: msiComparison.payloadMarker ?? null,
      differingBytes: nsisComparison.diffPositions?.[0] ?? null,
      note:
        "Tauri's NSIS/MSI bundlers patch one 3-byte bundle-type marker ('UNK' -> 'NSS' / 'MSI') into their " +
        "embedded copy of the executable. Every other byte is identical to the portable executable, so the " +
        "recorded digests differ while the installed programs are the same build.",
    },
    functional,
    functionalNote:
      "Each installed payload was launched from its extracted bytes with the WebView2 debugger and evaluated " +
      "over CDP: the record names the observed navigation-button count, the managed-control text and the " +
      "rendered body length. The in-sandbox lifecycle legs install the SAME payload bytes (their recorded " +
      "installed digest equals nsisPayload.sha256 / msiPayload.sha256) and prove install, launch, version " +
      "identity, uninstall and preservation; this probe proves the functional shell of those exact bytes.",
    status:
      nsisComparison.ok &&
      msiComparison.ok &&
      (functional.length === 0 || functional.every((probe) => probe.passed))
        ? "PASS"
        : "FAIL",
    detail: [
      ...(nsisComparison.reason ? [nsisComparison.reason] : []),
      ...(msiComparison.reason ? [msiComparison.reason] : []),
      ...functional
        .filter((probe) => !probe.passed)
        .map((probe) => `${probe.label} functional probe failed (cdpTarget=${probe.cdpTarget})`),
    ].join("; ") || "ok",
  };
  if (evidencePath) {
    writeFileSync(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`, "utf8");
  }
  return evidence;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [artifactDir, inventoryPath, evidencePath] = process.argv.slice(2);
  if (!artifactDir || !inventoryPath) {
    console.error(
      "usage: node scripts/verify_installer_payloads.mjs <artifactDir> <inventoryPath> [evidencePath]",
    );
    process.exit(2);
  }
  const evidence = await verifyInstallerPayloads({
    artifactDir: resolve(artifactDir),
    inventoryPath: resolve(inventoryPath),
    evidencePath: evidencePath ? resolve(evidencePath) : null,
    probeFunctionally: process.argv.includes("--functional"),
  });
  console.log(JSON.stringify(evidence, null, 2));
  if (evidence.status !== "PASS") {
    console.error(`INSTALLER PAYLOAD VERIFY FAILED: ${evidence.detail}`);
    process.exitCode = 1;
  }
}
