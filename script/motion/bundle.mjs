#!/usr/bin/env node
// Packs the keyframes of a demo, the snapshots the `motion` scene writes, into
// one JSON file with the markup of the icons they draw.
//
// Usage: script/motion/bundle.mjs <keyframe dir> <out.json> [--assets <dir>]
//                                 [--crop <left,top,width,height>]
//
// Keyframes are named by their file, without `.json`. The assets default to
// crates/broquest/assets. --crop keeps only that part of each keyframe, moved
// to the origin: on Linux the window draws its own frame inside a margin
// left for a shadow, and the crop is the frame.

import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const flag = (name) => {
  const ix = args.indexOf(name);
  return ix >= 0 ? args.splice(ix, 2)[1] : undefined;
};
const assets =
  flag("--assets") ?? join(dirname(fileURLToPath(import.meta.url)), "../../crates/broquest/assets");
const crop = flag("--crop")?.split(",").map(Number);
const [dir, output] = args;
if (!dir || !output || (crop && (crop.length !== 4 || crop.some(Number.isNaN)))) {
  console.error(
    "Usage: script/motion/bundle.mjs <keyframe dir> <out.json> [--assets <dir>] [--crop <left,top,width,height>]",
  );
  process.exit(2);
}

const keyframes = {};
const icons = {};
for (const file of readdirSync(dir).filter((file) => file.endsWith(".json")).sort()) {
  let snapshot = JSON.parse(readFileSync(join(dir, file), "utf8"));
  if (crop) snapshot = cropSnapshot(snapshot, crop);
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

/// Moves every item by `-left, -top` and sizes the snapshot to the crop.
/// Items wholly outside it, such as the frame's shadow, are dropped.
function cropSnapshot(snapshot, [left, top, width, height]) {
  const box = (b) => b && [b[0] - left, b[1] - top, b[2], b[3]];
  const inside = (b) => b[0] + b[2] > 0 && b[1] + b[3] > 0 && b[0] < width && b[1] < height;
  const items = [];
  for (const item of snapshot.items) {
    const moved = { ...item };
    for (const key of ["b", "eb", "clip"]) if (item[key]) moved[key] = box(item[key]);
    if (typeof item.y === "number") moved.y = item.y - top;
    if (Array.isArray(item.x)) moved.x = item.x.map((x) => x - left);
    else if (typeof item.x === "number") moved.x = item.x - left;
    if (typeof item.d === "string") moved.d = movePath(item.d, left, top);
    if (item.t === "shadow" && !inside(moved.b)) continue;
    items.push(moved);
  }
  return { ...snapshot, width, height, items };
}

/// Moves a path of absolute commands, which is all the scene writes: each
/// pair of numbers is a point.
function movePath(d, left, top) {
  if (/[a-y]/.test(d.replace(/e-?\d/g, ""))) throw new Error(`Relative path command in ${d.slice(0, 40)}`);
  let ix = 0;
  return d.replace(/-?\d+(\.\d+)?/g, (n) => {
    const value = Number(n) - (ix++ % 2 === 0 ? left : top);
    return String(Math.round(value * 100) / 100);
  });
}
