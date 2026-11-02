import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { once } from "node:events";
import { createServer } from "node:http";
import { createServer as createTcpServer } from "node:net";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  CORE_ACL_PROBES,
  Cdp,
  CdpClosedError,
  Cleanups,
  FatalWaitError,
  TimeoutError,
  captureOutput,
  classifyAclProbe,
  classifyStartup,
  classifyUnregisteredProbe,
  consoleProblems,
  diagnoseStartup,
  interceptFetch,
  redact,
  requestJson,
  selectAppTarget,
  sleep,
  stopProcess,
  until,
  waitForAppTarget,
  watchProcess,
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
    } finally {
      await stopProcess(child, "child");
    }
  },
);
