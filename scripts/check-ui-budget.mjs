// UI bundle budget (backlog: UI speed, measure first). Runs at the end of
// `npm run build`, after dist exists. Caps come from the 2026-09-26
// baseline (JS 350,816 B raw / 105,510 B gzip; CSS 43,222 B / 8,882 B)
// with headroom for the bounded-input work already shipped. Tighten via
// the BUDGET_JS_GZIP_KB / BUDGET_JS_RAW_KB overrides; a failure means the
// bundle grew past the measured budget, not that the app is slow.
import { strict as assert } from "node:assert";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { gzipSync } from "node:zlib";

const dir = new URL("../dist/assets/", import.meta.url);
const js = readdirSync(dir).filter((f) => f.endsWith(".js"));
assert.equal(js.length, 1, `expected one bundle, found ${js.length}`);
const raw = statSync(new URL(`../dist/assets/${js[0]}`, import.meta.url)).size;
const gzip = gzipSync(readFileSync(new URL(`../dist/assets/${js[0]}`, import.meta.url))).length;

const gzipCap = Number(process.env.BUDGET_JS_GZIP_KB ?? 130) * 1024;
const rawCap = Number(process.env.BUDGET_JS_RAW_KB ?? 420) * 1024;
console.log(`bundle: ${js[0]} raw=${raw} B gzip=${gzip} B (caps raw=${rawCap} gzip=${gzipCap})`);
assert.ok(raw <= rawCap, `JS raw ${raw} B exceeds cap ${rawCap} B`);
assert.ok(gzip <= gzipCap, `JS gzip ${gzip} B exceeds cap ${gzipCap} B`);
console.log("UI budget: PASS");
