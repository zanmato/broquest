#!/usr/bin/env node
// Writes a scene snapshot as an SVG file, with its icons read from the app's
// assets.
//
// Usage: script/motion/to-svg.mjs <snapshot.json> <out.svg> [--assets <dir>]
//
// The assets default to crates/broquest/assets.

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { renderSvg } from "./render.mjs";

const args = process.argv.slice(2);
const assetsFlag = args.indexOf("--assets");
const assets =
  assetsFlag >= 0
    ? args.splice(assetsFlag, 2)[1]
    : join(dirname(fileURLToPath(import.meta.url)), "../../crates/broquest/assets");
const [input, output] = args;
if (!input || !output || !assets) {
  console.error("Usage: script/motion/to-svg.mjs <snapshot.json> <out.svg> [--assets <dir>]");
  process.exit(2);
}

const snapshot = JSON.parse(readFileSync(input, "utf8"));
const icons = {};
for (const path of snapshot.icons) {
  try {
    icons[path] = readFileSync(join(assets, path), "utf8");
  } catch {
    console.warn(`No icon at ${join(assets, path)}; drawing a box`);
  }
}
writeFileSync(output, renderSvg(snapshot, { icons }));
