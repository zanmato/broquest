// Turns a scene snapshot, written by Blanco built with the `scene-export`
// feature, into SVG markup. Runs in Node and in the browser; it has no
// dependencies.
//
// Every item becomes one element carrying `data-i`, its index in the
// snapshot, and `data-t`, its kind, so an animation can pick items out. Text
// runs carry `data-text` as well.
//
// A snapshot names each icon by its asset path, such as `icons/play.svg`;
// the caller passes the markup of each, read from `crates/broquest/assets`.
//
//   import { renderSvg } from "./render.mjs";
//   const svg = renderSvg(snapshot, { icons: { "icons/play.svg": "<svg ...>" } });

const SYSTEM_FONTS = {
  ".SystemUIFont": "system-ui, -apple-system, sans-serif",
  ".ZedSans": "system-ui, sans-serif",
  ".ZedMono": "ui-monospace, monospace",
};

/// Renders `snapshot` as an SVG document.
///
/// Options:
/// - `icons`: the SVG markup of each icon, by the asset path the snapshot
///   names. An icon without markup is drawn as a faint box.
/// - `fonts`: CSS font stacks by family name, for families the page loads
///   under another name.
/// - `viewBox`: the part of the window to show, as `[x, y, width, height]`.
/// - `idPrefix`: put before every id the SVG defines. Ids are global to a
///   page, so each SVG on one page needs its own, or a `url(#...)` in one
///   finds the clip, gradient or icon of another.
export function renderSvg(snapshot, options = {}) {
  const ctx = new Context(snapshot, options);
  const body = snapshot.items.map((item, i) => ctx.item(item, i)).join("\n");
  const [x, y, width, height] = options.viewBox ?? [0, 0, snapshot.width, snapshot.height];
  return [
    `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="${x} ${y} ${width} ${height}">`,
    `<defs>${ctx.defs.join("")}</defs>`,
    body,
    trafficLights(snapshot.traffic_lights),
    `</svg>`,
  ].join("\n");
}

class Context {
  constructor(snapshot, options) {
    this.snapshot = snapshot;
    this.options = options;
    this.defs = [];
    this.clips = new Map();
    this.icons = new Map();
    this.ids = 0;
    this.prefix = options.idPrefix ?? "";
  }

  item(item, i) {
    const body = this[item.t]?.(item);
    if (!body) return "";
    const clip = this.clip(item.clip, item);
    const attrs = `data-i="${i}" data-t="${item.t}"${clip ? ` clip-path="url(#${clip})"` : ""}`;
    return `<g ${attrs}${item.t === "text" ? ` data-text="${esc(item.text)}"` : ""}>${body}</g>`;
  }

  /// The id of a clip path for `bounds`, or none when the item lies inside it.
  clip(bounds, item) {
    if (!bounds) return null;
    const [x, y, w, h] = bounds;
    const inner = itemBounds(item);
    if (
      inner &&
      inner[0] >= x &&
      inner[1] >= y &&
      inner[0] + inner[2] <= x + w &&
      inner[1] + inner[3] <= y + h
    ) {
      return null;
    }
    const key = bounds.join(",");
    let id = this.clips.get(key);
    if (!id) {
      id = `${this.prefix}c${this.clips.size}`;
      this.clips.set(key, id);
      this.defs.push(`<clipPath id="${id}"><rect x="${x}" y="${y}" width="${w}" height="${h}"/></clipPath>`);
    }
    return id;
  }

  fill(fill) {
    if (!fill) return "none";
    if (fill.type === "solid" || fill.type === "pattern") return fill.color;
    if (fill.type === "linear") {
      const id = `${this.prefix}g${this.ids++}`;
      // gpui's angle is CSS's: 0 points up, turning clockwise.
      const angle = ((fill.angle ?? 180) - 90) * (Math.PI / 180);
      const dx = Math.cos(angle) / 2;
      const dy = Math.sin(angle) / 2;
      const stops = fill.stops
        .map(([color, at]) => `<stop offset="${at}" stop-color="${color}"/>`)
        .join("");
      this.defs.push(
        `<linearGradient id="${id}" x1="${0.5 - dx}" y1="${0.5 - dy}" x2="${0.5 + dx}" y2="${0.5 + dy}">${stops}</linearGradient>`,
      );
      return `url(#${id})`;
    }
    return "none";
  }

  rect(item) {
    const [x, y, w, h] = item.b;
    const out = [];
    if (item.fill) {
      out.push(`<path d="${roundedRect(x, y, w, h, item.r)}" fill="${this.fill(item.fill)}"/>`);
    }
    if (item.border) {
      const [top, right, bottom, left] = item.border.w;
      const inner = item.r.map((r, ix) => Math.max(0, r - Math.max([top, top, bottom, bottom][ix], [left, right, right, left][ix])));
      if (item.border.style === "dashed") {
        const width = Math.max(top, right, bottom, left);
        out.push(
          `<path d="${roundedRect(x + width / 2, y + width / 2, w - width, h - width, item.r.map((r) => Math.max(0, r - width / 2)))}" fill="none" stroke="${item.border.color}" stroke-width="${width}" stroke-dasharray="${width * 3} ${width * 2}"/>`,
        );
      } else {
        const d =
          roundedRect(x, y, w, h, item.r) +
          roundedRect(x + left, y + top, w - left - right, h - top - bottom, inner);
        out.push(`<path d="${d}" fill="${item.border.color}" fill-rule="evenodd"/>`);
      }
    }
    return out.join("");
  }

  shadow(item) {
    if (item.inset) return "";
    const [x, y, w, h] = item.b;
    const id = `${this.prefix}s${this.ids++}`;
    // gpui blurs with a Gaussian whose deviation is half the blur radius.
    const deviation = item.blur / 2;
    this.defs.push(
      `<filter id="${id}" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="${deviation}"/></filter>`,
    );
    return `<path d="${roundedRect(x, y, w, h, item.r)}" fill="${item.color}" filter="url(#${id})"/>`;
  }

  path(item) {
    return `<path d="${item.d}" fill="${this.fill(item.fill)}"/>`;
  }

  underline(item) {
    const [x, y, w] = item.b;
    if (!item.wavy) {
      return `<rect x="${x}" y="${y}" width="${w}" height="${item.thickness}" fill="${item.color}"/>`;
    }
    const step = item.thickness * 2;
    let d = `M${x} ${y + step / 2}`;
    for (let at = x, up = true; at < x + w; at += step, up = !up) {
      d += `L${at + step} ${y + (up ? 0 : step)}`;
    }
    return `<path d="${d}" fill="none" stroke="${item.color}" stroke-width="${item.thickness}"/>`;
  }

  text(item) {
    const font = this.snapshot.fonts[item.font] ?? {};
    const attrs = [
      `y="${item.y}"`,
      `font-family="${esc(this.fontStack(font))}"`,
      `font-size="${item.size}"`,
      font.weight && font.weight !== 400 ? `font-weight="${font.weight}"` : "",
      font.style && font.style !== "normal" ? `font-style="${font.style}"` : "",
    ].filter(Boolean);
    const chars = [...item.text];
    const spans = item.colors
      .map(([start, length, color]) => {
        const text = chars.slice(start, start + length);
        const xs = item.x.slice(start, start + length);
        // Spaces at the end of a span draw nothing; dropping them keeps the
        // markup short.
        while (text.length && text[text.length - 1] === " ") {
          text.pop();
          xs.pop();
        }
        if (!text.length) return "";
        return `<tspan x="${xs.join(" ")}" fill="${color}">${esc(text.join(""))}</tspan>`;
      })
      .join("");
    return `<text ${attrs.join(" ")} xml:space="preserve">${spans}</text>`;
  }

  fontStack(font) {
    const families = font.families ?? [];
    const stacks = families.map(
      (family) => this.options.fonts?.[family] ?? SYSTEM_FONTS[family] ?? `"${family}"`,
    );
    return [...stacks, "sans-serif"].join(", ");
  }

  icon(item) {
    const markup = this.options.icons?.[item.path];
    const [x, y, w, h] = item.b;
    if (!markup) {
      return `<rect x="${x}" y="${y}" width="${w}" height="${h}" fill="${item.color}" opacity="0.3"/>`;
    }
    let symbol = this.icons.get(item.path);
    if (!symbol) {
      symbol = `${this.prefix}i${this.icons.size}`;
      this.icons.set(item.path, symbol);
      this.defs.push(iconSymbol(markup, symbol));
    }
    return `<use href="#${symbol}" x="${x}" y="${y}" width="${w}" height="${h}" color="${item.color}"/>`;
  }

  image(item) {
    const [x, y, w, h] = item.b;
    if (item.emoji) {
      return `<text x="${x}" y="${y + h * 0.85}" font-size="${h * 0.9}">${esc(item.emoji)}</text>`;
    }
    return `<path d="${roundedRect(x, y, w, h, item.r)}" fill="#8888" opacity="${item.opacity}"/>`;
  }
}

/// The macOS traffic lights, which the system draws over the window and so
/// are not in the snapshot's items, at the position the window asked for.
/// Each button is 14 pixels wide with a 12 pixel circle, 20 pixels apart.
function trafficLights(position) {
  if (!position) return "";
  const [x, y] = position;
  const colors = ["#ff5f57", "#febc2e", "#28c840"];
  const circles = colors
    .map(
      (color, ix) =>
        `<circle cx="${x + 7 + ix * 20}" cy="${y + 8}" r="6" fill="${color}" stroke="rgba(0,0,0,0.18)" stroke-width="0.5"/>`,
    )
    .join("");
  return `<g data-t="traffic-lights">${circles}</g>`;
}

/// An icon as a `<symbol>`. gpui paints an icon as a mask in one colour, so
/// every colour in it becomes `currentColor`.
function iconSymbol(markup, id) {
  const viewBox = /viewBox="([^"]+)"/.exec(markup)?.[1];
  const open = /<svg\b[^>]*>/.exec(markup);
  const root = open?.[0] ?? "";
  const inner = open ? markup.slice(open.index + root.length).replace(/<\/svg>\s*$/, "") : markup;
  // Paint attributes on the root apply to everything inside it.
  const inherited = ["fill", "stroke", "stroke-width", "stroke-linecap", "stroke-linejoin"]
    .map((name) => {
      const value = new RegExp(`\\s${name}="([^"]+)"`).exec(root)?.[1];
      return value ? ` ${name}="${value}"` : "";
    })
    .join("");
  // Without a fill on the root, SVG fills in black, which the mask paints in
  // the icon's colour like anything else.
  const fill = /\sfill="/.test(root) ? "" : ' fill="currentColor"';
  const tint = (text) => text.replace(/(fill|stroke)="(?!none)(?!url)[^"]*"/g, '$1="currentColor"');
  // Ids inside icons would clash between icons and with the page's own.
  const body = tint(inner).replace(/\sid="[^"]*"/g, "");
  return `<symbol id="${id}"${viewBox ? ` viewBox="${viewBox}"` : ""}><g${fill}${tint(inherited)}>${body}</g></symbol>`;
}

/// A rectangle with corner radii `[top left, top right, bottom right, bottom
/// left]`, as path data.
function roundedRect(x, y, w, h, radii = [0, 0, 0, 0]) {
  if (w <= 0 || h <= 0) return "";
  const limit = Math.min(w, h) / 2;
  const [tl, tr, br, bl] = radii.map((r) => Math.min(r, limit));
  return (
    `M${x + tl} ${y}H${x + w - tr}` +
    (tr ? `A${tr} ${tr} 0 0 1 ${x + w} ${y + tr}` : "") +
    `V${y + h - br}` +
    (br ? `A${br} ${br} 0 0 1 ${x + w - br} ${y + h}` : "") +
    `H${x + bl}` +
    (bl ? `A${bl} ${bl} 0 0 1 ${x} ${y + h - bl}` : "") +
    `V${y + tl}` +
    (tl ? `A${tl} ${tl} 0 0 1 ${x + tl} ${y}` : "") +
    "Z"
  );
}

/// Roughly where an item draws, to tell whether its clip cuts it.
function itemBounds(item) {
  if (item.b) {
    if (item.t === "shadow") {
      const [x, y, w, h] = item.b;
      const margin = item.blur * 1.5;
      return [x - margin, y - margin, w + margin * 2, h + margin * 2];
    }
    return item.b;
  }
  if (item.t === "text") {
    const left = item.x[0];
    const right = item.x[item.x.length - 1] + item.size;
    return [left, item.y - item.size, right - left, item.size * 1.3];
  }
  return null;
}

function esc(text) {
  return String(text)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
