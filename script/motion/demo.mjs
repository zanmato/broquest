// Plays the motion demo of the website: query parameters typed into the URL
// bar, which fill the query table; the request sent with its JSON response;
// the response filtered with JSONPath; and the token request's scripts run.
// Every state shown is a keyframe Broquest exported (the `motion` scene in
// crates/broquest/src/screenshots.rs). What moves between them is drawn
// here: the camera, the typing, the caret, the pointer and the keys. It plays
// once and stops on the whole window, with hotspots over features that tell
// more about them when pointed at.
//
//   import { mountDemo } from "./demo.mjs";
//   const bundle = await (await fetch("demo.json")).json();
//   const demo = mountDemo(element, bundle);
//
// A frame is a function of time alone, so `demo.seek(t)` shows exactly what
// playback shows at `t` seconds, as a video capture needs.

import { renderSvg } from "./render.mjs";

const WIDTH = 1640;
const HEIGHT = 984;

/// What the scene types into the URL bar, a keyframe each, as `02-query-1`
/// to `04-query-3` hold it.
const QUERY = ["?status=in_transit", "&destination_country=SE", "&limit=4"];
/// What it types into the response filter, as `06-filter-1` and
/// `07-filter-2` hold it.
const FILTER = ["$.data", "[?(@.service == 'express')]"];

const CARET = "#f5e0dc";

/// Keyframes captured after the demo, only for the hotspots' cards. This
/// demo's cards crop keyframes of the demo itself.
const MEDIA_KEYS = new Set();

/// The hotspots shown once the demo stops: what each says and how to find
/// where it goes in the last keyframe.
const HOTSPOTS = [
  {
    title: "Environments",
    text: "A collection can have several environments, such as Local, Staging and Production. Reference a variable as {{name}}. Secret values are masked in the editor and saved to the system keychain.",
    find: (items) => anchorOn(textRun(items, (r) => r.text === "Local" && r.y < 130)),
  },
  {
    title: "Auth",
    text: "Basic, Digest, API key, JWT and OAuth2 client credentials, set on a request or on the collection.",
    find: (items) => anchorOn(textRun(items, (r) => r.text === "Auth" && r.x[0] > 300 && r.y < 200)),
  },
  {
    title: "Scripting",
    text: "Each request has a pre-request and a post-response script, run in QuickJS with LLRT modules for crypto, Buffer and URL. bro.setEnvVar saves a value, such as a token, to the selected environment.",
    find: (items) => anchorAfter(textRun(items, (r) => r.text === "Pre-request Script")),
  },
  {
    title: "Recipes",
    text: "The script editor checks syntax as you type, completes the API and has recipes for common scripts such as HMAC signatures.",
    find: (items) => anchorOn(textRun(items, (r) => r.text === "Recipes")),
  },
  {
    title: "Collections",
    text: "Each request is a TOML file and each group is a folder, so a collection can live in the repository of the API it tests. Bruno's OpenCollection folders open too, with partial support.",
    find: (items) => anchorOn(textRun(items, (r) => r.text === "Kestrel Shipping" && r.x[0] < 300)),
  },
  {
    title: "Palette",
    text: "Press ⌘ K to open a request by name or run a command.",
    find: (items) => anchorAfter(textRun(items, (r) => r.text.startsWith("Search commands"))),
  },
  {
    title: "Response filter",
    text: "Filter a JSON response with a JSONPath expression in the field under it.",
    find: (items) => anchorAfter(textRun(items, (r) => r.text.startsWith("Filter the response"))),
    // The filtered response, from the end of the filter's typing.
    media: (keyframes) => {
      const run = textRun(keyframes["07-filter-2"].items, (r) => r.text.startsWith("$.data[?"));
      return { key: "07-filter-2", box: [314, run.y - 250, 560, 268] };
    },
  },
  {
    title: "Import",
    text: "Paste a cURL command into the URL bar to import it, or copy a request as cURL. A collection can also be made from an OpenAPI spec or a WSDL URL.",
    find: (items) => anchorOnIcon(items, (i) => i.path.includes("copy") && i.b[1] < 130),
  },
  {
    title: "History",
    text: "Every response is listed in the history with its method, URL, status and time.",
    find: (items) => anchorOnIcon(items, (i) => i.path.includes("history")),
  },
];

export function mountDemo(root, bundle, options = {}) {
  const { keyframes, icons } = bundle;
  const reduced =
    options.reducedMotion ??
    globalThis.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

  injectStyles();
  root.classList.add("bd-root");
  root.innerHTML = `
    <div class="bd-viewport">
      <div class="bd-stage"><div class="bd-camera"></div></div>
      <div class="bd-overlay">
        <div class="bd-caret"></div>
        <div class="bd-ring"></div>
        <div class="bd-ripple"></div>
        <svg class="bd-pointer" viewBox="0 0 16 22" aria-hidden="true">
          <path d="M1 1v17l4.5-4.2 3 6.7 3-1.4-3-6.6H15z" fill="#fff" stroke="#11111b" stroke-width="1.4" stroke-linejoin="round"/>
        </svg>
        <div class="bd-keys"></div>
        <div class="bd-caption"></div>
        <div class="bd-dim"></div>
        <button class="bd-replay" type="button">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
          Replay
        </button>
      </div>
    </div>
    <div class="bd-spots"></div>`;
  const viewport = root.querySelector(".bd-viewport");
  const stage = root.querySelector(".bd-stage");
  const camera = root.querySelector(".bd-camera");
  const caret = root.querySelector(".bd-caret");
  const ring = root.querySelector(".bd-ring");
  const ripple = root.querySelector(".bd-ripple");
  const pointer = root.querySelector(".bd-pointer");
  const keys = root.querySelector(".bd-keys");
  const caption = root.querySelector(".bd-caption");
  const dim = root.querySelector(".bd-dim");
  const spots = root.querySelector(".bd-spots");
  const replay = root.querySelector(".bd-replay");

  // Covers what is still to be typed where the keyframe already holds it, in
  // the colour under it.
  const blank = document.createElement("div");
  blank.className = "bd-blank";
  camera.append(blank);

  const layers = {};
  const page = Math.random().toString(36).slice(2, 6);
  const timeline = Object.entries(keyframes).filter(([name]) => !MEDIA_KEYS.has(name));
  for (const [ix, [name, snapshot]] of timeline.entries()) {
    const layer = document.createElement("div");
    layer.className = "bd-layer";
    // Every layer is on the same page, so each needs ids of its own.
    const idPrefix = `bd${page}k${ix}-`;
    layer.innerHTML = renderSvg(snapshot, { icons, fonts: options.fonts, idPrefix });
    camera.append(layer);
    layers[name] = layer;
  }

  const plan = makePlan(keyframes);
  // The lines of each response that comes in, by line, so they come in in
  // turn.
  for (const stagger of plan.staggers) {
    stagger.lines = lineElements(layers[stagger.key], keyframes[stagger.key], stagger.region);
  }
  buildHotspots(spots, dim, keyframes, plan.lastKey, {
    docs: options.docs ?? null,
    scale: () => scale,
    // A card's crop of a keyframe is an SVG of its own, with ids of its own.
    media: (key, box, ix) =>
      renderSvg(keyframes[key], {
        icons,
        fonts: options.fonts,
        viewBox: box,
        idPrefix: `bd${page}m${ix}-`,
      }),
  });

  let scale = 1;
  const resize = () => {
    scale = viewport.clientWidth / WIDTH;
    stage.style.transform = `scale(${scale})`;
  };
  new ResizeObserver(resize).observe(viewport);
  resize();

  let shownCaption = null;
  let shownKeys = null;

  function frame(t) {
    if (reduced) t = plan.duration;
    t = Math.min(t, plan.duration);
    // Stopped on the whole window, the hotspots take over.
    const ended = t >= plan.duration;
    root.classList.toggle("bd-ended", ended);

    // The camera, as the point of the stage at the centre of the view and a
    // zoom.
    const view = interpolate(plan.camera, t);
    const zoom = view.z;
    const tx = clamp(WIDTH / 2 - view.x * zoom, WIDTH - WIDTH * zoom, 0);
    const ty = clamp(HEIGHT / 2 - view.y * zoom, HEIGHT - HEIGHT * zoom, 0);
    camera.style.transform = `translate(${tx}px, ${ty}px) scale(${zoom})`;
    const toScreen = (x, y) => [(x * zoom + tx) * scale, (y * zoom + ty) * scale];

    // Keyframes: the one shown, and the one under it while it fades in or
    // is revealed character by character.
    const { base, top, opacity, reveal, cover } = layerState(plan, t);
    if (cover && cover.right > cover.x) {
      blank.style.display = "";
      blank.style.background = cover.color;
      blank.style.transform = `translate(${cover.x}px, ${cover.top}px)`;
      blank.style.width = `${cover.right - cover.x}px`;
      blank.style.height = `${cover.bottom - cover.top}px`;
    } else {
      blank.style.display = "none";
    }
    for (const [name, layer] of Object.entries(layers)) {
      const visible = name === base || name === top;
      layer.style.display = visible ? "" : "none";
      layer.style.zIndex = name === top ? 2 : 1;
      layer.style.opacity = name === top ? opacity : 1;
      layer.style.clipPath = name === top && reveal ? patch(reveal) : "";
    }

    // The lines of a response come in one after another, while the response
    // that just arrived shows.
    for (const stagger of plan.staggers) {
      const local = t - stagger.at;
      const shown = base === stagger.key || top === stagger.key;
      if (shown && local >= 0 && local < 2.5) {
        stagger.lines.forEach((elements, line) => {
          const p = easeOut(clamp((local - 0.05 - line * 0.03) / 0.3, 0, 1));
          for (const element of elements) {
            element.style.opacity = p;
            element.style.transform = `translateY(${(1 - p) * 6}px)`;
          }
        });
        stagger.dirty = true;
      } else if (stagger.dirty) {
        stagger.lines.flat().forEach((element) => {
          element.style.opacity = "";
          element.style.transform = "";
        });
        stagger.dirty = false;
      }
    }

    // The caret, solid while typing and blinking while it waits.
    const caretAt = caretState(plan, t);
    if (caretAt) {
      const [x, y] = toScreen(caretAt.x, caretAt.top);
      caret.style.display = "";
      caret.style.transform = `translate(${x}px, ${y}px)`;
      caret.style.height = `${caretAt.height * zoom * scale}px`;
      caret.style.width = `${Math.max(1.5, 1.6 * zoom * scale)}px`;
      caret.style.opacity = caretAt.blink ? (Math.floor(t * 1.8) % 2 ? 0 : 1) : 1;
    } else {
      caret.style.display = "none";
    }

    // The pointer, and the ripples of its clicks.
    const hand = interpolate(plan.pointer, t);
    if (hand.o > 0.01) {
      const [x, y] = toScreen(hand.x, hand.y);
      pointer.style.display = "";
      pointer.style.opacity = hand.o;
      pointer.style.transform = `translate(${x - 2}px, ${y - 1}px)`;
    } else {
      pointer.style.display = "none";
    }
    const click = plan.clicks.findLast((c) => t >= c.t && t < c.t + 0.45);
    if (click) {
      const p = (t - click.t) / 0.45;
      const [x, y] = toScreen(click.x, click.y);
      const size = 10 + 34 * easeOut(p);
      ripple.style.display = "";
      ripple.style.opacity = 0.55 * (1 - p);
      ripple.style.width = ripple.style.height = `${size}px`;
      ripple.style.transform = `translate(${x - size / 2}px, ${y - size / 2}px)`;
    } else {
      ripple.style.display = "none";
    }

    // A ring drawn around what the scene points at.
    const highlight = plan.rings.find((r) => t >= r.from && t < r.to);
    if (highlight) {
      // Kept inside the view, which a button at the window's edge would
      // otherwise push the ring out of.
      const [x0, y0] = toScreen(highlight.b[0] - 4, highlight.b[1] - 4).map((v) => Math.max(v, 3));
      const [x1, y1] = toScreen(highlight.b[0] + highlight.b[2] + 4, highlight.b[1] + highlight.b[3] + 4).map(
        (v, ix) => Math.min(v, (ix ? HEIGHT : WIDTH) * scale - 3),
      );
      const p = clamp(Math.min(t - highlight.from, highlight.to - t) / 0.3, 0, 1);
      ring.style.display = "";
      ring.style.opacity = p;
      ring.style.transform = `translate(${x0}px, ${y0}px)`;
      ring.style.width = `${x1 - x0}px`;
      ring.style.height = `${y1 - y0}px`;
    } else {
      ring.style.display = "none";
    }

    // Keys pressed, shown as keycaps.
    const press = plan.keys.find((k) => t >= k.from && t < k.to);
    if (press !== shownKeys) {
      keys.innerHTML = press ? press.keys.map((key) => `<kbd>${key}</kbd>`).join("") : "";
      shownKeys = press;
    }
    if (press) {
      const p = clamp(Math.min(t - press.from, press.to - t) / 0.18, 0, 1);
      keys.style.opacity = p;
      keys.classList.toggle("bd-pressed", t >= press.down && t < press.down + 0.18);
      keys.style.transform = `translate(-50%, ${(1 - p) * 10}px)`;
    }

    const words = ended
      ? { text: "Point at a dot to see more", from: -Infinity, to: Infinity }
      : plan.captions.find((c) => t >= c.from && t < c.to);
    if (words !== shownCaption) {
      caption.textContent = words ? words.text : "";
      shownCaption = words;
    }
    caption.style.opacity = ended
      ? ""
      : words
        ? clamp(Math.min(t - words.from, words.to - t) / 0.35, 0, 1)
        : 0;
  }

  let start = null;
  let offset = 0;
  let playing = false;
  let raf = 0;
  const tick = (now) => {
    if (start === null) start = now - offset * 1000;
    const t = (now - start) / 1000;
    offset = Math.min(t, plan.duration);
    frame(offset);
    if (t >= plan.duration) {
      playing = false;
      return;
    }
    raf = requestAnimationFrame(tick);
  };
  const play = () => {
    if (playing || reduced) return;
    if (offset >= plan.duration) offset = 0;
    playing = true;
    start = null;
    raf = requestAnimationFrame(tick);
  };
  replay.addEventListener("click", () => {
    pause();
    offset = 0;
    play();
  });
  const pause = () => {
    playing = false;
    cancelAnimationFrame(raf);
  };

  frame(0);
  if (options.autoplay !== false) {
    // Only play while on screen.
    // Only play while on screen, and only once: scrolled back to, it stays
    // stopped on the hotspots.
    new IntersectionObserver(
      ([entry]) => {
        if (!entry.isIntersecting) pause();
        else if (offset < plan.duration) play();
      },
      { threshold: 0.2 },
    ).observe(viewport);
  }

  return {
    duration: plan.duration,
    get ended() {
      return offset >= plan.duration;
    },
    play,
    pause,
    seek(t) {
      pause();
      offset = t;
      frame(t);
    },
  };
}

/// Adds the hotspots, placed by where their features are in `items`, the last
/// keyframe's. Pointing at one, or focusing or tapping it, shows its card and
/// dims the rest of the window around it.
function buildHotspots(container, dim, keyframes, lastKey, { docs, scale, media }) {
  const items = keyframes[lastKey].items;
  // Without docs to point at, the cards go without their links.
  const base = docs ? (docs.endsWith("/") ? docs : `${docs}/`) : null;
  let index = 0;
  for (const spot of HOTSPOTS) {
    let at;
    try {
      at = spot.find(items);
    } catch {
      continue;
    }
    let figure = "";
    if (spot.media) {
      try {
        const { key, box } = spot.media(keyframes);
        figure = `<div class="bd-media" style="aspect-ratio: ${box[2]} / ${box[3]}">${media(key, box, index)}</div>`;
      } catch {
        // Without the keyframe the card goes without its picture.
      }
    }
    const id = `bd-card-${Math.random().toString(36).slice(2, 8)}`;
    const element = document.createElement("div");
    element.className = "bd-spot";
    element.style.left = `${(at.x / WIDTH) * 100}%`;
    element.style.top = `${(at.y / HEIGHT) * 100}%`;
    element.style.setProperty("--i", index++);
    element.dataset.side = at.x > WIDTH * 0.55 ? "left" : "right";
    element.dataset.rise = at.y > HEIGHT * 0.55 ? "up" : "down";
    element.innerHTML = `
      <button class="bd-dot" type="button" aria-label="${spot.title}" aria-describedby="${id}"></button>
      <div class="bd-card${figure ? " bd-wide" : ""}" role="tooltip" id="${id}">
        ${figure}
        <strong>${spot.title}</strong>
        <p>${spot.text}</p>
        ${spot.page && base ? `<a href="${base}${spot.page}">Read more</a>` : ""}
      </div>`;
    const show = () => {
      for (const other of container.querySelectorAll(".bd-open")) {
        if (other !== element) other.classList.remove("bd-open");
      }
      element.classList.add("bd-open");
      const radius = 70 * scale();
      dim.style.background = `radial-gradient(circle at ${(at.x / WIDTH) * 100}% ${(at.y / HEIGHT) * 100}%, transparent ${radius}px, rgb(17 17 27 / 0.62) ${radius + 70 * scale()}px)`;
      dim.classList.add("bd-on");
    };
    const hide = () => {
      element.classList.remove("bd-open");
      if (!container.querySelector(".bd-open")) dim.classList.remove("bd-on");
    };
    element.addEventListener("pointerenter", show);
    element.addEventListener("pointerleave", hide);
    element.addEventListener("focusin", show);
    element.addEventListener("focusout", (event) => {
      if (!element.contains(event.relatedTarget)) hide();
    });
    // A tap opens a card, a second tap closes it.
    element.querySelector(".bd-dot").addEventListener("click", () =>
      element.classList.contains("bd-open") ? hide() : show(),
    );
    container.append(element);
  }
}

function textRun(items, test) {
  const run = items.find((item) => item.t === "text" && test(item));
  if (!run) throw new Error("no such text");
  return run;
}

/// The box a text run fills: its clip when that is a button or badge around
/// it, otherwise the text itself.
function runBox(run) {
  const right = run.x[run.x.length - 1] + run.size * 0.6;
  if (run.clip && run.clip[2] < 220) return run.clip;
  return [run.x[0], run.y - run.size, right - run.x[0], run.size * 1.3];
}

/// A hotspot just off the top right corner of what a run labels, as a badge
/// sits, clear of the label itself.
function anchorOn(run) {
  const [x, y, w] = runBox(run);
  return { x: x + w + 3, y: y - 1 };
}

function union(boxes) {
  const left = Math.min(...boxes.map((b) => b[0]));
  const top = Math.min(...boxes.map((b) => b[1]));
  const right = Math.max(...boxes.map((b) => b[0] + b[2]));
  const bottom = Math.max(...boxes.map((b) => b[1] + b[3]));
  return [left, top, right - left, bottom - top];
}

function pad([x, y, w, h], by) {
  return [x - by, y - by, w + by * 2, h + by * 2];
}

/// A hotspot just off the top right corner of an icon.
function anchorOnIcon(items, test) {
  const icon = items.find((item) => item.t === "icon" && test(item));
  if (!icon) throw new Error("no such icon");
  return { x: icon.b[0] + icon.b[2] + 3, y: icon.b[1] - 1 };
}

/// A hotspot just after the end of a run's text, on its line.
function anchorAfter(run) {
  return { x: run.x[run.x.length - 1] + run.size * 1.8, y: run.y - run.size * 0.35 };
}

/// Where `text` is typed into `run`, character by character from `at`: the
/// time and position of each character, and the band of the line.
function typing(run, text, at, seed) {
  const start = [...run.text.slice(0, run.text.indexOf(text))].length;
  let random = seed;
  const chars = [...text].map((ch, ix) => {
    random = (random * 16807) % 2147483647;
    at += ch === " " ? 0.03 : 0.036 + (random / 2147483647) * 0.03;
    return { t: at, x: run.x[start + ix] };
  });
  const step = run.x[start + 1] - run.x[start];
  return {
    start: chars[0].t - 0.04,
    end: at + 0.05,
    chars,
    from: run.x[start] - 1,
    right: chars[chars.length - 1].x + step,
    step,
    top: run.y - run.size * 1.05,
    bottom: run.y + run.size * 0.4,
  };
}

/// The response under the request, as a text run of its first line shows it.
function responseRegion(snapshot) {
  const first = textRun(snapshot.items, (r) => /^ *1 /.test(r.text) && r.clip[1] > 300);
  return { clip: first.clip, first: first.y, size: first.size };
}

/// Works out the timeline from where things are in the keyframes.
function makePlan(keyframes) {
  // The URL, a part of the query at a time.
  const typed = [];
  let at = 1.9;
  let url = "";
  for (const [ix, part] of QUERY.entries()) {
    const key = `0${ix + 2}-query-${ix + 1}`;
    url += part;
    const run = textRun(keyframes[key].items, (r) => r.text.includes(url) && r.y < 140);
    const below = ix === 0 ? "01-start" : `0${ix + 1}-query-${ix}`;
    const segment = { key, below, ...typing(run, part, at, 7 + ix) };
    typed.push(segment);
    at = segment.end + 0.35;
  }
  const urlTyped = typed[typed.length - 1].end;

  const send = urlTyped + 0.95;
  const filterBox = textRun(keyframes["05-response"].items, (r) =>
    r.text.startsWith("Filter the response"),
  );
  const filterPoint = [filterBox.x[0] + 160, filterBox.y - 4];
  const filterClick = send + 2.2;

  // The filter, its array first and then the condition.
  at = filterClick + 0.45;
  let filter = "";
  for (const [ix, part] of FILTER.entries()) {
    const key = `0${ix + 6}-filter-${ix + 1}`;
    filter += part;
    const run = textRun(keyframes[key].items, (r) => r.text === filter);
    const below = ix === 0 ? "05-response" : `0${ix + 5}-filter-${ix}`;
    const segment = { key, below, ...typing(run, part, at, 31 + ix) };
    if (ix === 0) {
      // The empty field shows its placeholder, which typing clears: the
      // whole field comes from the keyframe typed into, with the part not
      // yet typed covered.
      segment.field = [run.clip[0] - 4, run.clip[0] + run.clip[2]];
      segment.color = backgroundAt(keyframes[key].items, run.x[0] + 1, run.y - run.size * 0.3);
    }
    typed.push(segment);
    at = segment.end + 0.9;
  }
  const filtered = typed[typed.length - 1].end;

  // Over to the token request and its scripts, then sent.
  const tab = textRun(keyframes["07-filter-2"].items, (r) => r.text === "Log in" && r.y < 80);
  const tabPoint = [tab.x[0] + 18, tab.y - 4];
  const scripts = filtered + 2.2;
  const sendScripts = scripts + 2.6;
  const outro = sendScripts + 3.0;
  const duration = outro + 1.4;

  const urlRun = typed[0];
  const post = textRun(keyframes["09-scripts-sent"].items, (r) => r.text === "Post-response Script");
  const response = responseRegion(keyframes["05-response"]);

  return {
    duration,
    lastKey: "09-scripts-sent",
    typed,
    send,
    filterClick,
    scripts,
    sendScripts,
    staggers: [
      { key: "05-response", at: send, region: responseRegion(keyframes["05-response"]) },
      { key: "06-filter-1", at: typed[3].end, region: responseRegion(keyframes["06-filter-1"]) },
      { key: "07-filter-2", at: typed[4].end, region: responseRegion(keyframes["07-filter-2"]) },
      {
        key: "09-scripts-sent",
        at: sendScripts,
        region: responseRegion(keyframes["09-scripts-sent"]),
      },
    ],
    camera: [
      { t: 0, x: WIDTH / 2, y: HEIGHT / 2, z: 1 },
      { t: 0.7, x: WIDTH / 2, y: HEIGHT / 2, z: 1 },
      // Onto the URL bar and the query table under it.
      { t: 1.8, x: 860, y: 210, z: 1.55 },
      { t: urlTyped + 0.3, x: 880, y: 220, z: 1.55 },
      // Out as the response comes in.
      { t: send + 0.2, x: 980, y: 520, z: 1.25 },
      { t: filterClick - 0.6, x: 980, y: 560, z: 1.25 },
      // Down to the filter and the response above it.
      { t: filterClick + 0.3, x: filterPoint[0] + 180, y: response.first + 280, z: 1.55 },
      { t: filtered + 1.2, x: filterPoint[0] + 180, y: response.first + 280, z: 1.55 },
      // Over to the scripts.
      { t: scripts + 0.6, x: 940, y: post.y - 40, z: 1.3 },
      { t: sendScripts + 0.4, x: 940, y: post.y - 20, z: 1.3 },
      { t: sendScripts + 1.2, x: 940, y: post.y + 80, z: 1.3 },
      { t: outro, x: 940, y: post.y + 80, z: 1.3 },
      { t: outro + 1.3, x: WIDTH / 2, y: HEIGHT / 2, z: 1 },
      { t: duration, x: WIDTH / 2, y: HEIGHT / 2, z: 1 },
    ],
    pointer: [
      { t: 0, x: 1250, y: 700, o: 0 },
      { t: send + 0.6, x: 1250, y: 700, o: 0 },
      { t: send + 1.0, x: 1250, y: 700, o: 1 },
      { t: filterClick, x: filterPoint[0], y: filterPoint[1], o: 1 },
      { t: filtered + 0.4, x: filterPoint[0], y: filterPoint[1], o: 1 },
      { t: scripts, x: tabPoint[0], y: tabPoint[1], o: 1 },
      { t: scripts + 0.8, x: tabPoint[0] + 40, y: tabPoint[1] + 120, o: 1 },
      { t: outro, x: tabPoint[0] + 40, y: tabPoint[1] + 120, o: 1 },
      { t: outro + 0.5, x: tabPoint[0] + 40, y: tabPoint[1] + 120, o: 0 },
      { t: duration, x: tabPoint[0] + 40, y: tabPoint[1] + 120, o: 0 },
    ],
    clicks: [
      { t: filterClick, x: filterPoint[0], y: filterPoint[1] },
      { t: scripts, x: tabPoint[0], y: tabPoint[1] },
    ],
    rings: [{ from: sendScripts + 0.6, to: outro, b: [post.x[0] - 8, post.y - 16, 170, 22] }],
    keys: [
      { from: urlTyped + 0.3, to: send + 0.4, down: send - 0.15, keys: ["⌘", "↵"] },
      { from: sendScripts - 0.5, to: sendScripts + 0.4, down: sendScripts - 0.15, keys: ["⌘", "↵"] },
    ],
    captions: [
      { from: 0.9, to: urlTyped + 0.2, text: "The query table and the URL stay in sync" },
      { from: urlTyped + 0.4, to: filterClick - 0.4, text: "Send with ⌘ Enter" },
      { from: filterClick - 0.2, to: filtered + 1.2, text: "Filter a JSON response with JSONPath" },
      { from: scripts - 0.2, to: sendScripts + 0.3, text: "Pre-request and post-response scripts" },
      { from: sendScripts + 0.5, to: outro + 0.5, text: "A script can save the token to the environment" },
    ],
  };
}

/// Which keyframes show at `t`.
function layerState(plan, t) {
  const { typed } = plan;
  const urlTyping = typed.slice(0, 3);
  if (t < urlTyping[0].start) return { base: "01-start" };
  for (const segment of urlTyping) {
    if (t < segment.start) return { base: segment.below };
    if (t < segment.end) return typingState(segment, t);
  }
  if (t < plan.send) return { base: urlTyping[2].key };
  if (t < typed[3].start) return fade("04-query-3", "05-response", (t - plan.send) / 0.15);
  for (const segment of typed.slice(3)) {
    if (t < segment.start) return { base: segment.below };
    if (t < segment.end) return typingState(segment, t);
    if (t < segment.end + 0.15) return fade(segment.below, segment.key, (t - segment.end) / 0.15);
  }
  if (t < plan.scripts) return { base: "07-filter-2" };
  if (t < plan.sendScripts) return fade("07-filter-2", "08-scripts", (t - plan.scripts) / 0.2);
  return fade("08-scripts", "09-scripts-sent", (t - plan.sendScripts) / 0.15);
}

/// The keyframe that holds a part typed, shown only where it has been typed,
/// over the one before it: the rest of the window changes when the part is
/// done, as the query table fills and the response is filtered.
function typingState(segment, t) {
  const shown = segment.chars.filter((c) => c.t <= t).length;
  const x = shown < segment.chars.length ? segment.chars[shown].x : segment.right;
  const band = { top: segment.top, bottom: segment.bottom };
  if (segment.field) {
    return {
      base: segment.below,
      top: segment.key,
      opacity: 1,
      reveal: { from: segment.field[0], x: segment.field[1], ...band },
      cover: { x, right: segment.right, color: segment.color, ...band },
    };
  }
  return { base: segment.below, top: segment.key, opacity: 1, reveal: { from: segment.from, x, ...band } };
}

/// The colour painted at a point: the solid fills of the rectangles over it,
/// blended in the order they were painted.
function backgroundAt(items, x, y) {
  let [r, g, b] = [0, 0, 0];
  for (const item of items) {
    if (item.t !== "rect" || item.fill?.type !== "solid" || !item.b) continue;
    const [left, top, width, height] = item.b;
    if (x < left || x > left + width || y < top || y > top + height) continue;
    const [fr, fg, fb, fa] = item.fill.color.match(/[\d.]+/g).map(Number);
    [r, g, b] = [r + (fr - r) * fa, g + (fg - g) * fa, b + (fb - b) * fa];
  }
  return `rgb(${Math.round(r)}, ${Math.round(g)}, ${Math.round(b)})`;
}

function fade(base, top, p) {
  p = clamp(p, 0, 1);
  return p >= 1 ? { base: top } : { base, top, opacity: easeInOut(p) };
}

/// Where the caret is at `t`, if it shows: in what is being typed, solid
/// while typing and blinking just before.
function caretState(plan, t) {
  for (const segment of plan.typed) {
    if (t >= segment.start - 0.6 && t < segment.end + 0.25) {
      const shown = segment.chars.filter((c) => c.t <= t).length;
      const x = shown === 0 ? segment.chars[0].x : segment.chars[shown - 1].x + segment.step;
      return {
        x,
        top: segment.top + 2,
        height: segment.bottom - segment.top - 2,
        blink: t < segment.start,
      };
    }
  }
  return null;
}

/// The elements of a response, by its line, so the lines can come in in turn.
function lineElements(layer, snapshot, region) {
  const [left, top, width, height] = region.clip;
  const step = region.size * 1.5;
  const lines = [];
  // What spans several lines, such as indent guides, comes in with the last.
  const spanning = [];
  for (const element of layer.querySelectorAll("[data-i]")) {
    const item = snapshot.items[Number(element.dataset.i)];
    const box = itemBox(item);
    if (!box) continue;
    const middle = box[1] + box[3] / 2;
    if (middle < top || middle > top + height || box[0] < left || box[0] > left + width) continue;
    if (box[3] > step * 1.6) {
      if (box[3] < height * 0.9) spanning.push(element);
      continue;
    }
    const line = Math.max(0, Math.round((middle - (region.first - region.size * 0.35)) / step));
    (lines[line] ??= []).push(element);
  }
  const all = lines.map((line) => line ?? []);
  if (all.length) all[all.length - 1].push(...spanning);
  return all;
}

/// Roughly the box an item draws in: its bounds, a text run's line, or the
/// extent of a path's points.
function itemBox(item) {
  if (item.b) return item.b;
  if (item.t === "text") return [item.x[0], item.y - item.size, 0, item.size];
  if (item.t === "path") {
    const n = item.d.match(/-?[\d.]+/g)?.map(Number) ?? [];
    const xs = n.filter((_, ix) => ix % 2 === 0);
    const ys = n.filter((_, ix) => ix % 2 === 1);
    if (!xs.length) return null;
    const [x, y] = [Math.min(...xs), Math.min(...ys)];
    return [x, y, Math.max(...xs) - x, Math.max(...ys) - y];
  }
  return null;
}

/// A clip path showing only what has been typed of a part: from where it
/// starts to `x`, on its line.
function patch({ from, x, top, bottom }) {
  if (x <= from) return "polygon(0 0, 0 0, 0 0)";
  return `polygon(${from}px ${top}px, ${x}px ${top}px, ${x}px ${bottom}px, ${from}px ${bottom}px)`;
}

/// The value of keyed `frames` at `t`, eased between keys.
function interpolate(frames, t) {
  if (t <= frames[0].t) return frames[0];
  for (let ix = 1; ix < frames.length; ix++) {
    const a = frames[ix - 1];
    const b = frames[ix];
    if (t < b.t) {
      const p = easeInOut((t - a.t) / (b.t - a.t));
      const out = {};
      for (const key of Object.keys(b)) out[key] = a[key] + (b[key] - a[key]) * p;
      return out;
    }
  }
  return frames[frames.length - 1];
}

function clamp(value, min, max) {
  return Math.min(max, Math.max(min, value));
}

function easeInOut(p) {
  return p < 0.5 ? 4 * p * p * p : 1 - (-2 * p + 2) ** 3 / 2;
}

function easeOut(p) {
  return 1 - (1 - p) ** 3;
}

let styled = false;
function injectStyles() {
  if (styled) return;
  styled = true;
  const style = document.createElement("style");
  style.textContent = `
    .bd-root { container-type: inline-size; position: relative; }
    .bd-viewport {
      position: relative; aspect-ratio: ${WIDTH} / ${HEIGHT}; overflow: hidden;
      border-radius: 12px; background: #11111b;
      box-shadow: 0 0 0 1px rgb(255 255 255 / 0.08), 0 40px 80px -30px rgb(0 0 0 / 0.6);
    }
    .bd-stage { position: absolute; left: 0; top: 0; width: ${WIDTH}px; height: ${HEIGHT}px; transform-origin: 0 0; }
    .bd-camera { position: absolute; inset: 0; transform-origin: 0 0; will-change: transform; }
    .bd-layer { position: absolute; inset: 0; }
    .bd-layer svg { display: block; }
    .bd-layer [data-i] { transform-box: fill-box; }
    .bd-overlay { position: absolute; inset: 0; pointer-events: none; }
    .bd-overlay > * { position: absolute; left: 0; top: 0; }
    .bd-blank { position: absolute; left: 0; top: 0; z-index: 3; display: none; }
    .bd-caret { background: ${CARET}; border-radius: 1px; }
    .bd-pointer { width: 18px; height: 25px; filter: drop-shadow(0 2px 3px rgb(0 0 0 / 0.4)); }
    .bd-ripple { border-radius: 50%; background: #cba6f7; }
    .bd-ring { border: 2px solid #cba6f7; border-radius: 8px; box-shadow: 0 0 0 4px rgb(203 166 247 / 0.18); }
    .bd-keys {
      left: 50%; top: auto; bottom: 18%; display: flex; gap: 8px;
    }
    .bd-keys kbd {
      font: 600 clamp(14px, 2.4cqw, 26px)/1 system-ui, -apple-system, sans-serif;
      min-width: 1.9em; padding: 0.45em 0.6em; text-align: center; color: #cdd6f4;
      background: #313244; border-radius: 0.35em;
      box-shadow: 0 0.18em 0 #181825, 0 10px 24px -8px rgb(0 0 0 / 0.6), inset 0 1px 0 rgb(255 255 255 / 0.08);
      transition: transform 80ms, box-shadow 80ms;
    }
    .bd-keys.bd-pressed kbd { transform: translateY(0.12em); box-shadow: 0 0.06em 0 #181825, 0 6px 16px -8px rgb(0 0 0 / 0.6); }
    .bd-caption {
      left: 50%; top: 4.5%; transform: translateX(-50%);
      font: 500 clamp(12px, 1.7cqw, 19px)/1.2 system-ui, -apple-system, sans-serif;
      color: #cdd6f4; white-space: nowrap; padding: 0.6em 1em; border-radius: 999px;
      background: rgb(17 17 27 / 0.82); backdrop-filter: blur(8px);
      box-shadow: 0 0 0 1px rgb(255 255 255 / 0.08), 0 12px 28px -12px rgb(0 0 0 / 0.7);
    }
    .bd-caption:empty { display: none; }
    /* The hint shows for a while, then clears the tabs it covers. */
    .bd-ended .bd-caption { animation: bd-hint 5s 0.9s both; }

    .bd-dim { inset: 0; width: 100%; height: 100%; opacity: 0; transition: opacity 0.25s; }
    .bd-dim.bd-on { opacity: 1; }

    .bd-replay {
      left: auto; top: auto; right: 2.2%; bottom: 3.5%; display: none; align-items: center; gap: 0.45em;
      font: 500 clamp(11px, 1.3cqw, 15px)/1 system-ui, -apple-system, sans-serif; color: #cdd6f4;
      padding: 0.6em 0.9em; border: 0; border-radius: 999px; cursor: pointer; pointer-events: auto;
      background: rgb(49 50 68 / 0.9); box-shadow: 0 0 0 1px rgb(255 255 255 / 0.08), 0 10px 24px -10px rgb(0 0 0 / 0.7);
    }
    .bd-replay svg { width: 1.1em; height: 1.1em; }
    .bd-replay:hover { background: #45475a; }
    .bd-ended .bd-replay { display: inline-flex; animation: bd-in 0.4s 1.2s both; }

    .bd-spots { position: absolute; inset: 0; pointer-events: none; }
    .bd-spot { position: absolute; display: none; z-index: 1; }
    .bd-spot.bd-open { z-index: 2; }
    .bd-ended .bd-spot { display: block; animation: bd-pop 0.45s calc(0.25s + var(--i) * 0.09s) both; }
    .bd-dot {
      position: absolute; width: 16px; height: 16px; margin: -8px 0 0 -8px; padding: 0;
      border: 0; border-radius: 50%; cursor: pointer; pointer-events: auto;
      background: radial-gradient(circle, #fff 0 22%, #cba6f7 32% 100%);
      box-shadow: 0 0 0 0 rgb(203 166 247 / 0.7), 0 0 14px 2px rgb(203 166 247 / 0.65);
      animation: bd-glow 2s calc(var(--i) * -0.3s) infinite;
    }
    .bd-dot:focus-visible { outline: 2px solid #f5e0dc; outline-offset: 3px; }
    .bd-open .bd-dot { animation: none; box-shadow: 0 0 0 5px rgb(203 166 247 / 0.3), 0 0 18px 4px rgb(203 166 247 / 0.8); }
    .bd-card {
      position: absolute; width: min(270px, 70vw); box-sizing: border-box; padding: 14px 16px;
      pointer-events: auto; visibility: hidden; opacity: 0; transition: opacity 0.18s, transform 0.18s, visibility 0.18s;
      font: 400 13.5px/1.45 system-ui, -apple-system, sans-serif; color: #bac2de;
      background: rgb(30 30 46 / 0.97); border-radius: 12px;
      box-shadow: 0 0 0 1px rgb(203 166 247 / 0.35), 0 24px 48px -16px rgb(0 0 0 / 0.75);
    }
    .bd-card.bd-wide { width: min(360px, 80vw); }
    .bd-media {
      margin: -4px -6px 12px; border-radius: 8px; overflow: hidden; background: #11111b;
      box-shadow: 0 0 0 1px rgb(255 255 255 / 0.08);
    }
    .bd-media svg { display: block; width: 100%; height: 100%; }
    .bd-card strong { display: block; font-weight: 600; font-size: 14.5px; color: #cdd6f4; margin-bottom: 4px; }
    .bd-card p { margin: 0 0 8px; }
    .bd-card a { color: #cba6f7; font-weight: 500; text-decoration: none; }
    .bd-card a:hover { text-decoration: underline; }
    .bd-card a::after { content: " →"; }
    .bd-spot[data-side="right"] .bd-card { left: 16px; }
    .bd-spot[data-side="left"] .bd-card { right: 16px; }
    .bd-spot[data-rise="down"] .bd-card { top: -18px; }
    .bd-spot[data-rise="up"] .bd-card { bottom: -18px; }
    .bd-spot[data-side="right"] .bd-card { transform: translateX(-6px); }
    .bd-spot[data-side="left"] .bd-card { transform: translateX(6px); }
    .bd-open .bd-card { visibility: visible; opacity: 1; transform: none !important; }

    @keyframes bd-glow {
      0% { box-shadow: 0 0 0 0 rgb(203 166 247 / 0.65), 0 0 14px 2px rgb(203 166 247 / 0.6); }
      70% { box-shadow: 0 0 0 12px rgb(203 166 247 / 0), 0 0 14px 2px rgb(203 166 247 / 0.6); }
      100% { box-shadow: 0 0 0 0 rgb(203 166 247 / 0), 0 0 14px 2px rgb(203 166 247 / 0.6); }
    }
    @keyframes bd-pop { from { opacity: 0; transform: scale(0.4); } to { opacity: 1; transform: none; } }
    @keyframes bd-in { from { opacity: 0; } }
    @keyframes bd-hint { 0% { opacity: 0; } 10%, 80% { opacity: 0.9; } 100% { opacity: 0; } }
    @media (prefers-reduced-motion: reduce) {
      .bd-dot, .bd-ended .bd-spot, .bd-ended .bd-caption, .bd-ended .bd-replay { animation: none; }
    }
  `;
  document.head.append(style);
}
