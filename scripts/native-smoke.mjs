#!/usr/bin/env node
// Launches the real built application and verifies what the native window shows.
//
//   npm run smoke:native -- [--binary path] [--evidence dir]
//   npm run smoke:native -- --negative-probe <name>
//
// Windows: drives WebView2 over the DevTools protocol (full proof).
// Linux:   drives WebKitGTK through tauri-driver (run under xvfb; DOM proof).
// macOS:   no WebDriver exists for WKWebView, so this only proves the process starts
//          and stays alive. It reports "process-only" and must never be read as more.
//
// A negative probe is not an acceptance run. It breaks something on purpose, on Windows, and
// succeeds only if the matching guard fails the run. It never writes smoke.json.
import { execFileSync, spawn } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  CORE_ACL_PROBES,
  Cdp,
  Cleanups,
  INVALID_ARGUMENTS_PROBE,
  OTHER_APP_ORIGIN,
  UNREGISTERED_COMMAND,
  WEBVIEW2_DEBUG_ARGUMENTS,
  captureOutput,
  classifyAclProbe,
  classifyUnregisteredProbe,
  cleanUpOnCrash,
  consoleProblems,
  createWebView2Profile,
  diagnoseStartup,
  findOwnDevTools,
  inspectWebView2Profile,
  interceptFetch,
  invokeExpression,
  isAppUrl,
  redact,
  removeWebView2Profile,
  requestJson,
  sleep,
  startWebDriverSession,
  stopProcess,
  until,
  waitForAppTarget,
  watchProcess,
  webDriverClient,
  webView2DataFolder,
} from "./smoke-harness.mjs";

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
const negativeProbe = option("--negative-probe");
// Each Windows run gets a fresh WebView2 profile here, removed afterwards.
const profileRoot = resolve(option("--profile-root") ?? join(ROOT, ".smoke", "webview2-profiles"));

const consoleProbe = (inject, text) => ({
  inject: inject(JSON.stringify(text)),
  expect: new RegExp(`Console errors or CSP violations.*${text}`),
});
const NEGATIVE_PROBES = {
  "console-error": consoleProbe((text) => `console.error(${text})`, "probe-console"),
  "uncaught-exception": consoleProbe(
    (text) => `setTimeout(() => { throw new Error(${text}); }, 0)`,
    "probe-uncaught",
  ),
  "unhandled-rejection": consoleProbe(
    (text) => `void Promise.reject(new Error(${text}))`,
    "probe-rejection",
  ),
  // Only trips where set_title is granted: the ACL answers first in the shipped app.
  "acl-invalid-arguments": { expect: /Capability boundary not proven.*invalid args/s },
};
const probe = negativeProbe ? NEGATIVE_PROBES[negativeProbe] : undefined;

const cleanups = new Cleanups();
cleanUpOnCrash(cleanups);

function expectedFacts() {
  const manifest = readFileSync(join(ROOT, "Cargo.toml"), "utf8");
  const version = /\[workspace\.package\][^[]*?version\s*=\s*"([^"]+)"/.exec(manifest)?.[1];
  if (!version) throw new Error("Could not read the workspace version from Cargo.toml");
  const platform = { win32: "windows", darwin: "macos", linux: "linux" }[process.platform];
  const rustc = execFileSync("rustc", ["-vV"], { encoding: "utf8", timeout: 30_000 });
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
  return JSON.stringify({ url: location.href, title: document.title,
    heading: document.querySelector("h1")?.textContent,
    facts, alert: alert ? alert.textContent : null });
})()`;

function checkRendered(state) {
  if (state.alert) throw new Error(`Window shows an error: ${state.alert}`);
  if (state.heading !== "MaddenTool") throw new Error(`Unexpected heading: ${state.heading}`);
  return assertFacts(state.facts);
}

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

// Requires Tauri's exact answer from the real webview. Registered core commands are the
// capability proof. The unregistered command is reported separately.
async function checkBoundary(cdp) {
  const probes = CORE_ACL_PROBES.map((entry) =>
    negativeProbe === "acl-invalid-arguments" && entry.name === INVALID_ARGUMENTS_PROBE.name
      ? INVALID_ARGUMENTS_PROBE
      : entry,
  );
  const aclDenied = {};
  const failures = [];
  for (const { name, command, args: commandArgs } of probes) {
    const answer = classifyAclProbe(
      command,
      await cdp.evaluate(invokeExpression(command, commandArgs)),
    );
    if (answer.denied) aclDenied[name] = answer.error;
    else failures.push({ command, ...answer });
  }
  const unregistered = classifyUnregisteredProbe(
    UNREGISTERED_COMMAND,
    await cdp.evaluate(invokeExpression(UNREGISTERED_COMMAND)),
  );
  if (!unregistered.rejected) failures.push({ command: UNREGISTERED_COMMAND, ...unregistered });
  if (failures.length > 0) {
    throw new Error(`Capability boundary not proven: ${JSON.stringify(failures)}`);
  }
  const appInfo = await cdp.evaluate(`window.__TAURI_INTERNALS__.invoke("get_app_info")
    .then((value) => JSON.stringify(value))`);
  if (JSON.parse(appInfo).game !== "Madden 27") {
    throw new Error(`Native command mismatch: ${appInfo}`);
  }
  return {
    get_app_info: appInfo,
    aclDenied,
    unregisteredCommand: { command: UNREGISTERED_COMMAND, rejection: unregistered.error },
  };
}

async function smokeWindows() {
  // The debug server belongs to this run's profile: the browser picks a free port and records it
  // there. A fixed port could be held by another WebView2 app, whose page has the same origin.
  const profile = createWebView2Profile(profileRoot);
  cleanups.add("WebView2 profile", () => removeWebView2Profile(profileRoot, profile));
  const portFile = join(webView2DataFolder(profile), "DevToolsActivePort");
  let child;
  try {
    child = spawn(binary, [], {
      stdio: ["ignore", "pipe", "pipe"],
      env: {
        ...process.env,
        WEBVIEW2_USER_DATA_FOLDER: profile,
        WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: WEBVIEW2_DEBUG_ARGUMENTS,
      },
    });
  } catch (error) {
    // Some failures throw here instead of raising the process's error event.
    const diagnosis = { classification: "spawn-error", spawnError: redact(error.message) };
    throw new Error(`The application could not start; startup=${JSON.stringify(diagnosis)}`);
  }
  const app = watchProcess(child, "The application");
  const readOutput = captureOutput(child);
  cleanups.add("application", () => stopProcess(child, "The application"));
  const endpoint = {};
  const discovery = {};
  let devTools;
  let target;
  let cdp;
  try {
    try {
      target = await waitForAppTarget(
        async (signal) => {
          devTools ??= await findOwnDevTools({
            readPortFile: () => readFile(portFile, "utf8"),
            requestVersion: async (url) =>
              (await requestJson(url, { timeoutMs: 2_000, signal })).json,
            endpoint: discovery,
          });
          const { json } = await requestJson(`${devTools.base}/json/list`, {
            timeoutMs: 5_000,
            signal,
          });
          // Page sockets are on the same server; anything else is not this run's browser.
          return json.filter(
            (entry) =>
              !entry.webSocketDebuggerUrl ||
              new URL(entry.webSocketDebuggerUrl).port === String(devTools.port),
          );
        },
        { endpoint, guard: app.guard },
      );
    } catch (error) {
      // Gathered before cleanup stops the process, so a blocking dialog is still visible.
      const diagnosis = await diagnoseStartup({
        child,
        state: app,
        readOutput,
        endpoint: { ...endpoint, devTools: discovery },
        profile: inspectWebView2Profile(profile),
      });
      throw new Error(`${error.message}; startup=${JSON.stringify(diagnosis)}`, { cause: error });
    }
    cdp = await Cdp.connect(target.webSocketDebuggerUrl);
    cleanups.add("DevTools socket", () => cdp.close());
    await cdp.send("Runtime.enable");
    await cdp.send("Log.enable");

    const state = JSON.parse(
      await until(
        "the startup facts",
        async () => {
          const raw = await cdp.evaluate(READ_FACTS);
          const parsed = JSON.parse(raw);
          return parsed.alert || Object.keys(parsed.facts).length > 0 ? raw : null;
        },
        { guard: app.guard },
      ),
    );
    const facts = checkRendered(state);

    const boundary = await checkBoundary(cdp);

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
    const ipcRequests = [];
    const interception = interceptFetch(cdp, ({ request }) => {
      ipcRequests.push(`${request.method} ${request.url}`);
      if (injected || request.method !== "POST" || !request.url.includes("get_app_info")) {
        return null;
      }
      injected = true;
      return {
        method: "Fetch.fulfillRequest",
        params: {
          responseCode: 200,
          responseHeaders: [
            { name: "Content-Type", value: "application/json" },
            { name: "Tauri-Response", value: "ok" },
            { name: "Access-Control-Allow-Origin", value: "*" },
            { name: "Access-Control-Expose-Headers", value: "Tauri-Response" },
          ],
          body: Buffer.from(JSON.stringify({ injected: true })).toString("base64"),
        },
      };
    });
    const releaseInterception = cleanups.add("Fetch interception", () => interception.stop());
    await cdp.send("Fetch.enable", { patterns: [{ urlPattern: "*ipc.localhost*" }] });
    await cdp.send("Page.reload");
    const retryButton = `document.querySelector('[role="alert"] button')`;
    await until("the injected error state", () => cdp.evaluate(`!!${retryButton}`), {
      timeoutMs: 20_000,
      guard: () => {
        app.guard();
        interception.assertHealthy();
      },
    }).catch(async (error) => {
      const page = await cdp.evaluate(READ_FACTS, { timeoutMs: 2_000 }).catch(() => "unreadable");
      throw new Error(`${error.message}; ipc=${JSON.stringify(ipcRequests)}; page=${page}`);
    });
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
    releaseInterception();
    const { teardownRaces } = await interception.stop();
    const recovered = JSON.parse(
      await until(
        "recovery through Retry",
        async () => {
          const raw = await cdp.evaluate(READ_FACTS);
          return Object.keys(JSON.parse(raw).facts).length > 0 ? raw : null;
        },
        { guard: app.guard },
      ),
    );
    checkRendered(recovered);

    if (probe?.inject) await cdp.evaluate(probe.inject);
    // Exceptions and rejections are reported after the task that raised them finishes.
    await sleep(250);
    await cdp.evaluate("1");
    const problems = consoleProblems(cdp.events);
    if (problems.length > 0) {
      throw new Error(`Console errors or CSP violations: ${JSON.stringify(problems)}`);
    }

    return {
      level: "native-dom-and-ipc",
      target: target.url,
      debugEndpoint: {
        host: devTools.host,
        port: "chosen by the browser, read from this run's DevToolsActivePort",
        profile: "fresh per run, removed afterwards",
      },
      facts,
      boundary,
      layouts,
      retry: { errorShown: errorText, ...retryLayout, recoveredWithKeyboard: true },
      consoleProblems: 0,
      teardownRaces,
      lateCdpResponses: cdp.lateResponses,
    };
  } catch (error) {
    const page = cdp
      ? await cdp.evaluate("location.href", { timeoutMs: 2_000 }).catch(() => "unreadable")
      : "not attached";
    throw new Error(`${error.message}; target=${target?.url ?? "none selected"}; page=${page}`, {
      cause: error,
    });
  }
}

// Observed on the hosted ubuntu-24.04 runner: one session took about 32 s to create, most of it
// before WebKitGTK initialised EGL, because tauri-driver launches the application for it. A retry
// is a new launch that starts that wait again, so the session gets one long attempt.
const SESSION_TIMEOUT_MS = 90_000;

async function smokeLinux() {
  // Its own process group, so cleanup also reaches WebKitWebDriver and the application it launched.
  const driver = spawn("tauri-driver", [], { stdio: "inherit", detached: true });
  const driverState = watchProcess(driver, "tauri-driver");
  cleanups.add("tauri-driver", () => stopProcess(driver, "tauri-driver", { group: true }));
  const call = webDriverClient("http://127.0.0.1:4444");
  const { session, sessionMs } = await startWebDriverSession(
    call,
    { alwaysMatch: { browserName: "wry", "tauri:options": { application: binary } } },
    { watch: driverState, sessionTimeoutMs: SESSION_TIMEOUT_MS },
  );
  const id = session.sessionId;
  cleanups.add("WebDriver session", () => call("DELETE", `/session/${id}`));
  const raw = await until(
    "the startup facts",
    async () => {
      const value = await call("POST", `/session/${id}/execute/sync`, {
        script: `return ${READ_FACTS};`,
        args: [],
      });
      const parsed = JSON.parse(value);
      return parsed.alert || Object.keys(parsed.facts).length > 0 ? value : null;
    },
    { guard: driverState.guard },
  );
  const state = JSON.parse(raw);
  if (!isAppUrl(state.url, OTHER_APP_ORIGIN)) {
    throw new Error(`The session is not on the application page: ${state.url}`);
  }
  return { level: "native-dom", target: state.url, sessionMs, facts: checkRendered(state) };
}

async function smokeProcessOnly() {
  const ALIVE_MS = 8_000;
  const child = spawn(binary, [], { stdio: "inherit" });
  const app = watchProcess(child, "The application");
  cleanups.add("application", () => stopProcess(child, "The application"));
  const deadline = Date.now() + ALIVE_MS;
  while (Date.now() < deadline) {
    app.guard();
    await sleep(250);
  }
  app.guard();
  return {
    level: "process-only",
    note: `Stayed alive ${ALIVE_MS} ms; window content not read.`,
    rendering: "not proven",
    ipc: "not proven",
  };
}

if (!existsSync(binary)) {
  console.error(`Application binary not found: ${binary}\nBuild it first with: npm run build`);
  process.exit(1);
}
if (negativeProbe && (!probe || !isWindows)) {
  console.error(
    `Negative probes run on Windows only. Available: ${Object.keys(NEGATIVE_PROBES).join(", ")}`,
  );
  process.exit(1);
}
if (evidenceDir && !probe) mkdirSync(evidenceDir, { recursive: true });

const run = { win32: smokeWindows, linux: smokeLinux, darwin: smokeProcessOnly }[process.platform];
if (!run) throw new Error(`No smoke strategy for ${process.platform}`);

let result;
let failure;
try {
  result = await run();
} catch (error) {
  failure = error;
}
const cleanupFailures = await cleanups.run();
for (const problem of cleanupFailures) console.error(`Cleanup failed: ${problem}`);

if (probe) {
  const tripped = failure && probe.expect.test(failure.message);
  console.log(
    tripped
      ? `Negative probe ${negativeProbe}: guard failed the run as required.\n${failure.message}`
      : `Negative probe ${negativeProbe}: GUARD DID NOT TRIP.\n${failure?.message ?? "The run passed."}`,
  );
  process.exit(tripped && cleanupFailures.length === 0 ? 0 : 1);
}
if (failure) {
  console.error(`Native smoke failed: ${failure.message}`);
  process.exit(1);
}
if (cleanupFailures.length > 0) process.exit(1);

const report = JSON.stringify({ platform: process.platform, ...result }, null, 2);
console.log(report);
if (evidenceDir) writeFileSync(join(evidenceDir, "smoke.json"), `${report}\n`);
process.exit(0);
