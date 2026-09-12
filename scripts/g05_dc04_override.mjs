// DC-04.V2 packaged verification: user-override authority through the
// supported command path and against controlled bytes.
//
// Legs (all through the real Tauri commands, no override UI required):
//   1  correct checksum:  save an override whose digest matches the fixture
//      bytes; the download completes and the completion message names the
//      local-override authority.
//   2  incorrect checksum: re-save the override with a wrong digest; the same
//      download fails checksum verification and publishes nothing.
//   3  revoked: removing the override ends authorization; the download is
//      refused and no request reaches the server.
//   4  distinction: the user row carries user provenance while curated rows
//      keep theirs, and removing a curated id is refused.
//
// Usage: node scripts/g05_dc04_override.mjs <cdpPort> <fixturePort> <destinationRoot>
//
// The app must be launched with LOCALMOTIVE_VERIFY_ISOLATED_ROOT set and
// LOCALMOTIVE_HF_BASE=http://127.0.0.1:<fixturePort>, so catalog downloads
// resolve to the loopback fixture this driver serves.
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync } from "node:fs";
import { createServer } from "node:http";
import { join } from "node:path";

import { attach } from "./lib/cdp_client.mjs";

const [portArg, fixturePortArg, destinationArg] = process.argv.slice(2);
if (!portArg || !fixturePortArg || !destinationArg) {
  console.error("usage: node scripts/g05_dc04_override.mjs <cdpPort> <fixturePort> <destinationRoot>");
  process.exit(2);
}
const client = await attach(Number(portArg));
const fixturePort = Number(fixturePortArg);
const destinationRoot = destinationArg;

const REPO = "local/Handmade-GGUF";
const FILENAME = "mine-Q4_K_M.gguf";
const BODY = Buffer.from("LOCALMOTIVE-DC04-V2-FIXTURE-BYTES\n".repeat(2048));
const SHA = createHash("sha256").update(BODY).digest("hex");
const WRONG_SHA = createHash("sha256").update("definitely-not-the-fixture").digest("hex");

let fixtureHits = 0;
const server = createServer((request, response) => {
  const path = request.url ?? "";
  if (!path.startsWith(`/${REPO}/resolve/main/${FILENAME}`)) {
    response.writeHead(404, { "content-type": "text/plain" });
    response.end("not the fixture");
    return;
  }
  fixtureHits += 1;
  const total = BODY.length;
  const range = /bytes=(\d+)-(\d*)/.exec(request.headers.range ?? "");
  if (range) {
    const start = Number(range[1]);
    const end = range[2] === "" ? total - 1 : Math.min(Number(range[2]), total - 1);
    const slice = BODY.subarray(start, end + 1);
    response.writeHead(206, {
      "content-type": "application/octet-stream",
      "content-range": `bytes ${start}-${end}/${total}`,
      "content-length": String(slice.length),
      "accept-ranges": "bytes",
    });
    response.end(slice);
    return;
  }
  response.writeHead(200, {
    "content-type": "application/octet-stream",
    "content-length": String(total),
    "accept-ranges": "bytes",
  });
  response.end(BODY);
});
await new Promise((resolvePromise) => server.listen(fixturePort, "127.0.0.1", resolvePromise));
console.log(`fixture on 127.0.0.1:${fixturePort} size=${BODY.length} sha256=${SHA.slice(0, 16)}...`);

const checks = [];
const check = (name, ok, detail = "") => {
  checks.push({ name, ok });
  console.log(`${ok ? "PASS" : "FAIL"} ${name}${detail ? ` | ${detail}` : ""}`);
};
const invoke = (command, args = {}) =>
  client.evaluate(`(async () => {
    try {
      const value = await window.__TAURI_INTERNALS__.invoke(${JSON.stringify(command)}, ${JSON.stringify(args)});
      return { ok: true, value };
    } catch (error) {
      return { ok: false, error: String(error && error.message ? error.message : error) };
    }
  })()`);

const overrideModel = (sha256) => ({
  id: "mine",
  repo: REPO,
  family: "Handmade",
  parameters: "1B",
  publisher: "local",
  author: "local",
  files: [
    {
      quant: "Q4_K_M",
      filename: FILENAME,
      sizeBytes: BODY.length,
      sha256,
      revision: "main",
      userSourced: true,
    },
  ],
  userSourced: true,
});

const destination = (name) => {
  const dir = join(destinationRoot, name);
  mkdirSync(dir, { recursive: true });
  return dir;
};

// Seed the curated mirror so the provenance distinction is observable.
const fetched = await invoke("fetch_model_catalog");
console.log(`fetch_model_catalog ok=${fetched.ok}${fetched.ok ? "" : ` error=${fetched.error}`}`);

// ---------- Leg 1: correct checksum authorizes the transfer ----------
const saved = await invoke("save_user_catalog_override", { model: overrideModel(SHA) });
check("dc04.override-saved-with-correct-digest", saved.ok, saved.ok ? "" : saved.error);
const rowsAfterSave = Array.isArray(saved.value) ? saved.value : [];
const userRow = rowsAfterSave.find((entry) => entry.id === "mine");
check("dc04.user-row-keeps-user-provenance", userRow?.userSourced === true);
check(
  "dc04.curated-rows-keep-curated-provenance",
  rowsAfterSave.some((entry) => entry.userSourced !== true),
  `rows=${rowsAfterSave.length}`,
);

const hitsBeforeLeg1 = fixtureHits;
const downloadOk = await invoke("download_catalog_file", {
  repo: REPO,
  filename: FILENAME,
  revision: "main",
  destination: destination("ok"),
  connections: 1,
});
check(
  "dc04.correct-checksum-download-completes",
  // On success the command returns the published path; the authority-labelled
  // completion message travels on the download:progress event (state "done")
  // and is pinned by the DC-04 release gate.
  downloadOk.ok && String(downloadOk.value).includes(FILENAME),
  downloadOk.ok ? String(downloadOk.value).slice(-70) : downloadOk.error,
);
const published = join(destination("ok"), FILENAME);
const publishedBytes = existsSync(published) ? readFileSync(published) : null;
check(
  "dc04.published-bytes-match-the-fixture",
  publishedBytes !== null && publishedBytes.equals(BODY),
  `hits=${fixtureHits - hitsBeforeLeg1}`,
);

// ---------- Leg 2: incorrect checksum refuses and publishes nothing ----------
const resaved = await invoke("save_user_catalog_override", { model: overrideModel(WRONG_SHA) });
check("dc04.override-resaved-with-wrong-digest", resaved.ok, resaved.ok ? "" : resaved.error);
const badDest = destination("bad");
const badDownload = await invoke("download_catalog_file", {
  repo: REPO,
  filename: FILENAME,
  revision: "main",
  destination: badDest,
  connections: 1,
});
check(
  "dc04.incorrect-checksum-download-refused",
  badDownload.ok === false && String(badDownload.error).includes("failed its SHA-256 checksum"),
  badDownload.ok ? `unexpected success: ${badDownload.value}` : String(badDownload.error).slice(0, 110),
);
check("dc04.no-file-published-after-mismatch", !existsSync(join(badDest, FILENAME)));

// ---------- Leg 3: removal revokes authorization before any transfer ----------
const removed = await invoke("remove_user_catalog_override", { id: "mine" });
check("dc04.override-removed", removed.ok, removed.ok ? "" : removed.error);
const hitsBeforeLeg3 = fixtureHits;
const revoked = await invoke("download_catalog_file", {
  repo: REPO,
  filename: FILENAME,
  revision: "main",
  destination: destination("revoked"),
  connections: 1,
});
check(
  "dc04.revoked-download-refused-with-identity",
  revoked.ok === false &&
    String(revoked.error).includes("not in the validated catalog or in your local overrides"),
  revoked.ok ? `unexpected success: ${revoked.value}` : String(revoked.error).slice(0, 110),
);
check("dc04.revoked-refusal-touches-no-network", fixtureHits === hitsBeforeLeg3, `hitsDelta=${fixtureHits - hitsBeforeLeg3}`);

// ---------- Leg 4: curated versus user provenance stays distinct ----------
const rowsAfterRemoval = Array.isArray(removed.value) ? removed.value : [];
check("dc04.user-row-gone-after-removal", !rowsAfterRemoval.some((entry) => entry.id === "mine"));
const curated = rowsAfterRemoval.find((entry) => entry.userSourced !== true && typeof entry.id === "string");
if (curated) {
  const attempted = await invoke("remove_user_catalog_override", { id: curated.id });
  check(
    "dc04.curated-row-cannot-be-removed-here",
    attempted.ok === false && String(attempted.error).includes("ships with the curated catalog"),
    attempted.ok ? "curated row was removed" : String(attempted.error).slice(0, 110),
  );
} else {
  check("dc04.curated-row-cannot-be-removed-here", false, "no curated row available to probe");
}

const failed = checks.filter((entry) => !entry.ok);
console.log(`DC04 SUMMARY: ${checks.length - failed.length}/${checks.length} checks PASS`);
server.close();
process.exit(failed.length === 0 ? 0 : 1);
