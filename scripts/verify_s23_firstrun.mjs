// Packaged first-run walkthrough (audit S-23 V1): launch the packaged binary
// with a pre-created fresh WebView2 user-data folder (no settings, no models),
// then verify each empty state gives an accurate next action.
//
// Usage: node scripts/verify_s23_firstrun.mjs <debugPort> <emptyFolder> <fixtureFolder>
import { spawnSync } from "node:child_process";
import { mkdtempSync, mkdirSync, writeFileSync, rmSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { attach } from "./lib/cdp_client.mjs";

const [portArg, emptyFolder, fixtureFolder] = process.argv.slice(2);
if (!portArg || !emptyFolder || !fixtureFolder) {
  console.error("usage: verify_s23_firstrun.mjs <debugPort> <emptyFolder> <fixtureFolder>");
  process.exit(2);
}
const port = Number(portArg);

const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok, detail });
  console.log(`${ok ? "PASS" : "FAIL"} ${name}${detail ? ` — ${detail}` : ""}`);
};

const client = await attach(port);
const text = async () => client.evaluate("document.body.innerText");
const clickNav = async (label) =>
  client.evaluate(
    `(() => {
      const button = [...document.querySelectorAll("button")].find(
        (candidate) => (candidate.textContent ?? "").trim() === ${JSON.stringify(label)},
      );
      if (!button) return false;
      button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      return true;
    })()`,
  );
const clickText = async (needle) =>
  client.evaluate(
    `(() => {
      const button = [...document.querySelectorAll("button")].find(
        (candidate) => (candidate.textContent ?? "").includes(${JSON.stringify(needle)}),
      );
      if (!button) return false;
      button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      return true;
    })()`,
  );
const settle = async (ms = 450) => new Promise((resolve) => setTimeout(resolve, ms));

/// Poll until the predicate returns true (bounded), so a fast machine and a
/// slow scan both pass without fixed sleeps deciding the result.
const waitFor = async (predicate, timeoutMs = 12_000) => {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    if (await predicate()) return true;
    if (Date.now() > deadline) return false;
    await settle(250);
  }
};

// 1) First run: Profile must not be a blank screen.
await clickNav("Profile");
await settle();
let rendered = await text();
check(
  "first-run Profile shows the empty state, not a blank screen",
  rendered.includes("No models to profile yet") || rendered.includes("No model selected"),
  rendered.includes("Open Inventory") ? "open-inventory action present" : "action missing",
);

// 2) The empty-state action leads to Inventory with the folder prompt.
await clickText("Open Inventory");
await settle();
rendered = await text();
check(
  "Inventory with no root asks for a folder and offers recovery",
  rendered.includes("Choose your GGUF model folder") &&
    rendered.includes("Choose folder") &&
    rendered.includes("Rescan this folder"),
);

// 3) An empty-but-valid folder is distinguished from a failure.
const setRoot = (value) =>
  client.evaluate(
    `(() => {
      const input = document.querySelector('input[aria-label="Model root"]');
      if (!input) return false;
      const proto = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value");
      proto.set.call(input, ${JSON.stringify(value)});
      input.dispatchEvent(new Event("input", { bubbles: true }));
      return true;
    })()`,
  );
await setRoot(emptyFolder);
await settle(200);
await clickText("Rescan");
await settle(900);
rendered = await text();
check(
  "an empty valid folder reports honestly (no models yet, not a failure)",
  rendered.includes("No GGUF models in this folder yet"),
  rendered.includes("diagnostic") ? "diagnostics counted" : "",
);

// 4) A real fixture folder fills the inventory. A successful rescan loads the
// replacement selection and opens its Profile (FE-01 behaviour), so accept
// either the populated table or the switched screen, then verify both.
await setRoot(fixtureFolder);
await settle(200);
await clickText("Rescan");
const scanned = await waitFor(async () => (await text()).includes("1 logical targets"));
check(
  "a folder with one GGUF shard loads a logical target",
  scanned,
  scanned ? "scan reported 1 target" : "scan did not report a target",
);

await clickNav("Inventory");
const rowCount = async () =>
  Number(
    await client.evaluate(
      `document.querySelectorAll("table.inventory-table tbody tr").length`,
    ),
  );
const populated = await waitFor(async () => (await rowCount()) >= 1);
const rows = await rowCount();
check(
  "the inventory table carries the fixture row",
  populated,
  `rows=${rows}`,
);
const rowText = populated
  ? await client.evaluate(
      `document.querySelector("table.inventory-table tbody tr")?.innerText ?? ""`,
    )
  : "";
check(
  "the fixture row names the file and its shard count",
  rowText.includes("fixture-alpha") && rowText.includes("1/1"),
  rowText.split("\n")[0] ?? "no row",
);

await clickNav("Profile");
// The heading is rendered uppercase by the design system; match case-insensitively.
const profileReady = await waitFor(async () =>
  (await text()).toLowerCase().includes("launch profile"),
);
rendered = await text();
check(
  "Profile becomes a real profile once a model is selected",
  profileReady,
  rendered.includes("Start") ? "start action present" : "start action missing",
);

const failed = results.filter((entry) => !entry.ok);
console.log(`S23_RESULT ${failed.length === 0 ? "PASS" : "FAIL"} (${results.length} checks)`);
process.exit(failed.length === 0 ? 0 : 1);
