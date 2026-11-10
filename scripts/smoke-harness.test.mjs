import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { once } from "node:events";
import { createServer } from "node:http";
import { existsSync, mkdirSync, mkdtempSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { createServer as createTcpServer } from "node:net";
import { tmpdir } from "node:os";
import { basename, dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  CORE_ACL_PROBES,
  Cdp,
  CdpClosedError,
  Cleanups,
  FatalWaitError,
  OTHER_APP_ORIGIN,
  TimeoutError,
  WEBVIEW2_ARGUMENT_SOURCES,
  WEBVIEW2_DEBUG_ARGUMENTS,
  WEBVIEW2_OBSERVED_SWITCHES,
  WEBVIEW2_PROFILE_PREFIX,
  captureOutput,
  classifyAclProbe,
  classifyStartup,
  classifyUnregisteredProbe,
  consoleProblems,
  createWebView2Profile,
  diagnoseStartup,
  findOwnDevTools,
  inspectWebView2Profile,
  interceptFetch,
  isAppUrl,
  parseDevToolsActivePort,
  redact,
  removeWebView2Profile,
  requestJson,
  selectAppTarget,
  sleep,
  startWebDriverSession,
  stopProcess,
  until,
  waitForAppTarget,
  watchProcess,
  webDriverClient,
  webView2LaunchEnvironment,
  withTimeout,
} from "./smoke-harness.mjs";

const elapsed = async (work) => {
  const start = Date.now();
  await work();
  return Date.now() - start;
};

class FakeSocket extends EventTarget {
  sent = [];
  closed = 0;
  send(text) {
    this.sent.push(JSON.parse(text));
  }
  close() {
    this.closed += 1;
  }
  reply(message) {
    this.dispatchEvent(new MessageEvent("message", { data: JSON.stringify(message) }));
  }
}

function fakeCdp(options) {
  const socket = new FakeSocket();
  return { socket, cdp: new Cdp(socket, options) };
}

/** Accepts real WebSocket upgrades. The caller decides what happens to the raw socket. */
async function webSocketServer(onSocket) {
  const server = createServer();
  server.on("upgrade", (request, socket) => {
    const accept = createHash("sha1")
      .update(`${request.headers["sec-websocket-key"]}258EAFA5-E914-47DA-95CA-C5AB0DC85B11`)
      .digest("base64");
    socket.write(
      `HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ${accept}\r\n\r\n`,
    );
    onSocket(socket);
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  return {
    url: `ws://127.0.0.1:${server.address().port}/`,
    close: () => {
      server.closeAllConnections();
      server.close();
    },
  };
}

// --- until and timeouts

test("a hung probe cannot outlast the overall deadline", async () => {
  let aborted = false;
  const probe = (signal) =>
    new Promise(() => signal.addEventListener("abort", () => (aborted = true)));
  const took = await elapsed(() =>
    assert.rejects(
      until("hung", probe, { timeoutMs: 120, probeTimeoutMs: 5_000 }),
      /Timed out.*hung/,
    ),
  );
  assert.ok(took < 1_000, `took ${took} ms`);
  assert.ok(aborted, "the hung probe was told to stop");
});

test("a hung attempt does not stop later attempts from succeeding", async () => {
  let calls = 0;
  const value = await until(
    "second try",
    () => (++calls === 1 ? new Promise(() => {}) : Promise.resolve("ok")),
    { timeoutMs: 2_000, probeTimeoutMs: 50, intervalMs: 1 },
  );
  assert.equal(value, "ok");
});

test("a fatal error ends the wait immediately", async () => {
  const took = await elapsed(() =>
    assert.rejects(
      until(
        "lost",
        () => {
          throw new CdpClosedError("socket gone");
        },
        { timeoutMs: 30_000 },
      ),
      /socket gone/,
    ),
  );
  assert.ok(took < 1_000, `took ${took} ms`);
});

test("a guard ends the wait even while the probe keeps failing", async () => {
  let guarded = 0;
  const guard = () => {
    if (++guarded > 2) throw new FatalWaitError("process exited");
  };
  await assert.rejects(
    until(
      "x",
      () => {
        throw new Error("retry");
      },
      { timeoutMs: 30_000, intervalMs: 1, guard },
    ),
    /process exited/,
  );
});

test("the last error and description are reported on timeout", async () => {
  await assert.rejects(
    until(
      "thing",
      () => {
        throw new Error("nope");
      },
      { timeoutMs: 20, intervalMs: 1, describe: () => "seen: a" },
    ),
    /thing: nope; seen: a/,
  );
});

test("withTimeout rejects and a late rejection stays handled", async () => {
  await assert.rejects(
    withTimeout("late", 10, () => sleep(30).then(() => Promise.reject(new Error("late")))),
    TimeoutError,
  );
  await sleep(40);
});

// --- HTTP

test("a server that never answers fails at the deadline", async () => {
  const server = createServer(() => {});
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  try {
    const url = `http://127.0.0.1:${server.address().port}/`;
    const took = await elapsed(() =>
      assert.rejects(requestJson(url, { timeoutMs: 100 }), TimeoutError),
    );
    assert.ok(took < 2_000, `took ${took} ms`);
  } finally {
    server.closeAllConnections();
    server.close();
  }
});

test("a body that stalls after its headers fails at the deadline", async () => {
  const server = createServer((_, response) => {
    response.writeHead(200, { "content-type": "application/json" });
    response.write("{");
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  try {
    const url = `http://127.0.0.1:${server.address().port}/`;
    await assert.rejects(requestJson(url, { timeoutMs: 100 }), TimeoutError);
  } finally {
    server.closeAllConnections();
    server.close();
  }
});

test("a refused connection reports why, not only that fetch failed", async () => {
  const server = createTcpServer();
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const { port } = server.address();
  server.close();
  await once(server, "close");
  await assert.rejects(
    requestJson(`http://127.0.0.1:${port}/json`, { timeoutMs: 2_000 }),
    /fetch failed: ECONNREFUSED/,
  );
});

// --- CDP

test("a stalled CDP call times out and clears its pending entry", async () => {
  const { cdp } = fakeCdp({ callTimeoutMs: 20 });
  await assert.rejects(cdp.send("Runtime.evaluate"), /Runtime\.evaluate timed out after 20 ms/);
  assert.equal(cdp.pendingCount, 0);
});

test("socket loss rejects every pending call at once", async () => {
  const { cdp, socket } = fakeCdp();
  const calls = [cdp.send("A"), cdp.send("B")];
  socket.dispatchEvent(new Event("close"));
  for (const call of calls) await assert.rejects(call, CdpClosedError);
  assert.equal(cdp.pendingCount, 0);
  await assert.rejects(cdp.send("C"), CdpClosedError);
});

test("a socket error rejects pending calls", async () => {
  const { cdp, socket } = fakeCdp();
  const call = cdp.send("A");
  socket.dispatchEvent(new Event("error"));
  await assert.rejects(call, /A: CDP socket failed/);
});

test("a late response is counted and settles nothing", async () => {
  const { cdp, socket } = fakeCdp({ callTimeoutMs: 10 });
  const call = cdp.send("A");
  const { id } = socket.sent[0];
  await assert.rejects(call, TimeoutError);
  socket.reply({ id, result: { late: true } });
  assert.equal(cdp.lateResponses, 1);
  assert.equal(cdp.pendingCount, 0);
});

test("a malformed message fails pending calls rather than throwing in the listener", async () => {
  const { cdp, socket } = fakeCdp();
  const call = cdp.send("A");
  socket.dispatchEvent(new MessageEvent("message", { data: "{not json" }));
  await assert.rejects(call, /malformed/);
  assert.equal(socket.closed, 1);
});

test("a throwing event handler fails pending calls", async () => {
  const { cdp, socket } = fakeCdp();
  cdp.onEvent = () => {
    throw new Error("handler bug");
  };
  const call = cdp.send("A");
  socket.reply({ method: "Some.event", params: {} });
  await assert.rejects(call, /handler bug/);
});

test("closing is idempotent and rejects pending calls", async () => {
  const { cdp, socket } = fakeCdp();
  const call = cdp.send("A");
  cdp.close();
  cdp.close();
  await assert.rejects(call, CdpClosedError);
  assert.equal(socket.closed, 2);
  assert.equal(cdp.pendingCount, 0);
});

test("a real socket that drops fails a pending call and the wait polling it", async () => {
  let peer;
  const server = await webSocketServer((socket) => (peer = socket));
  try {
    const cdp = await Cdp.connect(server.url, { timeoutMs: 2_000 });
    const pending = cdp.send("Runtime.evaluate", {}, { timeoutMs: 30_000 });
    await sleep(20);
    peer.destroy();
    const took = await elapsed(async () => {
      await assert.rejects(pending, CdpClosedError);
      await assert.rejects(
        until("page", () => cdp.evaluate("1"), { timeoutMs: 30_000 }),
        CdpClosedError,
      );
    });
    assert.ok(took < 5_000, `took ${took} ms`);
  } finally {
    server.close();
  }
});

test("connecting to a server that never finishes the handshake is bounded", async () => {
  const sockets = [];
  const server = createTcpServer((socket) => {
    sockets.push(socket);
    socket.on("error", () => {});
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  try {
    const url = `ws://127.0.0.1:${server.address().port}/`;
    await assert.rejects(Cdp.connect(url, { timeoutMs: 100 }), /CDP connect timed out/);
  } finally {
    for (const socket of sockets) socket.destroy();
    server.close();
  }
});

// --- target selection

const blank = { type: "page", url: "about:blank", webSocketDebuggerUrl: "ws://blank" };
const app = { type: "page", url: "http://tauri.localhost/", webSocketDebuggerUrl: "ws://app" };

test("only the local application page is a target", () => {
  assert.equal(selectAppTarget([blank]), undefined);
  assert.equal(selectAppTarget([blank, app]), app);
  for (const url of [
    "http://tauri.localhost.example.com/",
    "https://tauri.localhost/",
    "http://localhost:1420/",
    "not a url",
  ]) {
    assert.equal(selectAppTarget([{ ...app, url }]), undefined, url);
  }
  assert.equal(selectAppTarget([{ ...app, type: "iframe" }]), undefined);
  assert.equal(selectAppTarget([{ ...app, webSocketDebuggerUrl: undefined }]), undefined);
});

test("the wait passes an initial blank target and takes the application", async () => {
  const lists = [[blank], [blank], [blank, app]];
  const target = await waitForAppTarget(async () => lists.shift(), {
    timeoutMs: 2_000,
    intervalMs: 1,
  });
  assert.equal(target, app);
});

test("a target that stays blank times out naming what was seen", async () => {
  const endpoint = {};
  await assert.rejects(
    waitForAppTarget(async () => [blank], { timeoutMs: 40, intervalMs: 1, endpoint }),
    /targets seen: \["page about:blank"\]/,
  );
  assert.ok(endpoint.answered > 0);
  assert.equal(endpoint.lastError, null);
});

test("an endpoint that never answers is recorded differently from a blank page", async () => {
  const endpoint = {};
  await assert.rejects(
    waitForAppTarget(
      async () => {
        throw new Error("fetch failed: ECONNREFUSED");
      },
      { timeoutMs: 40, intervalMs: 1, endpoint },
    ),
    /ECONNREFUSED/,
  );
  assert.ok(endpoint.attempts > 0);
  assert.equal(endpoint.answered, 0);
  assert.equal(endpoint.lastError, "fetch failed: ECONNREFUSED");
});

// --- capability boundary

const rejected = (error) => JSON.stringify({ outcome: "rejected", error });
const windowCommand = "plugin:window|set_title";

test("only the exact ACL denial proves the boundary", () => {
  assert.deepEqual(
    classifyAclProbe(windowCommand, rejected(`Command ${windowCommand} not allowed by ACL`)),
    { denied: true, kind: "acl-denied", error: `Command ${windowCommand} not allowed by ACL` },
  );
});

for (const [name, error] of [
  ["an argument error", "invalid args `value` for command `set_title`: missing required key value"],
  ["a plugin error", "this feature requires the `unstable` flag on Cargo.toml"],
  ["a registration error", `Command ${windowCommand} not found`],
  ["another command's denial", "Command plugin:window|close not allowed by ACL"],
  ["a remote-origin refusal", `${windowCommand} not allowed. Plugin not found`],
]) {
  test(`${name} cannot count as a boundary check`, () => {
    const answer = classifyAclProbe(windowCommand, rejected(error));
    assert.equal(answer.denied, false);
    assert.equal(answer.kind, "other-rejection");
  });
}

test("a command that resolves is reported as allowed", () => {
  const raw = JSON.stringify({ outcome: "resolved" });
  assert.deepEqual(classifyAclProbe(windowCommand, raw), { denied: false, kind: "resolved" });
});

test("probes supply the arguments their commands take", () => {
  const byName = Object.fromEntries(CORE_ACL_PROBES.map((probe) => [probe.name, probe]));
  assert.equal(typeof byName.core_window_plugin.args.value, "string");
  assert.equal(typeof byName.core_webview_plugin.args.windowLabel, "string");
  assert.equal(typeof byName.core_webview_plugin.args.options.label, "string");
});

test("the unregistered command stays separate from ACL proof", () => {
  assert.equal(
    classifyUnregisteredProbe("read_file", rejected("Command read_file not found")).rejected,
    true,
  );
  assert.equal(
    classifyUnregisteredProbe("read_file", rejected("Command read_file not allowed by ACL"))
      .rejected,
    true,
  );
  assert.equal(classifyUnregisteredProbe("read_file", rejected("invalid args")).rejected, false);
  assert.equal(
    classifyUnregisteredProbe("read_file", JSON.stringify({ outcome: "resolved" })).rejected,
    false,
  );
});

// --- console evidence

test("console errors, exceptions, rejections and CSP reports all count", () => {
  const problems = consoleProblems([
    { method: "Runtime.consoleAPICalled", params: { type: "error", args: [{ value: "boom" }] } },
    {
      method: "Runtime.exceptionThrown",
      params: {
        exceptionDetails: { text: "Uncaught", exception: { description: "Error: thrown" } },
      },
    },
    {
      method: "Runtime.exceptionThrown",
      params: {
        exceptionDetails: {
          text: "Uncaught (in promise)",
          exception: { description: "Error: rejected" },
        },
      },
    },
    { method: "Log.entryAdded", params: { entry: { level: "error", text: "failed to load" } } },
    {
      method: "Log.entryAdded",
      params: { entry: { level: "warning", text: "Refused by Content Security Policy" } },
    },
  ]);
  assert.deepEqual(
    problems.map((problem) => problem.source),
    ["console", "exception", "exception", "log", "log"],
  );
  assert.match(problems[0].text, /boom/);
  assert.match(problems[2].text, /in promise.*rejected/);
});

test("ordinary console and log output is not a problem", () => {
  assert.deepEqual(
    consoleProblems([
      { method: "Runtime.consoleAPICalled", params: { type: "log", args: [{ value: "hi" }] } },
      { method: "Runtime.consoleAPICalled", params: { type: "warning", args: [] } },
      { method: "Log.entryAdded", params: { entry: { level: "info", text: "fine" } } },
      { method: "Page.loadEventFired", params: {} },
    ]),
    [],
  );
});

// --- Fetch teardown

const paused = (requestId) => ({
  method: "Fetch.requestPaused",
  params: { requestId, request: { method: "POST", url: "http://ipc.localhost/x" } },
});

/** Replays a teardown: `script(method, id)` answers each command at the moment it is sent. */
function scriptedCdp(answerFor) {
  const { cdp, socket } = fakeCdp();
  const send = socket.send.bind(socket);
  socket.send = (text) => {
    send(text);
    const { id, method } = JSON.parse(text);
    const answer = answerFor(method, id);
    if (answer) queueMicrotask(() => socket.reply({ id, ...answer }));
  };
  return { cdp, socket };
}

async function withUnhandledWatch(work) {
  const seen = [];
  const listener = (reason) => seen.push(reason);
  process.on("unhandledRejection", listener);
  try {
    await work();
    await sleep(10);
  } finally {
    process.off("unhandledRejection", listener);
  }
  assert.deepEqual(seen, [], "no rejection escaped");
}

test("a command losing the race with Fetch.disable is drained, not left unhandled", async () => {
  await withUnhandledWatch(async () => {
    const { cdp, socket } = scriptedCdp((method) =>
      method === "Fetch.disable" ? { result: {} } : null,
    );
    const interception = interceptFetch(cdp, () => null);
    socket.dispatchEvent(new MessageEvent("message", { data: JSON.stringify(paused("r1")) }));
    const continued = socket.sent.find((sent) => sent.method === "Fetch.continueRequest");
    const stopping = interception.stop();
    await sleep(5);
    assert.ok(
      !socket.sent.some((sent) => sent.method === "Fetch.disable"),
      "disable waits for commands already in flight",
    );
    socket.reply({ id: continued.id, error: { message: "Fetch domain is not enabled" } });
    assert.deepEqual(await stopping, { teardownRaces: 1 });
  });
});

test("a request paused while stopping is still answered, and its race is tolerated", async () => {
  await withUnhandledWatch(async () => {
    let socketRef;
    const { cdp, socket } = scriptedCdp((method) => {
      if (method === "Fetch.disable") {
        socketRef.dispatchEvent(
          new MessageEvent("message", { data: JSON.stringify(paused("r2")) }),
        );
        return { result: {} };
      }
      return { error: { message: "Invalid InterceptionId." } };
    });
    socketRef = socket;
    const decide = () => assert.fail("no decisions are made while stopping");
    const interception = interceptFetch(cdp, decide);
    assert.deepEqual(await interception.stop(), { teardownRaces: 1 });
    assert.ok(socket.sent.some((sent) => sent.method === "Fetch.continueRequest"));
    assert.equal(cdp.onEvent, undefined, "the handler is detached afterwards");
  });
});

test("a command failing before teardown is a failure, and it fails the wait", async () => {
  await withUnhandledWatch(async () => {
    const { cdp, socket } = scriptedCdp((method) =>
      method === "Fetch.disable"
        ? { result: {} }
        : { error: { message: "Fetch domain is not enabled" } },
    );
    const interception = interceptFetch(cdp, () => null);
    socket.dispatchEvent(new MessageEvent("message", { data: JSON.stringify(paused("r3")) }));
    await sleep(5);
    assert.throws(() => interception.assertHealthy(), FatalWaitError);
    await assert.rejects(interception.stop(), /Fetch interception failed/);
  });
});

test("an unrelated error during teardown is not mistaken for the race", async () => {
  await withUnhandledWatch(async () => {
    const { cdp, socket } = scriptedCdp((method) =>
      method === "Fetch.disable" ? { result: {} } : null,
    );
    const interception = interceptFetch(cdp, () => null);
    socket.dispatchEvent(new MessageEvent("message", { data: JSON.stringify(paused("r4")) }));
    const continued = socket.sent.find((sent) => sent.method === "Fetch.continueRequest");
    const stopping = interception.stop();
    socket.reply({ id: continued.id, error: { message: "Target crashed" } });
    await assert.rejects(stopping, /Target crashed/);
  });
});

test("socket loss during teardown is reported, not swallowed", async () => {
  await withUnhandledWatch(async () => {
    const { cdp, socket } = fakeCdp();
    const interception = interceptFetch(cdp, () => null);
    const stopping = interception.stop();
    await sleep(1);
    socket.dispatchEvent(new Event("close"));
    await assert.rejects(stopping, /CDP socket closed/);
  });
});

test("stopping twice shares one result and disables once", async () => {
  const { cdp, socket } = scriptedCdp(() => ({ result: {} }));
  const interception = interceptFetch(cdp, () => null);
  await interception.stop();
  await interception.stop();
  assert.equal(socket.sent.filter((sent) => sent.method === "Fetch.disable").length, 1);
});

test("a decision is sent with the paused request's id", async () => {
  const { cdp, socket } = scriptedCdp(() => ({ result: {} }));
  const fulfil = { method: "Fetch.fulfillRequest", params: { responseCode: 200 } };
  const interception = interceptFetch(cdp, () => fulfil);
  socket.dispatchEvent(new MessageEvent("message", { data: JSON.stringify(paused("r5")) }));
  await interception.stop();
  const sent = socket.sent.find((entry) => entry.method === "Fetch.fulfillRequest");
  assert.deepEqual(sent.params, { requestId: "r5", responseCode: 200 });
});

// --- cleanup

test("cleanup runs newest first, once, past failures and hung steps", async () => {
  const order = [];
  const cleanups = new Cleanups({ timeoutMs: 30 });
  cleanups.add("application", () => order.push("application"));
  cleanups.add("hung", () => new Promise(() => {}));
  cleanups.add("broken", () => {
    throw new Error("boom");
  });
  const removed = cleanups.add("finished", () => order.push("finished"));
  removed();
  const failures = await cleanups.run();
  assert.deepEqual(order, ["application"]);
  assert.match(failures[0], /broken: boom/);
  assert.match(failures[1], /hung: Cleanup of hung timed out/);
  assert.deepEqual(await cleanups.run(), []);
  assert.deepEqual(order, ["application"]);
});

const harnessUrl = new URL("./smoke-harness.mjs", import.meta.url).href;

for (const [name, crash, label] of [
  ["an unhandled rejection", `void Promise.reject(new Error("late Fetch failure"));`, /unhandled/],
  [
    "an uncaught exception",
    `setTimeout(() => { throw new Error("listener bug"); }, 0);`,
    /uncaught/,
  ],
]) {
  test(`${name} still cleans up and fails the run`, async () => {
    const script = `
      import { Cleanups, cleanUpOnCrash } from ${JSON.stringify(harnessUrl)};
      const cleanups = new Cleanups();
      cleanUpOnCrash(cleanups);
      cleanups.add("application", () => console.log("CLEANED"));
      ${crash}
      setTimeout(() => console.log("SURVIVED"), 2000);`;
    const child = spawn(process.execPath, ["--input-type=module", "-e", script]);
    let output = "";
    child.stdout.on("data", (chunk) => (output += chunk));
    child.stderr.on("data", (chunk) => (output += chunk));
    const [code] = await once(child, "close");
    assert.equal(code, 1, output);
    assert.match(output, /CLEANED/);
    assert.match(output, label);
    assert.doesNotMatch(output, /SURVIVED/);
  });
}

// --- processes and startup diagnostics

const idleChild = (extra = "") =>
  spawn(process.execPath, ["-e", `${extra}console.log("ready");setInterval(()=>{},1000)`], {
    stdio: ["ignore", "pipe", "pipe"],
  });

test("stopProcess ends a running process and is safe to repeat", async () => {
  const child = idleChild();
  await once(child.stdout, "data");
  await stopProcess(child, "child");
  assert.ok(child.exitCode !== null || child.signalCode !== null);
  await stopProcess(child, "child");
});

test(
  "stopProcess escalates when a process ignores the polite signal",
  { skip: process.platform === "win32" && "Windows has no SIGTERM; taskkill /f is forced" },
  async () => {
    const child = idleChild(`process.on("SIGTERM",()=>{});`);
    await once(child.stdout, "data");
    await stopProcess(child, "child", { graceMs: 100 });
    assert.equal(child.signalCode, "SIGKILL");
  },
);

test("stopProcess does nothing for a process that never started", async () => {
  const child = spawn(fileURLToPath(new URL("./no-such-binary.exe", import.meta.url)), []);
  const state = watchProcess(child, "child");
  await once(child, "error");
  await stopProcess(child, "child");
  assert.throws(state.guard, /child could not start/);
});

test("an exit is reported with its code, in hex where Windows uses NTSTATUS", async () => {
  const child = spawn(process.execPath, ["-e", "process.exit(3)"]);
  const state = watchProcess(child, "child");
  await once(child, "exit");
  assert.throws(state.guard, /child exited early with code 3/);
  assert.equal(state.observed().exit, "code 3");

  const fake = new EventTarget();
  const events = new Map();
  fake.once = (name, listener) => events.set(name, listener);
  const crashed = watchProcess(fake, "child");
  events.get("exit")(3221225781, null);
  assert.equal(crashed.observed().exit, "code 3221225781 (0xC0000135)");
});

test("each startup failure gets its own classification", () => {
  assert.equal(classifyStartup({ spawnError: "ENOENT" }), "spawn-error");
  assert.equal(classifyStartup({ exit: "code 1" }), "early-exit");
  assert.equal(
    classifyStartup({ runtimeVersions: [null, null, null] }),
    "webview2-runtime-not-detected",
  );
  assert.equal(
    classifyStartup({ runtimeVersions: ["156.0.0.0", null], mainWindowTitle: "Error" }),
    "blocked-on-native-error-dialog",
  );
  assert.equal(
    classifyStartup({ runtimeVersions: ["156.0.0.0"], mainWindowTitle: "MaddenTool" }),
    "debug-endpoint-unreachable",
  );
  // Not on Windows, or inspection failed: no claim about the runtime.
  assert.equal(classifyStartup({}), "debug-endpoint-unreachable");
});

test("output and paths are redacted and kept to a bound", async () => {
  assert.equal(
    redact("at C:\\Home\\Pat\\app and PC-1\\pat@x", {
      home: "C:\\Home\\Pat",
      user: "pat",
      host: "pc-1",
    }),
    "at <home>\\app and <host>\\<user>@x",
  );
  const child = spawn(process.execPath, [
    "-e",
    `process.stderr.write("x".repeat(9000)+"END");setInterval(()=>{},1000)`,
  ]);
  const read = captureOutput(child, 100);
  await once(child.stderr, "data");
  await sleep(20);
  await stopProcess(child, "child");
  assert.ok(read().length <= 100);
  assert.match(read(), /END$/);
});

test("the diagnosis keeps a live process's observations and no environment values", async () => {
  const child = idleChild();
  await once(child.stdout, "data");
  try {
    const diagnosis = await diagnoseStartup({
      child,
      state: watchProcess(child, "child"),
      readOutput: () => "panic text",
      endpoint: { attempts: 3, answered: 0, lastError: "fetch failed: ECONNREFUSED" },
      inspect: async () => ({
        runtime: { HKLM: null, HKCU: null },
        mainWindowTitle: "Error",
        responding: true,
        childProcesses: [],
        overrides: ["WEBVIEW2_BROWSER_EXECUTABLE_FOLDER"],
      }),
    });
    assert.equal(diagnosis.classification, "webview2-runtime-not-detected");
    assert.equal(diagnosis.process.running, true);
    assert.equal(diagnosis.process.mainWindowTitle, "Error");
    assert.deepEqual(diagnosis.webView2OverridesSet, ["WEBVIEW2_BROWSER_EXECUTABLE_FOLDER"]);
    assert.equal(diagnosis.endpoint.lastError, "fetch failed: ECONNREFUSED");
    assert.equal(diagnosis.output, "panic text");
  } finally {
    await stopProcess(child, "child");
  }
});

test(
  "the real Windows inspection reports runtime, title and children, bounded",
  { skip: process.platform !== "win32" && "Windows only" },
  async () => {
    const child = idleChild();
    await once(child.stdout, "data");
    try {
      const diagnosis = await diagnoseStartup({
        child,
        state: watchProcess(child, "child"),
        endpoint: {},
        timeoutMs: 30_000,
      });
      assert.equal(diagnosis.inspectionError, undefined);
      assert.equal(typeof diagnosis.webView2Runtime, "object");
      assert.ok(Array.isArray(diagnosis.process.childProcesses));
      assert.match(diagnosis.hostIntegrity, /^(Low|Medium|High|System)$/);
      // A plain Node child has no WebView2 browser process.
      assert.equal(diagnosis.webView2BrowserProcess, undefined);
    } finally {
      await stopProcess(child, "child");
    }
  },
);

// --- WebView2 debug endpoint scoped to the run's profile

const BROWSER_PATH = "/devtools/browser/0b1c2d3e-0000-4000-8000-000000000001";
const FOREIGN_PATH = "/devtools/browser/0b1c2d3e-0000-4000-8000-000000000002";
const notFound = () => Object.assign(new Error("ENOENT: no such file"), { code: "ENOENT" });

test("DevToolsActivePort is read only in the form Chromium writes", () => {
  assert.deepEqual(parseDevToolsActivePort(`60419\n${BROWSER_PATH}`), {
    port: 60419,
    browserPath: BROWSER_PATH,
  });
  assert.equal(parseDevToolsActivePort(`60419\r\n${BROWSER_PATH}\n`).port, 60419);
  for (const text of ["", "60419", "60419\n", `0\n${BROWSER_PATH}`, `70000\n${BROWSER_PATH}`]) {
    assert.throws(() => parseDevToolsActivePort(text), /DevToolsActivePort/, JSON.stringify(text));
  }
  assert.throws(() => parseDevToolsActivePort("60419\n/devtools/page/1"), /no browser target/);
  assert.throws(() => parseDevToolsActivePort(`-1\n${BROWSER_PATH}`), /no valid port/);
});

test("the debug server is not looked for before the profile records one", async () => {
  const endpoint = {};
  let asked = 0;
  await assert.rejects(
    findOwnDevTools({
      readPortFile: async () => {
        throw notFound();
      },
      requestVersion: async () => {
        asked += 1;
      },
      endpoint,
    }),
    /DevToolsActivePort not written/,
  );
  assert.equal(endpoint.devToolsActivePort, "absent");
  assert.equal(asked, 0);
});

test("another browser answering the port is refused, and this one is found on ::1", async () => {
  const asked = [];
  const endpoint = {};
  const found = await findOwnDevTools({
    readPortFile: async () => `9555\n${BROWSER_PATH}`,
    requestVersion: async (url) => {
      asked.push(url);
      const path = url.startsWith("http://127.0.0.1") ? FOREIGN_PATH : BROWSER_PATH;
      return { webSocketDebuggerUrl: `ws://host:9555${path}` };
    },
    endpoint,
  });
  assert.deepEqual(found, { host: "[::1]", port: 9555, base: "http://[::1]:9555" });
  assert.deepEqual(asked, ["http://127.0.0.1:9555/json/version", "http://[::1]:9555/json/version"]);
  assert.equal(endpoint.host, "[::1]");
});

test("a port owned only by other browsers is reported, never used", async () => {
  const endpoint = {};
  await assert.rejects(
    findOwnDevTools({
      readPortFile: async () => `9555\n${BROWSER_PATH}`,
      requestVersion: async (url) => {
        if (url.includes("[::1]")) throw new Error("fetch failed: ECONNREFUSED");
        return { webSocketDebuggerUrl: `ws://127.0.0.1:9555${FOREIGN_PATH}` };
      },
      endpoint,
    }),
    /No debug server for this profile on port 9555: 127\.0\.0\.1: another browser/,
  );
  assert.equal(endpoint.hostsTried.length, 2);
  assert.match(endpoint.hostsTried[1], /ECONNREFUSED/);
});

/** A real HTTP server that answers /json/version as a browser with `path` would. */
async function versionServer(path) {
  const server = createServer((request, response) => {
    response.writeHead(request.url === "/json/version" ? 200 : 404, {
      "content-type": "application/json",
    });
    const { port } = server.address();
    response.end(JSON.stringify({ webSocketDebuggerUrl: `ws://127.0.0.1:${port}${path}` }));
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  return server;
}

test("over real sockets, only the server with this profile's browser target is accepted", async () => {
  const foreign = await versionServer(FOREIGN_PATH);
  const own = await versionServer(BROWSER_PATH);
  const requestVersion = async (url) => (await requestJson(url, { timeoutMs: 2_000 })).json;
  try {
    // Nothing listens on ::1 at the foreign port, so both addresses are tried and refused.
    await assert.rejects(
      findOwnDevTools({
        readPortFile: async () => `${foreign.address().port}\n${BROWSER_PATH}`,
        requestVersion,
      }),
      /another browser/,
    );
    const found = await findOwnDevTools({
      readPortFile: async () => `${own.address().port}\n${BROWSER_PATH}`,
      requestVersion,
    });
    assert.equal(found.host, "127.0.0.1");
  } finally {
    for (const server of [foreign, own]) server.close();
  }
});

test("the WebView2 arguments choose a free port and keep a log in the profile", () => {
  assert.match(WEBVIEW2_DEBUG_ARGUMENTS, /--remote-debugging-port=0( |$)/);
  assert.match(WEBVIEW2_DEBUG_ARGUMENTS, /--enable-logging/);
});

const argumentVariables = (env) =>
  Object.entries(env).filter(
    ([name]) => name.toUpperCase() === "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
  );

test("the environment source keeps the local behaviour: the variable carries the flags", () => {
  assert.deepEqual(WEBVIEW2_ARGUMENT_SOURCES, ["environment", "policy"]);
  const env = webView2LaunchEnvironment(
    {
      PATH: "p",
      webview2_additional_browser_arguments: "--inherited",
      WEBVIEW2_USER_DATA_FOLDER: "x",
    },
    "C:\\profile",
    "environment",
  );
  assert.deepEqual(argumentVariables(env), [
    ["WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", WEBVIEW2_DEBUG_ARGUMENTS],
  ]);
  assert.equal(env.WEBVIEW2_USER_DATA_FOLDER, "C:\\profile");
  assert.equal(env.PATH, "p");
});

test("the policy source sets no arguments variable, so only the machine policy can apply", () => {
  const env = webView2LaunchEnvironment(
    {
      PATH: "p",
      WebView2_Additional_Browser_Arguments: "--inherited",
      webview2_user_data_folder: "x",
    },
    "C:\\profile",
    "policy",
  );
  assert.deepEqual(argumentVariables(env), []);
  // The run keeps its own profile, which the app's loader applies (observed on the hosted run).
  const folders = Object.entries(env).filter(([name]) => /^WEBVIEW2_USER_DATA_FOLDER$/i.test(name));
  assert.deepEqual(folders, [["WEBVIEW2_USER_DATA_FOLDER", "C:\\profile"]]);
  assert.equal(env.PATH, "p");
  assert.throws(() => webView2LaunchEnvironment({}, "C:\\profile", "registry"), /argument source/);
  assert.throws(() => webView2LaunchEnvironment({}, "C:\\profile", undefined), /argument source/);
});

test("the native smoke refuses an unknown or missing argument source before launching", async () => {
  const script = join(dirname(fileURLToPath(import.meta.url)), "native-smoke.mjs");
  const missingBinary = join(tmpdir(), "maddentool-no-such-binary.exe");
  for (const extra of [["--webview2-arguments", "registry"], ["--webview2-arguments"]]) {
    const child = spawn(process.execPath, [script, "--binary", missingBinary, ...extra], {
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stderr = "";
    child.stderr.on("data", (chunk) => (stderr += chunk));
    const [code] = await once(child, "exit");
    assert.equal(code, 1);
    assert.match(stderr, /--webview2-arguments must be one of: environment, policy/);
    assert.doesNotMatch(stderr, /binary not found/);
  }
});

const scratch = () => mkdtempSync(join(tmpdir(), "maddentool-smoke-test-"));

test("each profile is fresh, and only a smoke profile under its root is removed", async () => {
  const root = scratch();
  try {
    const profiles = join(root, "profiles");
    const first = createWebView2Profile(profiles);
    const second = createWebView2Profile(profiles);
    assert.notEqual(first, second);
    assert.equal(dirname(first), profiles);
    assert.ok(basename(first).startsWith(WEBVIEW2_PROFILE_PREFIX));
    writeFileSync(join(first, "file"), "x");

    const outsider = join(root, "keep");
    mkdirSync(outsider);
    const unprefixed = join(profiles, "keep");
    mkdirSync(unprefixed);
    await assert.rejects(removeWebView2Profile(profiles, outsider), /Refusing/);
    await assert.rejects(removeWebView2Profile(profiles, unprefixed), /Refusing/);
    await assert.rejects(removeWebView2Profile(root, first), /Refusing/);
    assert.ok(existsSync(outsider) && existsSync(unprefixed) && existsSync(first));

    await removeWebView2Profile(profiles, first);
    assert.ok(!existsSync(first));
    assert.deepEqual(readdirSync(profiles).sort(), [basename(second), "keep"].sort());
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

/** Holds a file open without delete sharing, as the WebView2 crash handler did, for `ms`. */
async function holdFile(path, ms) {
  const literal = path.replace(/'/g, "''");
  const script = [
    `$f = [System.IO.File]::Open('${literal}', 'Open', 'Read', 'None')`,
    "[Console]::Out.WriteLine('held')",
    "[Console]::Out.Flush()",
    `Start-Sleep -Milliseconds ${ms}`,
    "$f.Close()",
  ].join("; ");
  const child = spawn("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", script]);
  await once(child.stdout, "data");
  return child;
}

test(
  "a profile file held for a moment after the browser stops is still removed",
  { skip: process.platform !== "win32" && "Windows file sharing" },
  async () => {
    const root = scratch();
    const profile = createWebView2Profile(root);
    const held = join(profile, "settings.dat");
    writeFileSync(held, "x");
    const holder = await holdFile(held, 1_500);
    try {
      await removeWebView2Profile(root, profile, { timeoutMs: 10_000 });
      assert.ok(!existsSync(profile));
    } finally {
      await stopProcess(holder, "holder");
      rmSync(root, { recursive: true, force: true });
    }
  },
);

test(
  "a profile file held past the deadline fails the cleanup instead of passing silently",
  { skip: process.platform !== "win32" && "Windows file sharing" },
  async () => {
    const root = scratch();
    const profile = createWebView2Profile(root);
    const held = join(profile, "settings.dat");
    writeFileSync(held, "x");
    const holder = await holdFile(held, 20_000);
    try {
      await assert.rejects(removeWebView2Profile(root, profile, { timeoutMs: 300 }), /EBUSY|EPERM/);
    } finally {
      await stopProcess(holder, "holder");
      rmSync(root, { recursive: true, force: true });
    }
  },
);

test("the profile shows whether a browser used it and started a debug server", () => {
  const root = scratch();
  try {
    assert.deepEqual(inspectWebView2Profile(join(root, "unused")), { dataFolderCreated: false });
    const data = join(root, "used", "EBWebView");
    mkdirSync(data, { recursive: true });
    const lines = [
      "[1:2:1108/1.0:INFO:x.cc] ordinary",
      `[1:2:1108/1.1:ERROR:devtools_http_handler.cc] Cannot start http server ${"x".repeat(400)}`,
      ...Array.from({ length: 20 }, (_, index) => `[1:2:1108/2.${index}:ERROR:x.cc] e${index}`),
    ];
    writeFileSync(join(data, "chrome_debug.log"), lines.join("\n"));
    const seen = inspectWebView2Profile(join(root, "used"), { lineLimit: 3, lineLength: 50 });
    assert.equal(seen.dataFolderCreated, true);
    assert.equal(seen.devToolsActivePort, "absent");
    assert.deepEqual(
      seen.logLines.map((line) => line.slice(-3)),
      ["e17", "e18", "e19"],
    );
    const all = inspectWebView2Profile(join(root, "used"), { lineLimit: 30, lineLength: 80 });
    assert.ok(all.logLines.some((line) => /Cannot start http server/.test(line)));
    assert.ok(all.logLines.every((line) => line.length <= 80 && !/ordinary/.test(line)));
    writeFileSync(join(data, "DevToolsActivePort"), `1\n${BROWSER_PATH}`);
    assert.equal(inspectWebView2Profile(join(root, "used")).devToolsActivePort, "present");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("the profile tells an unused override from a debug server that never started", () => {
  const live = { runtimeVersions: ["155.0.4283.62"], mainWindowTitle: "MaddenTool" };
  const used = (devToolsActivePort) => ({ dataFolderCreated: true, devToolsActivePort });
  assert.equal(
    classifyStartup({ ...live, profile: { dataFolderCreated: false } }),
    "webview2-profile-not-used",
  );
  assert.equal(classifyStartup({ ...live, profile: used("absent") }), "debug-server-not-started");
  assert.equal(
    classifyStartup({ ...live, profile: used("present") }),
    "debug-endpoint-unreachable",
  );
  // A dialog or an exit still explains more than the profile does.
  assert.equal(
    classifyStartup({ ...live, mainWindowTitle: "Error", profile: { dataFolderCreated: false } }),
    "blocked-on-native-error-dialog",
  );
});

test("a missing browser log is told apart from one that cannot be read", () => {
  const root = scratch();
  try {
    const data = join(root, "used", "EBWebView");
    mkdirSync(data, { recursive: true });
    assert.deepEqual(inspectWebView2Profile(join(root, "used")).logLines, [
      "chrome_debug.log absent",
    ]);
    mkdirSync(join(data, "chrome_debug.log"));
    assert.match(
      inspectWebView2Profile(join(root, "used")).logLines[0],
      /^chrome_debug\.log unreadable \(E[A-Z]+\)$/,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("a browser started without the debug switch is classified as such", () => {
  const live = { runtimeVersions: ["156.0.4314.58"], mainWindowTitle: "MaddenTool" };
  const profile = { dataFolderCreated: true, devToolsActivePort: "absent" };
  const browser = (given) => ({
    runtimeFolder: "156.0.4314.58",
    switches: { "remote-debugging-port": given, "enable-logging": given, "user-data-dir": true },
  });
  assert.equal(
    classifyStartup({ ...live, profile, browserProcess: browser(false) }),
    "debug-arguments-not-applied",
  );
  assert.equal(
    classifyStartup({ ...live, profile, browserProcess: browser(true) }),
    "debug-server-not-started",
  );
  // Without a browser process to read, nothing is claimed about its switches.
  assert.equal(
    classifyStartup({ ...live, profile, browserProcess: null }),
    "debug-server-not-started",
  );
  // An unused profile still explains more than the switches do.
  assert.equal(
    classifyStartup({
      ...live,
      profile: { dataFolderCreated: false },
      browserProcess: browser(false),
    }),
    "webview2-profile-not-used",
  );
});

test("only switch names on an allowlist are reported, never their values", () => {
  assert.deepEqual(WEBVIEW2_OBSERVED_SWITCHES, [
    "remote-debugging-port",
    "enable-logging",
    "user-data-dir",
  ]);
});

test("the diagnosis carries integrity, browser switches and policy names", async () => {
  const child = idleChild();
  await once(child.stdout, "data");
  try {
    const browserProcess = {
      runtimeFolder: "156.0.4314.58",
      switches: { "remote-debugging-port": false, "enable-logging": false, "user-data-dir": true },
    };
    const diagnosis = await diagnoseStartup({
      child,
      state: watchProcess(child, "child"),
      endpoint: { attempts: 2, answered: 0 },
      profile: { dataFolderCreated: true, devToolsActivePort: "absent", logLines: [] },
      inspect: async () => ({
        runtime: { HKLM: "156.0.4314.58" },
        mainWindowTitle: "MaddenTool",
        childProcesses: ["msedgewebview2.exe"],
        hostIntegrity: "High",
        browserProcess,
        webView2Policies: ["HKLM:AdditionalBrowserArguments"],
      }),
      debugArguments: "policy",
    });
    assert.equal(diagnosis.classification, "debug-arguments-not-applied");
    assert.equal(diagnosis.debugArguments, "policy");
    assert.equal(diagnosis.hostIntegrity, "High");
    assert.deepEqual(diagnosis.webView2BrowserProcess, browserProcess);
    assert.deepEqual(diagnosis.webView2Policies, ["HKLM:AdditionalBrowserArguments"]);
  } finally {
    await stopProcess(child, "child");
  }
});

test("the diagnosis carries the profile's observations", async () => {
  const child = idleChild();
  await once(child.stdout, "data");
  try {
    const diagnosis = await diagnoseStartup({
      child,
      state: watchProcess(child, "child"),
      endpoint: { attempts: 2, answered: 0, devTools: { devToolsActivePort: "absent" } },
      profile: { dataFolderCreated: true, devToolsActivePort: "absent", logLines: [] },
      inspect: async () => ({
        runtime: { HKLM: "155.0.4283.62" },
        mainWindowTitle: "MaddenTool",
        childProcesses: ["msedgewebview2.exe"],
        grandchildProcesses: ["msedgewebview2.exe"],
      }),
    });
    assert.equal(diagnosis.classification, "debug-server-not-started");
    assert.deepEqual(diagnosis.process.grandchildProcesses, ["msedgewebview2.exe"]);
    assert.equal(diagnosis.webView2Profile.devToolsActivePort, "absent");
    assert.equal(diagnosis.endpoint.devTools.devToolsActivePort, "absent");
  } finally {
    await stopProcess(child, "child");
  }
});

test("application URLs are compared by scheme and host", () => {
  assert.ok(isAppUrl("tauri://localhost/", OTHER_APP_ORIGIN));
  assert.ok(isAppUrl("tauri://localhost/index.html", OTHER_APP_ORIGIN));
  for (const url of [
    "about:blank",
    "tauri://localhost.evil/",
    "http://localhost/",
    "",
    undefined,
  ]) {
    assert.equal(isAppUrl(url, OTHER_APP_ORIGIN), false, String(url));
  }
});

// --- Linux: tauri-driver session, against a driver stub over real HTTP

/**
 * Answers like tauri-driver in front of WebKitWebDriver: GET /status and POST /session.
 * A `sessionDelayMs` of null never answers the session.
 */
async function driverStub({
  readyAfter = 0,
  sessionDelayMs = 0,
  stallStatusAfter = Infinity,
} = {}) {
  const seen = { status: 0, sessions: 0, aborted: 0 };
  const server = createServer((request, response) => {
    const reply = (value, code = 200) => {
      response.writeHead(code, { "content-type": "application/json" });
      response.end(JSON.stringify({ value }));
    };
    if (request.method === "GET" && request.url === "/status") {
      seen.status += 1;
      if (seen.status > stallStatusAfter) return;
      const ready = seen.status > readyAfter;
      reply({ ready, message: ready ? "No sessions" : "A session already exists" });
      return;
    }
    if (request.method === "POST" && request.url === "/session") {
      seen.sessions += 1;
      request.resume();
      response.on("close", () => {
        if (!response.writableFinished) seen.aborted += 1;
      });
      if (sessionDelayMs === null) return;
      setTimeout(() => reply({ sessionId: `s${seen.sessions}`, capabilities: {} }), sessionDelayMs);
      return;
    }
    reply({ error: "unknown command", message: "unknown" }, 404);
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  return {
    seen,
    call: webDriverClient(`http://127.0.0.1:${server.address().port}`),
    close: () => {
      server.closeAllConnections();
      server.close();
    },
  };
}

const CAPABILITIES = { alwaysMatch: { browserName: "wry" } };

test("a slow session is created once, after the driver reports ready", async () => {
  const stub = await driverStub({ readyAfter: 2, sessionDelayMs: 300 });
  try {
    const { session, sessionMs } = await startWebDriverSession(stub.call, CAPABILITIES, {
      sessionTimeoutMs: 5_000,
      intervalMs: 1,
    });
    assert.equal(session.sessionId, "s1");
    assert.ok(sessionMs >= 250, `took ${sessionMs} ms`);
    assert.equal(stub.seen.sessions, 1);
    assert.equal(stub.seen.status, 3);
  } finally {
    stub.close();
  }
});

test("the earlier per-attempt limit relaunched a slow session instead of waiting", async () => {
  // The shape of the harness on 2fc171f: POST /session polled with a probe limit shorter than
  // the session takes. Every attempt is a new launch, so none can finish.
  const stub = await driverStub({ sessionDelayMs: 300 });
  try {
    await assert.rejects(
      until("tauri-driver", (signal) => stub.call("POST", "/session", {}, 10_000, signal), {
        timeoutMs: 500,
        probeTimeoutMs: 200,
        intervalMs: 1,
      }),
      /Timed out waiting for tauri-driver: Probe for tauri-driver timed out/,
    );
    assert.ok(stub.seen.sessions >= 2, `${stub.seen.sessions} sessions requested`);
  } finally {
    stub.close();
  }
});

test("a session that never comes fails at its deadline, with one request, aborted", async () => {
  const stub = await driverStub({ sessionDelayMs: null });
  try {
    const took = await elapsed(() =>
      assert.rejects(
        startWebDriverSession(stub.call, CAPABILITIES, { sessionTimeoutMs: 200 }),
        /Creating the WebDriver session timed out after 200 ms/,
      ),
    );
    assert.ok(took < 2_000, `took ${took} ms`);
    await sleep(100);
    assert.equal(stub.seen.sessions, 1);
    assert.equal(stub.seen.aborted, 1);
  } finally {
    stub.close();
  }
});

test("a driver that exits ends the session wait at once", async () => {
  const stub = await driverStub({ sessionDelayMs: null });
  const driver = spawn(process.execPath, ["-e", "setTimeout(() => process.exit(4), 150)"]);
  try {
    const took = await elapsed(() =>
      assert.rejects(
        startWebDriverSession(stub.call, CAPABILITIES, {
          watch: watchProcess(driver, "tauri-driver"),
          sessionTimeoutMs: 20_000,
        }),
        /tauri-driver exited early with code 4/,
      ),
    );
    assert.ok(took < 5_000, `took ${took} ms`);
  } finally {
    stub.close();
  }
});

test("a driver that never becomes ready is reported with its message", async () => {
  const stub = await driverStub({ readyAfter: Infinity });
  try {
    await assert.rejects(
      startWebDriverSession(stub.call, CAPABILITIES, { readyTimeoutMs: 100, intervalMs: 1 }),
      (error) => {
        assert.match(error.message, /Timed out waiting for tauri-driver to accept a session/);
        assert.match(error.message, /A session already exists/);
        return true;
      },
    );
    assert.equal(stub.seen.sessions, 0);
  } finally {
    stub.close();
  }
});

test("a later stalled readiness probe keeps the last driver status", async () => {
  const stub = await driverStub({ readyAfter: Infinity, stallStatusAfter: 1 });
  try {
    await assert.rejects(
      startWebDriverSession(stub.call, CAPABILITIES, {
        readyTimeoutMs: 1_000,
        intervalMs: 1,
      }),
      /last driver status: A session already exists/,
    );
    assert.equal(stub.seen.status, 2);
    assert.equal(stub.seen.sessions, 0);
  } finally {
    stub.close();
  }
});

test(
  "stopping a driver's process group also stops what it launched",
  { skip: process.platform === "win32" && "POSIX process groups" },
  async () => {
    // The leader starts a long-lived grandchild, as WebKitWebDriver starts the application.
    const grandchildCode = "process.on('SIGTERM', () => {}); setInterval(() => {}, 1000)";
    const leaderCode = [
      'const { spawn } = require("node:child_process");',
      `const child = spawn(process.execPath, ["-e", ${JSON.stringify(grandchildCode)}], { stdio: "ignore" });`,
      "console.log(child.pid);",
      "setInterval(() => {}, 1000);",
    ].join("\n");
    const leader = spawn(process.execPath, ["-e", leaderCode], {
      detached: true,
      stdio: ["ignore", "pipe", "ignore"],
    });
    try {
      const [line] = await once(leader.stdout, "data");
      const grandchild = Number(String(line).trim());
      const alive = (pid) => {
        try {
          process.kill(pid, 0);
          return true;
        } catch {
          return false;
        }
      };
      assert.ok(alive(grandchild));
      await stopProcess(leader, "driver", { group: true, graceMs: 300 });
      assert.ok(leader.exitCode !== null || leader.signalCode !== null);
      assert.equal(alive(grandchild), false);
    } finally {
      // Independent of stopProcess: a failing assertion or cleanup check must not orphan
      // the test's detached group on the hosted POSIX runner.
      if (leader.pid !== undefined) {
        try {
          process.kill(-leader.pid, "SIGKILL");
        } catch (error) {
          assert.equal(error.code, "ESRCH", "test process-group cleanup failed");
        }
      }
    }
  },
);
