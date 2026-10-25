#!/usr/bin/env node
// Packs the keyframes of a demo, the snapshots the `motion` scene writes, into
// one JSON file with the markup of the icons they draw.
//
// Usage: script/motion/bundle.mjs <keyframe dir> <out.json> [--assets <dir>]
//
// Keyframes are named by their file, without `.json`. The assets default to
// crates/broquest/assets.

import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const assetsFlag = args.indexOf("--assets");
const assets =
  assetsFlag >= 0
    ? args.splice(assetsFlag, 2)[1]
    : join(dirname(fileURLToPath(import.meta.url)), "../../crates/broquest/assets");
const [dir, output] = args;
if (!dir || !output || !assets) {
  console.error("Usage: script/motion/bundle.mjs <keyframe dir> <out.json> [--assets <dir>]");
  process.exit(2);
}

const keyframes = {};
const icons = {};
for (const file of readdirSync(dir).filter((file) => file.endsWith(".json")).sort()) {
  const snapshot = JSON.parse(readFileSync(join(dir, file), "utf8"));
  keyframes[basename(file, ".json")] = snapshot;
  for (const path of snapshot.icons) {
    if (path in icons) continue;
    try {
      icons[path] = readFileSync(join(assets, path), "utf8");
    } catch {
      console.warn(`No icon at ${join(assets, path)}; drawing a box`);
      icons[path] = null;
    }
  }
}
writeFileSync(output, JSON.stringify({ keyframes, icons }));
console.log(`Bundled ${Object.keys(keyframes).length} keyframes into ${output}`);
