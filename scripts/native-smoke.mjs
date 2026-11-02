#!/usr/bin/env node
// Launches the real built application and verifies what the native window shows.
//
//   npm run smoke:native -- [--binary path] [--evidence dir]
//
// Windows: drives WebView2 over the DevTools protocol (full proof).
// Linux:   drives WebKitGTK through tauri-driver (run under xvfb; DOM proof).
// macOS:   no WebDriver exists for WKWebView, so this only proves the process starts
//          and stays alive. It reports "process-only" and must never be read as more.
import { execFileSync, spawn } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = process.argv.slice(2);
const option = (name) => {
  const index = args.indexOf(name);
  return index === -1 ? undefined : args[index + 1];
};
const isWindows = process.platform === "win32";
const binary = resolve(
  option("--binary") ?? join(ROOT, "target", "release", `maddentool${isWindows ? ".exe" : ""}`),
);
const evidenceDir = option("--evidence");
const STARTUP_TIMEOUT_MS = 45_000;

const sleep = (ms) => new Promise((done) => setTimeout(done, ms));

async function until(what, probe, timeoutMs = STARTUP_TIMEOUT_MS) {
  const deadline = Date.now() + timeoutMs;
  let lastError;
  while (Date.now() < deadline) {
    try {
      const value = await probe();
      if (value) return value;
    } catch (error) {
      lastError = error;
    }
    await sleep(250);
  }
  throw new Error(`Timed out waiting for ${what}${lastError ? `: ${lastError.message}` : ""}`);
}

function expectedFacts() {
  const manifest = readFileSync(join(ROOT, "Cargo.toml"), "utf8");
  const version = /\[workspace\.package\][^[]*?version\s*=\s*"([^"]+)"/.exec(manifest)?.[1];
  if (!version) throw new Error("Could not read the workspace version from Cargo.toml");
  const platform = { win32: "windows", darwin: "macos", linux: "linux" }[process.platform];
  const rustc = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
  const arch = /^host: ([^-]+)-/m.exec(rustc)?.[1];
  return { Game: "Madden 27", Version: version, Platform: platform, Architecture: arch };
}

function assertFacts(rendered) {
  const expected = expectedFacts();
  for (const [label, value] of Object.entries(expected)) {
    if (rendered[label] !== value) {
      throw new Error(`${label}: expected "${value}", window shows "${rendered[label]}"`);
    }
  }
  return expected;
}

// Reads the visible facts. Works in both CDP Runtime.evaluate and WebDriver execute.
const READ_FACTS = `(() => {
  const facts = {};
  for (const row of document.querySelectorAll(".startup__facts > div")) {
    facts[row.querySelector("dt").textContent] = row.querySelector("dd").textContent;
  }
  const alert = document.querySelector('[role="alert"]');
  return JSON.stringify({ title: document.title, heading: document.querySelector("h1")?.textContent,
    facts, alert: alert ? alert.textContent : null });
})()`;

function checkRendered(state) {
  if (state.alert) throw new Error(`Window shows an error: ${state.alert}`);
  if (state.heading !== "MaddenTool") throw new Error(`Unexpected heading: ${state.heading}`);
  return assertFacts(state.facts);
}

function killTree(child) {
  if (child.exitCode !== null) return;
  if (isWindows) {
    execFileSync("taskkill", ["/pid", String(child.pid), "/t", "/f"], { stdio: "ignore" });
  } else {
    child.kill("SIGTERM");
  }
}

class Cdp {
  #id = 0;
  #pending = new Map();
  events = [];
  onEvent;

  static async connect(url) {
    const cdp = new Cdp();
    cdp.socket = new WebSocket(url);
    await new Promise((open, fail) => {
      cdp.socket.addEventListener("open", open, { once: true });
      cdp.socket.addEventListener("error", () => fail(new Error("CDP connect failed")), {
        once: true,
      });
    });
    cdp.socket.addEventListener("message", ({ data }) => {
      const message = JSON.parse(data);
      if (message.id === undefined) {
        cdp.events.push(message);
        cdp.onEvent?.(message);
        return;
      }
      const call = cdp.#pending.get(message.id);
      cdp.#pending.delete(message.id);
      if (message.error) call.fail(new Error(message.error.message));
      else call.done(message.result);
    });
    return cdp;
  }

  send(method, params = {}) {
    const id = ++this.#id;
    return new Promise((done, fail) => {
      this.#pending.set(id, { done, fail });
      this.socket.send(JSON.stringify({ id, method, params }));
    });
  }

  async evaluate(expression) {
    const { result, exceptionDetails } = await this.send("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
    });
    if (exceptionDetails) throw new Error(exceptionDetails.exception?.description ?? "eval failed");
    return result.value;
  }
}

// A rejection proves the capability boundary from inside the real webview.
const denied = (command) => `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(command)})
  .then(() => "ALLOWED", (error) => "denied: " + String(error))`;

// CSS-pixel viewports: a window at 200% zoom has half the CSS pixels.
const VIEWPORTS = [
  { name: "800x600", width: 800, height: 600, scale: 1 },
  { name: "1024x768", width: 1024, height: 768, scale: 1 },
  { name: "1440x900", width: 1440, height: 900, scale: 1 },
  { name: "1920x1080", width: 1920, height: 1080, scale: 1 },
  { name: "800x600-at-200pct", width: 400, height: 300, scale: 2 },
  { name: "1024x768-at-200pct", width: 512, height: 384, scale: 2 },
];

const MEASURE = `JSON.stringify({
  overflowX: document.documentElement.scrollWidth - document.documentElement.clientWidth,
  clipped: [...document.querySelectorAll("h1, dt, dd, p")].filter((node) => {
    const box = node.getBoundingClientRect();
    return box.left < 0 || box.right > document.documentElement.clientWidth + 0.5;
  }).length,
})`;

async function smokeWindows() {
  const port = 9300 + (process.pid % 500);
  const child = spawn(binary, [], {
    stdio: "ignore",
    env: {
      ...process.env,
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`,
    },
  });
  try {
    const target = await until("the WebView2 debug target", async () => {
      const targets = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
      return targets.find((entry) => entry.type === "page");
    });
    const cdp = await Cdp.connect(target.webSocketDebuggerUrl);
    await cdp.send("Runtime.enable");
    await cdp.send("Log.enable");

    const state = JSON.parse(
      await until("the startup facts", async () => {
        const raw = await cdp.evaluate(READ_FACTS);
        const parsed = JSON.parse(raw);
        return parsed.alert || Object.keys(parsed.facts).length > 0 ? raw : null;
      }),
    );
    const facts = checkRendered(state);

    // Prove narrow capabilities from the real webview.
    const boundary = {
      get_app_info: await cdp.evaluate(`window.__TAURI_INTERNALS__.invoke("get_app_info")
        .then((value) => JSON.stringify(value))`),
      unregistered_command: await cdp.evaluate(denied("read_file")),
      core_window_plugin: await cdp.evaluate(denied("plugin:window|set_title")),
      core_webview_plugin: await cdp.evaluate(denied("plugin:webview|create_webview")),
    };
    const allowed = Object.entries(boundary).filter(([, value]) => value === "ALLOWED");
    if (allowed.length > 0) throw new Error(`Not denied: ${allowed.map(([name]) => name)}`);
    if (JSON.parse(boundary.get_app_info).game !== "Madden 27") {
      throw new Error(`Native command mismatch: ${boundary.get_app_info}`);
    }

    const layouts = [];
    for (const viewport of VIEWPORTS) {
      await cdp.send("Emulation.setDeviceMetricsOverride", {
        width: viewport.width,
        height: viewport.height,
        deviceScaleFactor: viewport.scale,
        mobile: false,
      });
      await sleep(150);
      const measured = JSON.parse(await cdp.evaluate(MEASURE));
      layouts.push({ ...viewport, ...measured });
      if (evidenceDir) {
        const { data } = await cdp.send("Page.captureScreenshot", { format: "png" });
        writeFileSync(join(evidenceDir, `${viewport.name}.png`), Buffer.from(data, "base64"));
      }
    }
    await cdp.send("Emulation.clearDeviceMetricsOverride");
    const broken = layouts.filter((layout) => layout.overflowX > 0 || layout.clipped > 0);
    if (broken.length > 0) throw new Error(`Layout overflow: ${JSON.stringify(broken)}`);

    // Answer the first IPC call after a reload with a malformed payload, so the real wrapper
    // rejects it, then recover with the keyboard through real IPC. Tauri locks its own invoke,
    // so the failure is injected at the IPC HTTP layer rather than by patching JavaScript.
    await cdp.send("Page.enable");
    let injected = false;
    cdp.onEvent = (event) => {
      if (event.method !== "Fetch.requestPaused") return;
      const { requestId, request } = event.params;
      if (injected || request.method !== "POST") {
        void cdp.send("Fetch.continueRequest", { requestId });
        return;
      }
      injected = true;
      void cdp.send("Fetch.fulfillRequest", {
        requestId,
        responseCode: 200,
        responseHeaders: [
          { name: "Content-Type", value: "application/json" },
          { name: "Tauri-Response", value: "ok" },
          { name: "Access-Control-Allow-Origin", value: "*" },
          { name: "Access-Control-Expose-Headers", value: "Tauri-Response" },
        ],
        body: Buffer.from(JSON.stringify({ injected: true })).toString("base64"),
      });
    };
    await cdp.send("Fetch.enable", { patterns: [{ urlPattern: "*ipc.localhost*" }] });
    await cdp.send("Page.reload");
    const retryButton = `document.querySelector('[role="alert"] button')`;
    await until("the injected error state", () => cdp.evaluate(`!!${retryButton}`));
    const errorText = await cdp.evaluate(`document.querySelector('[role="alert"]').textContent`);
    if (!/unexpected app information/i.test(errorText)) throw new Error(`Error: ${errorText}`);
    await cdp.send("Emulation.setDeviceMetricsOverride", {
      width: 400,
      height: 300,
      deviceScaleFactor: 2,
      mobile: false,
    });
    await sleep(150);
    const retryLayout = JSON.parse(
      await cdp.evaluate(`(() => {
        const button = ${retryButton};
        button.scrollIntoView();
        const box = button.getBoundingClientRect();
        return JSON.stringify({
          focused: (button.focus(), document.activeElement === button),
          reachable: box.top >= 0 && box.bottom <= innerHeight && box.left >= 0 && box.right <= innerWidth,
          overflowX: document.documentElement.scrollWidth - document.documentElement.clientWidth,
        });
      })()`),
    );
    if (!retryLayout.focused || !retryLayout.reachable || retryLayout.overflowX > 0) {
      throw new Error(`Retry is not usable at 200% zoom: ${JSON.stringify(retryLayout)}`);
    }
    if (evidenceDir) {
      const { data } = await cdp.send("Page.captureScreenshot", { format: "png" });
      writeFileSync(join(evidenceDir, "error-at-200pct.png"), Buffer.from(data, "base64"));
    }
    await cdp.send("Emulation.clearDeviceMetricsOverride");
    const enter = { key: "Enter", code: "Enter", windowsVirtualKeyCode: 13, text: "\r" };
    await cdp.send("Input.dispatchKeyEvent", { type: "keyDown", ...enter });
    await cdp.send("Input.dispatchKeyEvent", { type: "keyUp", ...enter });
    await cdp.send("Fetch.disable");
    const recovered = JSON.parse(
      await until("recovery through Retry", async () => {
        const raw = await cdp.evaluate(READ_FACTS);
        return Object.keys(JSON.parse(raw).facts).length > 0 ? raw : null;
      }),
    );
    checkRendered(recovered);

    const violations = cdp.events
      .filter((event) => event.method === "Log.entryAdded")
      .map((event) => event.params.entry)
      .filter((entry) => entry.level === "error" || /content security policy/i.test(entry.text));
    // The rejected payload is deliberate and is not logged as a console error by the app.
    if (violations.length > 0) {
      throw new Error(`Console errors or CSP violations: ${JSON.stringify(violations)}`);
    }

    cdp.socket.close();
    return {
      level: "native-dom-and-ipc",
      facts,
      boundary,
      layouts,
      retry: { errorShown: errorText, ...retryLayout, recoveredWithKeyboard: true },
      consoleErrors: 0,
    };
  } finally {
    killTree(child);
  }
}

async function smokeLinux() {
  const driver = spawn("tauri-driver", [], { stdio: "inherit" });
  const base = "http://127.0.0.1:4444";
  const call = async (method, path, body) => {
    const response = await fetch(`${base}${path}`, {
      method,
      headers: { "content-type": "application/json" },
      ...(body === undefined ? {} : { body: JSON.stringify(body) }),
    });
    const { value } = await response.json();
    if (!response.ok) throw new Error(`${path}: ${value?.message ?? response.status}`);
    return value;
  };
  try {
    const session = await until("tauri-driver", () =>
      call("POST", "/session", {
        capabilities: {
          alwaysMatch: { browserName: "wry", "tauri:options": { application: binary } },
        },
      }),
    );
    const id = session.sessionId;
    const raw = await until("the startup facts", async () => {
      const value = await call("POST", `/session/${id}/execute/sync`, {
        script: `return ${READ_FACTS};`,
        args: [],
      });
      const parsed = JSON.parse(value);
      return parsed.alert || Object.keys(parsed.facts).length > 0 ? value : null;
    });
    const facts = checkRendered(JSON.parse(raw));
    await call("DELETE", `/session/${id}`);
    return { level: "native-dom", facts };
  } finally {
    driver.kill("SIGTERM");
  }
}

async function smokeProcessOnly() {
  const ALIVE_MS = 8_000;
  const child = spawn(binary, [], { stdio: "inherit" });
  const exited = new Promise((done) => child.once("exit", (code) => done(code ?? "signal")));
  const outcome = await Promise.race([exited, sleep(ALIVE_MS).then(() => "alive")]);
  killTree(child);
  if (outcome !== "alive") throw new Error(`Application exited early with ${outcome}`);
  return { level: "process-only", note: `Stayed alive ${ALIVE_MS} ms; window content not read.` };
}

if (!existsSync(binary)) {
  console.error(`Application binary not found: ${binary}\nBuild it first with: npm run build`);
  process.exit(1);
}
if (evidenceDir) mkdirSync(evidenceDir, { recursive: true });

const run = { win32: smokeWindows, linux: smokeLinux, darwin: smokeProcessOnly }[process.platform];
if (!run) throw new Error(`No smoke strategy for ${process.platform}`);

try {
  const result = await run();
  const report = JSON.stringify({ platform: process.platform, ...result }, null, 2);
  console.log(report);
  if (evidenceDir) writeFileSync(join(evidenceDir, "smoke.json"), `${report}\n`);
  process.exit(0);
} catch (error) {
  console.error(`Native smoke failed: ${error.message}`);
  process.exit(1);
}
