// Bounded building blocks for the native smoke. Kept apart from native-smoke.mjs, which
// launches an application on import, so that each failure mode can be tested directly.
import { execFile } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync } from "node:fs";
import { rm } from "node:fs/promises";
import { homedir, hostname, userInfo } from "node:os";
import { basename, dirname, join, resolve } from "node:path";

export const sleep = (ms) => new Promise((done) => setTimeout(done, ms));

export class TimeoutError extends Error {}

/** A wait that cannot succeed by trying again, such as a lost socket or an exited process. */
export class FatalWaitError extends Error {}

export class CdpClosedError extends FatalWaitError {}

/** Runs `work(signal)`, rejecting after `ms`. The signal aborts when the time runs out. */
export async function withTimeout(label, ms, work) {
  const controller = new AbortController();
  let timer;
  const expired = new Promise((_, fail) => {
    timer = setTimeout(() => {
      const error = new TimeoutError(`${label} timed out after ${ms} ms`);
      controller.abort(error);
      fail(error);
    }, ms);
  });
  try {
    return await Promise.race([Promise.resolve().then(() => work(controller.signal)), expired]);
  } finally {
    clearTimeout(timer);
  }
}

/**
 * Polls `probe(signal)` until it returns something truthy. A hung probe is abandoned after
 * `probeTimeoutMs`, so the overall deadline holds. `guard` runs before each attempt and throws
 * FatalWaitError to end the wait early.
 */
export async function until(
  what,
  probe,
  { timeoutMs = 45_000, probeTimeoutMs = 10_000, intervalMs = 250, guard, describe } = {},
) {
  const deadline = Date.now() + timeoutMs;
  let lastError;
  const giveUp = () => {
    const details = [lastError?.message, describe?.()].filter(Boolean).join("; ");
    return new Error(`Timed out waiting for ${what}${details ? `: ${details}` : ""}`);
  };
  while (Date.now() < deadline) {
    guard?.();
    const budget = Math.max(1, Math.min(probeTimeoutMs, deadline - Date.now()));
    try {
      const value = await withTimeout(`Probe for ${what}`, budget, probe);
      if (value) return value;
      lastError = undefined;
    } catch (error) {
      if (error instanceof FatalWaitError) throw error;
      lastError = error;
    }
    await sleep(Math.min(intervalMs, Math.max(0, deadline - Date.now())));
  }
  throw giveUp();
}

/** A JSON HTTP request whose connection, headers and body all share one deadline. */
export async function requestJson(
  url,
  { method = "GET", body, timeoutMs = 10_000, signal: outer } = {},
) {
  return withTimeout(`${method} ${url}`, timeoutMs, async (signal) => {
    try {
      const response = await fetch(url, {
        method,
        signal: outer ? AbortSignal.any([signal, outer]) : signal,
        ...(body === undefined
          ? {}
          : { headers: { "content-type": "application/json" }, body: JSON.stringify(body) }),
      });
      return { ok: response.ok, status: response.status, json: await response.json() };
    } catch (error) {
      // fetch reports every transport failure as "fetch failed"; the reason is in its cause.
      const reason = error.cause?.code ?? error.cause?.message;
      throw reason ? new Error(`${error.message}: ${reason}`, { cause: error }) : error;
    }
  });
}

export class Cdp {
  #id = 0;
  #pending = new Map();
  #closed = null;
  #socket;
  #onMessage;
  #onClose;
  #onError;
  #callTimeoutMs;
  events = [];
  lateResponses = 0;
  onEvent;

  constructor(socket, { callTimeoutMs = 15_000 } = {}) {
    this.#socket = socket;
    this.#callTimeoutMs = callTimeoutMs;
    this.#onMessage = ({ data }) => this.#receive(data);
    this.#onClose = () => this.#fail(new CdpClosedError("CDP socket closed"));
    this.#onError = () => this.#fail(new CdpClosedError("CDP socket failed"));
    socket.addEventListener("message", this.#onMessage);
    socket.addEventListener("close", this.#onClose);
    socket.addEventListener("error", this.#onError);
  }

  static async connect(url, { timeoutMs = 10_000, callTimeoutMs, WebSocketImpl = WebSocket } = {}) {
    const socket = new WebSocketImpl(url);
    let settle = () => {};
    const opening = new Promise((done, fail) => {
      const finish = (outcome) => {
        socket.removeEventListener("open", onOpen);
        socket.removeEventListener("error", onFailure);
        socket.removeEventListener("close", onFailure);
        outcome();
      };
      const onOpen = () => finish(done);
      const onFailure = () => finish(() => fail(new CdpClosedError(`CDP connect failed: ${url}`)));
      settle = (error) => finish(() => fail(error));
      socket.addEventListener("open", onOpen);
      socket.addEventListener("error", onFailure);
      socket.addEventListener("close", onFailure);
    });
    const timer = setTimeout(() => {
      settle(new TimeoutError(`CDP connect timed out after ${timeoutMs} ms: ${url}`));
      socket.close();
    }, timeoutMs);
    try {
      await opening;
    } finally {
      clearTimeout(timer);
    }
    return new Cdp(socket, { callTimeoutMs });
  }

  get pendingCount() {
    return this.#pending.size;
  }

  #receive(data) {
    let message;
    try {
      message = JSON.parse(data);
    } catch {
      this.#fail(new CdpClosedError("CDP sent a malformed message"));
      this.#socket.close();
      return;
    }
    if (message.id === undefined) {
      this.events.push(message);
      try {
        this.onEvent?.(message);
      } catch (error) {
        this.#fail(new CdpClosedError(`CDP event handler failed: ${error.message}`));
        this.#socket.close();
      }
      return;
    }
    const call = this.#pending.get(message.id);
    if (!call) {
      // A call that already timed out. Its caller has been told; there is nothing to settle.
      this.lateResponses += 1;
      return;
    }
    this.#pending.delete(message.id);
    clearTimeout(call.timer);
    if (message.error) call.fail(new Error(`${call.method}: ${message.error.message}`));
    else call.done(message.result);
  }

  #fail(error) {
    if (this.#closed) return;
    this.#closed = error;
    this.#socket.removeEventListener("message", this.#onMessage);
    this.#socket.removeEventListener("close", this.#onClose);
    this.#socket.removeEventListener("error", this.#onError);
    for (const call of this.#pending.values()) {
      clearTimeout(call.timer);
      call.fail(new CdpClosedError(`${call.method}: ${error.message}`));
    }
    this.#pending.clear();
  }

  send(method, params = {}, { timeoutMs = this.#callTimeoutMs } = {}) {
    if (this.#closed) {
      return Promise.reject(new CdpClosedError(`${method}: ${this.#closed.message}`));
    }
    const id = ++this.#id;
    return new Promise((done, fail) => {
      const timer = setTimeout(() => {
        this.#pending.delete(id);
        fail(new TimeoutError(`CDP ${method} timed out after ${timeoutMs} ms`));
      }, timeoutMs);
      this.#pending.set(id, { method, done, fail, timer });
      try {
        this.#socket.send(JSON.stringify({ id, method, params }));
      } catch (error) {
        this.#pending.delete(id);
        clearTimeout(timer);
        fail(new CdpClosedError(`${method}: ${error.message}`));
      }
    });
  }

  async evaluate(expression, options) {
    const { result, exceptionDetails } = await this.send(
      "Runtime.evaluate",
      { expression, awaitPromise: true, returnByValue: true },
      options,
    );
    if (exceptionDetails) throw new Error(exceptionDetails.exception?.description ?? "eval failed");
    return result.value;
  }

  /** Idempotent: settles pending calls, removes listeners and closes the socket. */
  close(reason = "CDP closed by the harness") {
    this.#fail(new CdpClosedError(reason));
    this.onEvent = undefined;
    this.#socket.close();
  }
}

// Tauri's local origin in a Windows release build (`protocol::origin` in tauri 2.12.5).
export const WINDOWS_APP_ORIGIN = "http://tauri.localhost";
// Elsewhere it is a custom scheme.
export const OTHER_APP_ORIGIN = "tauri://localhost";

/** Whether a page URL belongs to the local application, compared by scheme and host. */
export function isAppUrl(url, origin) {
  try {
    const parsed = new URL(url);
    return `${parsed.protocol}//${parsed.host}` === origin;
  } catch {
    return false;
  }
}

// --- WebView2 debug endpoint, scoped to one profile

export const WEBVIEW2_PROFILE_PREFIX = "maddentool-webview2-";

/**
 * A fresh WebView2 user data folder for one run. WebView2 uses WEBVIEW2_USER_DATA_FOLDER in place
 * of the folder the app passes, so the run never touches the installed profile or joins a browser
 * process that another instance already started.
 */
export function createWebView2Profile(root) {
  mkdirSync(root, { recursive: true });
  return mkdtempSync(join(resolve(root), WEBVIEW2_PROFILE_PREFIX));
}

/**
 * Removes only a folder createWebView2Profile made directly under `root`. Observed locally: the
 * browser's crash handler still held files for a moment after its process tree was stopped, so a
 * locked file is retried until `timeoutMs`, then reported.
 */
export async function removeWebView2Profile(root, profile, { timeoutMs = 10_000 } = {}) {
  const folder = resolve(profile);
  if (dirname(folder) !== resolve(root) || !basename(folder).startsWith(WEBVIEW2_PROFILE_PREFIX)) {
    throw new Error(`Refusing to remove ${folder}: not a smoke profile under ${resolve(root)}`);
  }
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    try {
      await rm(folder, { recursive: true, force: true });
      return;
    } catch (error) {
      if (!["EBUSY", "EPERM", "ENOTEMPTY"].includes(error.code) || Date.now() >= deadline) {
        throw error;
      }
      await sleep(250);
    }
  }
}

/**
 * Port 0 lets the browser choose a free port. Chromium then writes it, with its browser target
 * path, to DevToolsActivePort in the user data folder (EBWebView for WebView2). Logging goes to
 * chrome_debug.log there, which is how a debug server that never starts can be told apart.
 */
export const WEBVIEW2_DEBUG_ARGUMENTS = "--remote-debugging-port=0 --enable-logging --v=0";
export const webView2DataFolder = (profile) => join(profile, "EBWebView");

/** `<port>\n/devtools/browser/<id>`, as Chromium writes it. Anything else is not trusted. */
export function parseDevToolsActivePort(text) {
  const [portLine, path] = String(text).split(/\r?\n/);
  const port = Number(portLine);
  if (!/^\d{1,5}$/.test(portLine ?? "") || port < 1 || port > 65_535) {
    throw new Error(`DevToolsActivePort has no valid port: ${JSON.stringify(portLine ?? "")}`);
  }
  if (!/^\/devtools\/browser\/[\w-]+$/.test(path ?? "")) {
    throw new Error(`DevToolsActivePort has no browser target: ${JSON.stringify(path ?? "")}`);
  }
  return { port, browserPath: path };
}

// Chromium's order (chrome/browser/devtools/remote_debugging_server.cc): 127.0.0.1, then ::1
// when that address cannot be bound, such as when another browser holds the port.
export const DEVTOOLS_HOSTS = ["127.0.0.1", "[::1]"];

/**
 * Finds the debug server that wrote this profile's DevToolsActivePort. A server on that port whose
 * browser target differs belongs to another browser and is never used. `endpoint` records what
 * was seen for the startup diagnosis.
 */
export async function findOwnDevTools({ readPortFile, requestVersion, endpoint = {} }) {
  let text;
  try {
    text = await readPortFile();
  } catch (error) {
    endpoint.devToolsActivePort = error.code === "ENOENT" ? "absent" : "unreadable";
    throw new Error(error.code === "ENOENT" ? "DevToolsActivePort not written" : error.message, {
      cause: error,
    });
  }
  const { port, browserPath } = parseDevToolsActivePort(text);
  endpoint.devToolsActivePort = `port ${port}`;
  const answers = [];
  for (const host of DEVTOOLS_HOSTS) {
    const base = `http://${host}:${port}`;
    let found;
    try {
      found = await requestVersion(`${base}/json/version`);
    } catch (error) {
      answers.push(`${host}: ${error.message}`);
      continue;
    }
    let path;
    try {
      path = new URL(found?.webSocketDebuggerUrl).pathname;
    } catch {
      path = undefined;
    }
    if (path === browserPath) {
      endpoint.host = host;
      return { host, port, base };
    }
    answers.push(`${host}: another browser (${path ?? "no browser target"})`);
  }
  endpoint.hostsTried = answers;
  throw new Error(`No debug server for this profile on port ${port}: ${answers.join("; ")}`);
}

/** What the scoped profile shows about a debug server that never answered. Bounded and redacted. */
export function inspectWebView2Profile(profile, { lineLimit = 12, lineLength = 300 } = {}) {
  const data = webView2DataFolder(profile);
  if (!existsSync(data)) return { dataFolderCreated: false };
  let logLines = [];
  try {
    logLines = readFileSync(join(data, "chrome_debug.log"), "utf8")
      .split(/\r?\n/)
      .filter((line) => /:(ERROR|FATAL):|devtools|remote.debugging/i.test(line))
      .slice(-lineLimit)
      .map((line) => redact(line.slice(0, lineLength)));
  } catch {
    logLines = ["chrome_debug.log not readable"];
  }
  return {
    dataFolderCreated: true,
    devToolsActivePort: existsSync(join(data, "DevToolsActivePort")) ? "present" : "absent",
    logLines,
  };
}

/**
 * WebView2 lists the page as about:blank before it navigates. Fetch interception attached at
 * that point did not fire in the independent review, so only the application page qualifies.
 */
export function selectAppTarget(targets, origin = WINDOWS_APP_ORIGIN) {
  return targets.find((entry) => {
    if (entry.type !== "page" || !entry.webSocketDebuggerUrl) return false;
    try {
      return new URL(entry.url).origin === origin;
    } catch {
      return false;
    }
  });
}

/**
 * Waits for the application page, naming every URL seen if it never appears. `endpoint` keeps a
 * record of the attempts, because "never reachable" and "reachable with no page" differ.
 */
export function waitForAppTarget(listTargets, { endpoint = {}, ...options } = {}) {
  let seen = [];
  Object.assign(endpoint, { attempts: 0, answered: 0, lastError: null, targetsSeen: seen });
  return until(
    "the application page",
    async (signal) => {
      endpoint.attempts += 1;
      let targets;
      try {
        targets = await listTargets(signal);
      } catch (error) {
        endpoint.lastError = error.message;
        throw error;
      }
      endpoint.answered += 1;
      endpoint.lastError = null;
      seen = targets.map((entry) => `${entry.type} ${entry.url}`);
      endpoint.targetsSeen = seen;
      return selectAppTarget(targets);
    },
    { describe: () => `targets seen: ${JSON.stringify(seen)}`, ...options },
  );
}

/**
 * Registered core commands, called with arguments that are valid for the pinned tauri 2.12.5:
 *   set_title (window/plugin.rs): `label: Option<String>`, `value: &str`.
 *   create_webview (webview/plugin.rs): no arguments without the `unstable` feature, which this
 *   app does not enable, and `windowLabel` plus `options` with it. Valid for both.
 * The ACL check runs before arguments are read, so a granted command gets past it and answers
 * for itself: set_title resolves and create_webview reports the unstable feature.
 */
export const CORE_ACL_PROBES = [
  { name: "core_window_plugin", command: "plugin:window|set_title", args: { value: "MaddenTool" } },
  {
    name: "core_webview_plugin",
    command: "plugin:webview|create_webview",
    args: {
      windowLabel: "main",
      options: { label: "acl-probe", x: 0, y: 0, width: 1, height: 1 },
    },
  },
];

/** The same set_title call with its required argument missing. */
export const INVALID_ARGUMENTS_PROBE = {
  name: "core_window_plugin",
  command: "plugin:window|set_title",
  args: {},
};

export const UNREGISTERED_COMMAND = "read_file";

/** An expression that reports how the real webview answered, without treating it as proof. */
export const invokeExpression = (command, args = {}) =>
  `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(command)}, ${JSON.stringify(args)}).then(
    () => JSON.stringify({ outcome: "resolved" }),
    (error) => JSON.stringify({ outcome: "rejected", error: String(error) }),
  )`;

// The release-build message from `Webview::on_message` in tauri 2.12.5.
const aclDenial = (command) => `Command ${command} not allowed by ACL`;

/** Only Tauri's exact ACL denial counts. An argument or plugin error reached the command. */
export function classifyAclProbe(command, raw) {
  const answer = JSON.parse(raw);
  if (answer.outcome === "resolved") return { denied: false, kind: "resolved" };
  if (answer.error === aclDenial(command)) {
    return { denied: true, kind: "acl-denied", error: answer.error };
  }
  return { denied: false, kind: "other-rejection", error: answer.error };
}

/**
 * A command with no handler. The real app has an ACL manifest, so the ACL answers before the
 * command table does. This is not capability proof for a registered command, and the Rust test
 * that requires `Command read_file not found` is what covers registration.
 */
export function classifyUnregisteredProbe(command, raw) {
  const answer = JSON.parse(raw);
  const expected = [aclDenial(command), `Command ${command} not found`];
  if (answer.outcome === "rejected" && expected.includes(answer.error)) {
    return { rejected: true, error: answer.error };
  }
  return { rejected: false, ...answer };
}

const consoleText = (args = []) =>
  args.map((arg) => arg.value ?? arg.description ?? arg.type).join(" ");

/** Every console error, uncaught exception, unhandled rejection or CSP report in the events. */
export function consoleProblems(events) {
  const problems = [];
  for (const { method, params } of events) {
    if (method === "Log.entryAdded") {
      const { level, text, url } = params.entry;
      if (level === "error" || /content security policy/i.test(text)) {
        problems.push({ source: "log", level, text, url });
      }
    } else if (method === "Runtime.consoleAPICalled") {
      if (params.type === "error" || params.type === "assert") {
        problems.push({ source: "console", level: params.type, text: consoleText(params.args) });
      }
    } else if (method === "Runtime.exceptionThrown") {
      const { text, exception, url } = params.exceptionDetails;
      problems.push({
        source: "exception",
        level: "error",
        text: [text, exception?.description].filter(Boolean).join(": "),
        url,
      });
    }
  }
  return problems;
}

// What CDP answers for a paused request once interception is off. Expected only during teardown.
const FETCH_TEARDOWN_RACE = /Fetch domain is not enabled|Invalid InterceptionId/;

/**
 * Pauses matching requests and answers each with what `decide(params)` returns, `{ method, params }`,
 * or by continuing it when that is null. Every command is tracked, so `stop()` can drain them
 * and report failures instead of leaving a rejection nobody is listening to.
 */
export function interceptFetch(cdp, decide) {
  const inFlight = new Set();
  const failures = [];
  let stopping = false;
  let raced = 0;
  let stopped = null;

  const answer = (method, params) => {
    const work = cdp
      .send(method, params)
      .catch((error) => {
        if (stopping && FETCH_TEARDOWN_RACE.test(error.message)) raced += 1;
        else failures.push(error);
      })
      .finally(() => inFlight.delete(work));
    inFlight.add(work);
  };
  const previous = cdp.onEvent;
  const handler = (event) => {
    previous?.(event);
    if (event.method !== "Fetch.requestPaused") return;
    const { requestId } = event.params;
    let decision = null;
    if (!stopping) {
      try {
        decision = decide(event.params);
      } catch (error) {
        failures.push(error);
      }
    }
    if (decision) answer(decision.method, { requestId, ...decision.params });
    else answer("Fetch.continueRequest", { requestId });
  };
  cdp.onEvent = handler;

  const drain = async () => {
    while (inFlight.size > 0) await Promise.allSettled(inFlight);
  };

  return {
    /** Lets a wait end as soon as an answer has failed, rather than at its deadline. */
    assertHealthy() {
      if (failures.length > 0) {
        throw new FatalWaitError(`Fetch interception failed: ${failures[0].message}`);
      }
    },
    /** Idempotent: drains commands, disables Fetch, detaches, then reports any failure. */
    stop() {
      stopped ??= (async () => {
        stopping = true;
        await drain();
        let disableError;
        try {
          await cdp.send("Fetch.disable");
        } catch (error) {
          disableError = error;
        }
        await drain();
        if (cdp.onEvent === handler) cdp.onEvent = previous;
        if (disableError) failures.push(disableError);
        if (failures.length > 0) {
          throw new AggregateError(
            failures,
            `Fetch interception failed: ${failures.map((error) => error.message).join("; ")}`,
          );
        }
        return { teardownRaces: raced };
      })();
      return stopped;
    },
  };
}

/** Runs cleanup newest first, once. Every step runs and is bounded, whatever the others do. */
export class Cleanups {
  #steps = [];
  #timeoutMs;

  constructor({ timeoutMs = 15_000 } = {}) {
    this.#timeoutMs = timeoutMs;
  }

  /** @returns a function that removes the step, for work that finished normally. */
  add(name, run) {
    const step = { name, run };
    this.#steps.push(step);
    return () => {
      this.#steps = this.#steps.filter((candidate) => candidate !== step);
    };
  }

  async run() {
    const failures = [];
    while (this.#steps.length > 0) {
      const { name, run } = this.#steps.pop();
      try {
        await withTimeout(`Cleanup of ${name}`, this.#timeoutMs, run);
      } catch (error) {
        failures.push(`${name}: ${error.message}`);
      }
    }
    return failures;
  }
}

/**
 * A rejection nobody handled, an uncaught exception or a signal would otherwise end the process
 * without running `finally` blocks, leaving the application open. This fails the run after
 * cleaning up.
 */
export function cleanUpOnCrash(cleanups) {
  let crashing = false;
  const crash = async (label, reason) => {
    if (crashing) return;
    crashing = true;
    console.error(`Native smoke failed: ${label}: ${reason?.message ?? reason}`);
    for (const problem of await cleanups.run()) console.error(`Cleanup failed: ${problem}`);
    process.exit(1);
  };
  process.on("unhandledRejection", (reason) => crash("unhandled rejection", reason));
  process.on("uncaughtException", (error) => crash("uncaught exception", error));
  for (const signal of ["SIGINT", "SIGTERM"]) {
    process.on(signal, () => crash("signal", signal));
  }
}

const hasExited = (child) => child.exitCode !== null || child.signalCode !== null;

/** Removes the home folder, account name and machine name, which child output can contain. */
export function redact(text, { home = homedir(), user = safeUsername(), host = hostname() } = {}) {
  let result = String(text);
  if (home) result = result.split(home).join("<home>");
  for (const [value, label] of [
    [host, "<host>"],
    [user, "<user>"],
  ]) {
    if (value && value.length > 2) {
      const escaped = value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      result = result.replace(new RegExp(escaped, "gi"), label);
    }
  }
  return result;
}

function safeUsername() {
  try {
    return userInfo().username;
  } catch {
    return "";
  }
}

/** Keeps the last `limit` characters a child writes, redacted when read. */
export function captureOutput(child, limit = 4_096) {
  let text = "";
  const append = (chunk) => {
    text = (text + chunk).slice(-limit);
  };
  child.stdout?.on("data", append);
  child.stderr?.on("data", append);
  return () => redact(text);
}

const exitText = (code, signal) => {
  if (code === null || code === undefined) return `signal ${signal}`;
  // Windows loader and crash failures are NTSTATUS values, which are read in hexadecimal.
  return code > 0xffff ? `code ${code} (0x${code.toString(16).toUpperCase()})` : `code ${code}`;
};

/** Which of the startup failures the observations support. It states no deeper cause. */
export function classifyStartup({
  spawnError,
  exit,
  runtimeVersions,
  mainWindowTitle,
  profile,
} = {}) {
  if (spawnError) return "spawn-error";
  if (exit) return "early-exit";
  if (runtimeVersions && !runtimeVersions.some(Boolean)) return "webview2-runtime-not-detected";
  // Tauri reports a runtime it cannot use in a dialog with this title, and waits on it.
  if (mainWindowTitle === "Error") return "blocked-on-native-error-dialog";
  // No browser has used the scoped folder: the override did not apply or no browser started.
  if (profile?.dataFolderCreated === false) return "webview2-profile-not-used";
  // The browser used the folder but never reported a debug server.
  if (profile?.devToolsActivePort === "absent") return "debug-server-not-started";
  return "debug-endpoint-unreachable";
}

// Registry values are Microsoft's documented detection for the Evergreen Runtime. Process
// names only: command lines hold profile paths. Window title identifies a blocking dialog.
const windowsInspection = (pid) => `
$client = 'Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
$runtime = @{}
foreach ($key in "HKLM:\\SOFTWARE\\WOW6432Node\\$client", "HKLM:\\SOFTWARE\\$client", "HKCU:\\Software\\$client") {
  $version = (Get-ItemProperty -Path $key -Name pv -ErrorAction SilentlyContinue).pv
  $runtime[$key.Substring(0, 4) + $(if ($key -match 'WOW6432Node') { ' WOW6432Node' } else { '' })] =
    $(if ($version -and $version -ne '0.0.0.0') { $version } else { $null })
}
$process = Get-Process -Id ${pid} -ErrorAction SilentlyContinue
$all = @(Get-CimInstance Win32_Process)
$direct = @($all | Where-Object { $_.ParentProcessId -eq ${pid} })
$childIds = @($direct | ForEach-Object { $_.ProcessId })
$grandchildren = @($all | Where-Object { $childIds -contains $_.ParentProcessId } | ForEach-Object { $_.Name })
@{
  runtime = $runtime
  mainWindowTitle = $(if ($process) { $process.MainWindowTitle } else { $null })
  responding = $(if ($process) { $process.Responding } else { $null })
  childProcesses = @($direct | ForEach-Object { $_.Name })
  grandchildProcesses = $grandchildren
  overrides = @(@('WEBVIEW2_BROWSER_EXECUTABLE_FOLDER', 'WEBVIEW2_RELEASE_CHANNEL_PREFERENCE', 'WEBVIEW2_PIPE_FOR_SCRIPT_DEBUGGER') |
    Where-Object { [Environment]::GetEnvironmentVariable($_) })
} | ConvertTo-Json -Compress -Depth 3`;

function inspectWindowsProcess(pid, timeoutMs) {
  return new Promise((done) => {
    execFile(
      "powershell.exe",
      ["-NoProfile", "-NonInteractive", "-Command", windowsInspection(pid)],
      { timeout: timeoutMs, windowsHide: true },
      (error, stdout) => {
        if (error) {
          done({ inspectionError: error.killed ? "timed out" : error.message.split("\n")[0] });
          return;
        }
        try {
          done(JSON.parse(stdout));
        } catch {
          done({ inspectionError: "unreadable output" });
        }
      },
    );
  });
}

/**
 * What is known about an application that never offered a debug target, without environment
 * values or command lines. `state` comes from watchProcess and `endpoint` from the wait.
 */
export async function diagnoseStartup({
  child,
  state,
  readOutput,
  endpoint,
  profile,
  inspect = process.platform === "win32" ? inspectWindowsProcess : undefined,
  timeoutMs = 15_000,
}) {
  const running = child.pid !== undefined && !hasExited(child);
  const inspected = inspect && child.pid !== undefined ? await inspect(child.pid, timeoutMs) : {};
  const { spawnError, exit } = state.observed();
  const versions = inspected.runtime ? Object.values(inspected.runtime) : undefined;
  const {
    mainWindowTitle,
    childProcesses,
    grandchildProcesses,
    responding,
    runtime,
    overrides,
    inspectionError,
  } = inspected;
  return {
    classification: classifyStartup({
      spawnError,
      exit,
      runtimeVersions: versions,
      mainWindowTitle,
      profile,
    }),
    process: {
      running,
      ...(spawnError ? { spawnError: redact(spawnError) } : {}),
      ...(exit ? { exit } : {}),
      ...(mainWindowTitle ? { mainWindowTitle: redact(mainWindowTitle) } : {}),
      ...(responding === undefined || responding === null ? {} : { responding }),
      ...(childProcesses ? { childProcesses } : {}),
      ...(grandchildProcesses ? { grandchildProcesses } : {}),
    },
    ...(profile ? { webView2Profile: profile } : {}),
    ...(runtime ? { webView2Runtime: runtime } : {}),
    ...(overrides?.length ? { webView2OverridesSet: overrides } : {}),
    ...(inspectionError ? { inspectionError: redact(inspectionError) } : {}),
    endpoint,
    output: readOutput?.() ?? "",
  };
}

export function waitForExit(child, timeoutMs) {
  if (hasExited(child)) return Promise.resolve(true);
  return new Promise((done) => {
    const onExit = () => {
      clearTimeout(timer);
      done(true);
    };
    const timer = setTimeout(() => {
      child.off("exit", onExit);
      done(false);
    }, timeoutMs);
    child.once("exit", onExit);
  });
}

/** Tracks a spawned process. `guard` ends a wait as soon as it fails to start or exits. */
export function watchProcess(child, name) {
  let failure = null;
  let spawnError;
  let exit;
  let reject;
  // For a single long request, which no guard between attempts would interrupt.
  const failed = new Promise((_, fail) => (reject = fail));
  failed.catch(() => {});
  const fail = (error) => {
    failure ??= error;
    reject(failure);
  };
  child.once("error", (error) => {
    spawnError = error.message;
    fail(new FatalWaitError(`${name} could not start: ${error.message}`));
  });
  child.once("exit", (code, signal) => {
    exit = exitText(code, signal);
    fail(new FatalWaitError(`${name} exited early with ${exit}`));
  });
  return {
    guard: () => {
      if (failure) throw failure;
    },
    failed,
    observed: () => ({ spawnError, exit }),
  };
}

/**
 * One bounded attempt that is never repeated, for work whose retry would not be the same request,
 * such as a WebDriver session that launches the application. It ends early if `watch` fails.
 */
export function singleAttempt(what, timeoutMs, work, watch) {
  return withTimeout(what, timeoutMs, (signal) =>
    Promise.race([work(signal), ...(watch ? [watch.failed] : [])]),
  );
}

/**
 * WebKitWebDriver's GET /status (Source/WebDriver/WebDriverService.cpp), which tauri-driver
 * forwards unchanged: `ready` is true while it holds no session.
 */
export async function waitForDriverReady(status, options = {}) {
  let lastStatus;
  return until(
    "tauri-driver to accept a session",
    async (signal) => {
      const value = await status(signal);
      lastStatus = value;
      if (value?.ready === true) return value;
      throw new Error(`driver not ready: ${value?.message ?? JSON.stringify(value)}`);
    },
    {
      timeoutMs: 30_000,
      probeTimeoutMs: 5_000,
      ...options,
      // A final probe can exhaust its deadline; retain the last real driver response too.
      describe: () =>
        [
          lastStatus === undefined
            ? null
            : `last driver status: ${lastStatus?.message ?? JSON.stringify(lastStatus)}`,
          options.describe?.(),
        ]
          .filter(Boolean)
          .join("; "),
    },
  );
}

/** A W3C WebDriver call that returns `value`, or throws the driver's message. */
export const webDriverClient = (base) => async (method, path, body, timeoutMs, signal) => {
  const { ok, status, json } = await requestJson(`${base}${path}`, {
    method,
    body,
    timeoutMs,
    signal,
  });
  if (!ok) throw new Error(`${path}: ${json.value?.message ?? status}`);
  return json.value;
};

/**
 * Waits for the driver to accept a session, then asks for one exactly once. `watch` is the
 * driver's watchProcess, so its exit ends either wait at once.
 */
export async function startWebDriverSession(
  call,
  capabilities,
  { watch, readyTimeoutMs = 30_000, sessionTimeoutMs = 90_000, intervalMs } = {},
) {
  await waitForDriverReady((signal) => call("GET", "/status", undefined, 5_000, signal), {
    guard: watch?.guard,
    timeoutMs: readyTimeoutMs,
    ...(intervalMs === undefined ? {} : { intervalMs }),
  });
  const started = Date.now();
  const session = await singleAttempt(
    "Creating the WebDriver session",
    sessionTimeoutMs,
    (signal) => call("POST", "/session", { capabilities }, sessionTimeoutMs, signal),
    watch,
  );
  return { session, sessionMs: Date.now() - started };
}

const taskkill = (pid) =>
  new Promise((done) => {
    // Failure is judged by whether the process exits, not by this result.
    execFile("taskkill", ["/pid", String(pid), "/t", "/f"], { timeout: 5_000 }, () => done());
  });

/** Whether any process remains in a POSIX process group. */
function groupAlive(pgid) {
  try {
    process.kill(-pgid, 0);
    return true;
  } catch (error) {
    return error.code === "EPERM";
  }
}

function signalGroup(pgid, signal) {
  try {
    process.kill(-pgid, signal);
  } catch (error) {
    if (error.code !== "ESRCH") throw error;
  }
}

/**
 * Stops a process and its children, and throws if it is still running afterwards. With `group`
 * (POSIX, for a child spawned with `detached: true`), the whole process group is stopped, which
 * includes processes the child started and did not wait for, such as a WebDriver's browser.
 */
export async function stopProcess(child, name, { graceMs = 5_000, group = false } = {}) {
  if (child.pid === undefined) return;
  if (group && process.platform !== "win32") {
    if (!hasExited(child) || groupAlive(child.pid)) {
      signalGroup(child.pid, "SIGTERM");
      const deadline = Date.now() + graceMs;
      while (groupAlive(child.pid) && Date.now() < deadline) await sleep(50);
      if (groupAlive(child.pid)) {
        signalGroup(child.pid, "SIGKILL");
        const killDeadline = Date.now() + 2_000;
        while (groupAlive(child.pid) && Date.now() < killDeadline) await sleep(50);
      }
    }
    await waitForExit(child, 1_000);
    if (!hasExited(child) || groupAlive(child.pid)) {
      throw new Error(`${name} (process group ${child.pid}) is still running`);
    }
    return;
  }
  if (hasExited(child)) return;
  if (process.platform === "win32") await taskkill(child.pid);
  else child.kill("SIGTERM");
  if (await waitForExit(child, graceMs)) return;
  if (process.platform !== "win32") {
    child.kill("SIGKILL");
    if (await waitForExit(child, 2_000)) return;
  }
  throw new Error(`${name} (pid ${child.pid}) is still running`);
}
