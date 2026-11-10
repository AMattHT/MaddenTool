// The machine-policy helper never touches a real registry here: every test injects an in-memory
// registry that records each call, and the guards are checked before it is ever reached.
import assert from "node:assert/strict";
import { execFile, spawn, spawnSync } from "node:child_process";
import { EventEmitter } from "node:events";
import { lstatSync, mkdtempSync, readFileSync, rmSync, symlinkSync, unlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  ELEVATION_SCRIPT,
  POLICY_BASE,
  POLICY_KEY,
  POLICY_KEYS,
  POLICY_VALUE_DATA,
  POLICY_VALUE_NAME,
  REGISTRY_SCRIPT,
  assertPolicyOperation,
  contextRefusal,
  elevationRefusal,
  main,
  probeElevation,
  runNativeSmoke,
  runUnderPolicy,
  windowsRegistry,
} from "./ci-windows-webview2-policy.mjs";
import { WEBVIEW2_DEBUG_ARGUMENTS } from "./smoke-harness.mjs";

const HOSTED = {
  CI: "true",
  GITHUB_ACTIONS: "true",
  RUNNER_ENVIRONMENT: "github-hosted",
  RUNNER_OS: "Windows",
};
const ELEVATED = { integritySid: "S-1-16-12288", administrator: true };

const parentOf = (key) => key.slice(0, key.lastIndexOf("\\"));

/**
 * HKLM and HKCU as maps of key -> (value name -> data). Records every call, checks every
 * mutation against the helper's own scope rule, and can fail any named operation.
 */
function fakeRegistry({
  keys = [POLICY_BASE],
  machineValues = {},
  userValues = [],
  failOn = {},
  during = {},
} = {}) {
  const machine = new Map(keys.map((key) => [key, new Map()]));
  for (const [name, data] of Object.entries(machineValues)) {
    if (!machine.has(POLICY_KEY)) machine.set(POLICY_KEY, new Map());
    machine.get(POLICY_KEY).set(name, data);
  }
  const calls = [];
  const maybeFail = (op, key) => {
    const rule = failOn[op];
    if (rule && (rule === true || rule === key)) throw new Error(`injected ${op} failure`);
  };
  const subkeys = (key) => [...machine.keys()].filter((candidate) => parentOf(candidate) === key);
  const registry = {
    calls,
    machine,
    async inspect() {
      calls.push(["inspect"]);
      assertPolicyOperation({ op: "inspect" });
      maybeFail("inspect");
      return {
        keys: Object.fromEntries(
          [POLICY_BASE, ...POLICY_KEYS].map((key) => [key, machine.has(key)]),
        ),
        machineValues: [...(machine.get(POLICY_KEY)?.keys() ?? [])],
        userValues: [...userValues],
      };
    },
    async createKey(key) {
      calls.push(["create-key", key]);
      assertPolicyOperation({ op: "create-key", key });
      if (machine.has(key)) return { created: false };
      if (!machine.has(parentOf(key))) throw new Error("parent missing");
      if (failOn["create-key"] === `${key}:after`) {
        machine.set(key, new Map());
        throw new Error("injected create-key failure after creating");
      }
      maybeFail("create-key", key);
      machine.set(key, new Map());
      during["create-key"]?.(registry, key);
      return { created: true };
    },
    async setValue(key, name, data) {
      calls.push(["set-value", key, name, data]);
      assertPolicyOperation({ op: "set-value", key, name, data });
      maybeFail("set-value", key);
      machine.get(key).set(name, failOn.corruptData ? "--something-else" : data);
    },
    async readValue(key, name) {
      calls.push(["read-value", key, name]);
      assertPolicyOperation({ op: "read-value", key, name });
      maybeFail("read-value", key);
      const values = machine.get(key);
      if (!values?.has(name)) return { exists: false };
      return { exists: true, kind: "String", data: values.get(name) };
    },
    async deleteValue(key, name) {
      calls.push(["delete-value", key, name]);
      assertPolicyOperation({ op: "delete-value", key, name });
      maybeFail("delete-value", key);
      machine.get(key)?.delete(name);
    },
    async deleteKeyIfEmpty(key) {
      calls.push(["delete-key-if-empty", key]);
      assertPolicyOperation({ op: "delete-key-if-empty", key });
      maybeFail("delete-key-if-empty", key);
      if (!machine.has(key)) return "absent";
      if (machine.get(key).size > 0 || subkeys(key).length > 0) return "not-empty";
      machine.delete(key);
      return "deleted";
    },
  };
  return registry;
}

const snapshot = (registry) =>
  [...registry.machine].map(([key, values]) => [key, [...values]]).sort();

const mutations = (registry) =>
  registry.calls.filter(([op]) => !["inspect", "read-value"].includes(op));

/** A registry or probe that fails the test if anything reaches it. */
const untouchable = new Proxy(
  {},
  {
    get: (_, property) => {
      if (property === "then") return undefined;
      return () => assert.fail(`registry.${String(property)} was reached`);
    },
  },
);
const noProbe = () => assert.fail("the elevation probe was reached");
const noSmoke = () => assert.fail("the smoke was started");

const quiet = () => {
  const lines = [];
  return {
    lines,
    log: (line) => lines.push(String(line)),
    error: (line) => lines.push(String(line)),
  };
};

test("the policy is the documented machine override for this executable, with the smoke's flags", () => {
  assert.equal(POLICY_BASE, "SOFTWARE\\Policies\\Microsoft");
  assert.deepEqual(POLICY_KEYS, [
    "SOFTWARE\\Policies\\Microsoft\\Edge",
    "SOFTWARE\\Policies\\Microsoft\\Edge\\WebView2",
    "SOFTWARE\\Policies\\Microsoft\\Edge\\WebView2\\AdditionalBrowserArguments",
  ]);
  assert.equal(POLICY_KEY, POLICY_KEYS.at(-1));
  assert.equal(POLICY_VALUE_NAME, "maddentool.exe");
  assert.equal(POLICY_VALUE_DATA, WEBVIEW2_DEBUG_ARGUMENTS);
  assert.equal(POLICY_VALUE_DATA, "--remote-debugging-port=0 --enable-logging --v=0");
});

test("only the owned key chain, value name and flags pass the scope rule", () => {
  assertPolicyOperation({ op: "inspect" });
  for (const key of POLICY_KEYS) {
    assertPolicyOperation({ op: "create-key", key });
    assertPolicyOperation({ op: "delete-key-if-empty", key });
  }
  assertPolicyOperation({
    op: "set-value",
    key: POLICY_KEY,
    name: POLICY_VALUE_NAME,
    data: POLICY_VALUE_DATA,
  });
  const refused = [
    { op: "create-key", key: POLICY_BASE },
    { op: "create-key", key: "SOFTWARE\\Policies" },
    { op: "create-key", key: `${POLICY_KEY}\\Child` },
    { op: "create-key", key: "SOFTWARE\\Policies\\Microsoft\\Edge\\WebView2\\UserDataFolder" },
    { op: "create-key", key: POLICY_KEY.toLowerCase() },
    { op: "delete-key-if-empty", key: POLICY_BASE },
    { op: "set-value", key: POLICY_KEY, name: "*", data: POLICY_VALUE_DATA },
    { op: "set-value", key: POLICY_KEY, name: "other.exe", data: POLICY_VALUE_DATA },
    {
      op: "set-value",
      key: POLICY_KEY,
      name: "io.github.amattht.maddentool",
      data: POLICY_VALUE_DATA,
    },
    {
      op: "set-value",
      key: POLICY_KEY,
      name: POLICY_VALUE_NAME,
      data: "--remote-debugging-port=9222",
    },
    { op: "set-value", key: POLICY_KEYS[1], name: POLICY_VALUE_NAME, data: POLICY_VALUE_DATA },
    { op: "delete-value", key: POLICY_KEY, name: "*" },
    { op: "read-value", key: POLICY_KEY, name: "other.exe" },
    { op: "inspect", key: POLICY_KEY },
    { op: "delete-tree", key: POLICY_KEY },
    { op: "toString" },
    { op: "__proto__" },
    { op: "constructor", key: POLICY_KEY },
  ];
  for (const operation of refused) {
    assert.throws(() => assertPolicyOperation(operation), /outside the policy scope/, operation);
  }
});

test("anything but a GitHub-hosted Windows job is refused before the registry is reached", async () => {
  const cases = [
    [{ platform: "linux", env: HOSTED }, /Windows/],
    [{ platform: "win32", env: {} }, /GITHUB_ACTIONS/],
    [{ platform: "win32", env: { ...HOSTED, GITHUB_ACTIONS: "false" } }, /GITHUB_ACTIONS/],
    [{ platform: "win32", env: { ...HOSTED, CI: undefined } }, /CI/],
    [{ platform: "win32", env: { ...HOSTED, RUNNER_ENVIRONMENT: "self-hosted" } }, /self-hosted/],
    [
      { platform: "win32", env: { ...HOSTED, RUNNER_ENVIRONMENT: undefined } },
      /RUNNER_ENVIRONMENT/,
    ],
    [{ platform: "win32", env: { ...HOSTED, RUNNER_OS: "Linux" } }, /RUNNER_OS/],
  ];
  for (const [context, reason] of cases) {
    assert.match(contextRefusal(context) ?? "", reason, JSON.stringify(context));
    const output = quiet();
    const code = await main({
      argv: [],
      ...context,
      probe: noProbe,
      registry: untouchable,
      runSmoke: noSmoke,
      ...output,
    });
    assert.equal(code, 1);
    assert.match(output.lines.join("\n"), /Refused before any registry access/);
  }
  assert.equal(contextRefusal({ platform: "win32", env: HOSTED }), null);
});

test("a hosted context that is not actually elevated is refused before the registry", async () => {
  const cases = [
    [{ integritySid: "S-1-16-8192", administrator: false }, /Medium/],
    [{ integritySid: "S-1-16-8192", administrator: true }, /Medium/],
    [{ integritySid: "S-1-16-12288", administrator: false }, /Administrators/],
    [{ integritySid: "S-1-16-16384", administrator: true }, /System/],
    [{ integritySid: "", administrator: true }, /unknown/],
  ];
  for (const [elevation, reason] of cases) {
    assert.match(elevationRefusal(elevation) ?? "", reason);
    const output = quiet();
    const code = await main({
      argv: [],
      platform: "win32",
      env: HOSTED,
      probe: async () => elevation,
      registry: untouchable,
      runSmoke: noSmoke,
      ...output,
    });
    assert.equal(code, 1);
    assert.match(output.lines.join("\n"), /Refused before any registry access/);
  }
  const output = quiet();
  const failedProbe = async () => {
    throw new Error("whoami failed");
  };
  const code = await main({
    argv: [],
    platform: "win32",
    env: HOSTED,
    probe: failedProbe,
    registry: untouchable,
    runSmoke: noSmoke,
    ...output,
  });
  assert.equal(code, 1);
  assert.match(output.lines.join("\n"), /whoami failed/);
  assert.equal(elevationRefusal(ELEVATED), null);
});

test("the helper takes no arguments, so it cannot be pointed at another binary or flag", async () => {
  for (const argv of [["--binary", "other.exe"], ["--negative-probe", "console-error"], ["x"]]) {
    const output = quiet();
    const code = await main({
      argv,
      platform: "win32",
      env: HOSTED,
      probe: noProbe,
      registry: untouchable,
      runSmoke: noSmoke,
      ...output,
    });
    assert.equal(code, 1);
    assert.match(output.lines.join("\n"), /takes no arguments/);
  }
});

test("on this machine the real elevation probe refuses before any registry access", async (t) => {
  if (process.platform !== "win32") return t.skip("Windows only");
  const elevation = await probeElevation();
  assert.match(elevation.integritySid, /^S-1-16-\d+$/);
  assert.equal(typeof elevation.administrator, "boolean");
  // An elevated developer shell would pass the probe; the fake registry still keeps it inert.
  if (elevationRefusal(elevation) === null) return t.skip("this shell is elevated");
  const output = quiet();
  const code = await main({
    argv: [],
    platform: "win32",
    env: HOSTED,
    registry: untouchable,
    runSmoke: noSmoke,
    ...output,
  });
  assert.equal(code, 1);
  assert.match(output.lines.join("\n"), /Refused before any registry access/);
});

test("the command line refuses outside GitHub Actions without reaching the registry", async () => {
  // GitHub's variables are cleared so this also refuses when the suite itself runs in CI.
  const env = { ...process.env };
  for (const name of Object.keys(env)) {
    if (/^(CI|GITHUB_ACTIONS|RUNNER_ENVIRONMENT|RUNNER_OS)$/i.test(name)) delete env[name];
  }
  const { code, stderr } = await new Promise((done) => {
    execFile(
      process.execPath,
      ["scripts/ci-windows-webview2-policy.mjs"],
      { env, timeout: 30_000 },
      (error, _stdout, stderr) => done({ code: error?.code ?? 0, stderr }),
    );
  });
  assert.equal(code, 1);
  assert.match(stderr, /Refused before any registry access/);
});

test("the owned keys and value exist only while the smoke runs, then the prior state returns", async () => {
  const registry = fakeRegistry();
  const before = snapshot(registry);
  let seenDuringSmoke;
  const output = quiet();
  const code = await main({
    argv: [],
    platform: "win32",
    env: HOSTED,
    probe: async () => ELEVATED,
    registry,
    runSmoke: async () => {
      seenDuringSmoke = new Map(registry.machine.get(POLICY_KEY));
      return 0;
    },
    ...output,
  });
  assert.equal(code, 0, output.lines.join("\n"));
  assert.deepEqual([...seenDuringSmoke], [[POLICY_VALUE_NAME, POLICY_VALUE_DATA]]);
  assert.deepEqual(snapshot(registry), before);
  assert.deepEqual(mutations(registry), [
    ["create-key", POLICY_KEYS[0]],
    ["create-key", POLICY_KEYS[1]],
    ["create-key", POLICY_KEYS[2]],
    ["set-value", POLICY_KEY, POLICY_VALUE_NAME, POLICY_VALUE_DATA],
    ["delete-value", POLICY_KEY, POLICY_VALUE_NAME],
    ["delete-key-if-empty", POLICY_KEYS[2]],
    ["delete-key-if-empty", POLICY_KEYS[1]],
    ["delete-key-if-empty", POLICY_KEYS[0]],
  ]);
  assert.match(output.lines.join("\n"), /removed and verified/);
});

test("keys that already existed are kept; only the value is added and removed", async () => {
  const registry = fakeRegistry({ keys: [POLICY_BASE, ...POLICY_KEYS] });
  const before = snapshot(registry);
  const { exitCode, problems } = await runUnderPolicy({
    registry,
    runSmoke: async () => 0,
    log: () => {},
  });
  assert.deepEqual({ exitCode, problems }, { exitCode: 0, problems: [] });
  assert.deepEqual(snapshot(registry), before);
  assert.deepEqual(mutations(registry), [
    ["set-value", POLICY_KEY, POLICY_VALUE_NAME, POLICY_VALUE_DATA],
    ["delete-value", POLICY_KEY, POLICY_VALUE_NAME],
  ]);
});

test("an existing machine or user policy that could apply is refused, never shadowed", async () => {
  const cases = [
    { machineValues: { [POLICY_VALUE_NAME]: "--existing" } },
    { machineValues: { "MaddenTool.EXE": "--existing" } },
    { machineValues: { "*": "--existing" } },
    // Could be this process's AppUserModelID, which Microsoft checks before the executable name.
    { machineValues: { "io.github.amattht.maddentool": "--existing" } },
    { machineValues: { "": "--default-value" } },
    { userValues: [POLICY_VALUE_NAME] },
  ];
  for (const options of cases) {
    const registry = fakeRegistry({ keys: [POLICY_BASE, ...POLICY_KEYS], ...options });
    const before = snapshot(registry);
    const { exitCode, problems } = await runUnderPolicy({
      registry,
      runSmoke: noSmoke,
      log: () => {},
    });
    assert.equal(exitCode, 1, JSON.stringify(options));
    assert.match(problems.join("\n"), /existing .*policy/i);
    assert.deepEqual(mutations(registry), []);
    assert.deepEqual(snapshot(registry), before);
  }
});

test("the policy parent is never created", async () => {
  const registry = fakeRegistry({ keys: [] });
  const { exitCode, problems } = await runUnderPolicy({
    registry,
    runSmoke: noSmoke,
    log: () => {},
  });
  assert.equal(exitCode, 1);
  assert.match(problems.join("\n"), /SOFTWARE\\Policies\\Microsoft is missing/);
  assert.deepEqual(mutations(registry), []);
});

test("a failed smoke keeps its exit code and still removes the policy", async () => {
  const registry = fakeRegistry();
  const before = snapshot(registry);
  const { exitCode, problems } = await runUnderPolicy({
    registry,
    runSmoke: async () => 3,
    log: () => {},
  });
  assert.equal(exitCode, 3);
  assert.deepEqual(problems, []);
  assert.deepEqual(snapshot(registry), before);
});

test("a smoke that cannot be started fails the run and still removes the policy", async () => {
  const registry = fakeRegistry();
  const before = snapshot(registry);
  const { exitCode, problems } = await runUnderPolicy({
    registry,
    runSmoke: async () => {
      throw new Error("spawn ENOENT");
    },
    log: () => {},
  });
  assert.equal(exitCode, 1);
  assert.match(problems.join("\n"), /spawn ENOENT/);
  assert.deepEqual(snapshot(registry), before);
});

test("a failed setup never starts the smoke and removes what it created", async () => {
  const cases = [
    { "set-value": true },
    { "create-key": POLICY_KEYS[1] },
    { "create-key": `${POLICY_KEYS[1]}:after` },
    { corruptData: true },
    { "read-value": true },
  ];
  for (const failOn of cases) {
    const registry = fakeRegistry({ failOn });
    const before = snapshot(registry);
    const { exitCode, problems } = await runUnderPolicy({
      registry,
      runSmoke: noSmoke,
      log: () => {},
    });
    assert.equal(exitCode, 1, JSON.stringify(failOn));
    assert.match(problems.join("\n"), /setup/i);
    if (failOn["read-value"]) {
      // Cleanup could not confirm what it would delete, so it says so instead of guessing.
      assert.match(problems.join("\n"), /could not read/i);
    } else if (failOn.corruptData) {
      assert.match(problems.join("\n"), /changed during the run; left in place/);
      assert.ok(registry.machine.get(POLICY_KEY).has(POLICY_VALUE_NAME));
      assert.ok(!registry.calls.some(([op]) => op === "delete-value"));
    } else {
      assert.deepEqual(snapshot(registry), before, JSON.stringify(failOn));
    }
  }
});

test("a successful value write with lost confirmation is still cleaned up", async () => {
  const registry = fakeRegistry();
  const before = snapshot(registry);
  const setValue = registry.setValue;
  registry.setValue = async (...args) => {
    await setValue(...args);
    throw new Error("write confirmation lost");
  };
  const result = await runUnderPolicy({ registry, runSmoke: noSmoke, log: () => {} });
  assert.equal(result.exitCode, 1);
  assert.match(result.problems.join("\n"), /write confirmation lost/);
  assert.deepEqual(snapshot(registry), before);
});

test("a cleanup failure fails a passing smoke and is reported beside a failing one", async () => {
  const passing = fakeRegistry({ failOn: { "delete-value": true } });
  const afterPass = await runUnderPolicy({
    registry: passing,
    runSmoke: async () => 0,
    log: () => {},
  });
  assert.equal(afterPass.exitCode, 1);
  assert.match(afterPass.problems.join("\n"), /injected delete-value failure/);
  assert.match(afterPass.problems.join("\n"), /still present/);

  const failing = fakeRegistry({ failOn: { "delete-key-if-empty": POLICY_KEYS[0] } });
  const afterFail = await runUnderPolicy({
    registry: failing,
    runSmoke: async () => 4,
    log: () => {},
  });
  assert.equal(afterFail.exitCode, 4);
  assert.match(afterFail.problems.join("\n"), /injected delete-key-if-empty failure/);

  const unverifiable = fakeRegistry();
  let inspections = 0;
  const original = unverifiable.inspect;
  unverifiable.inspect = async () => {
    inspections += 1;
    if (inspections > 1) throw new Error("inspection unavailable");
    return original();
  };
  const afterUnverified = await runUnderPolicy({
    registry: unverifiable,
    runSmoke: async () => 0,
    log: () => {},
  });
  assert.equal(afterUnverified.exitCode, 1);
  assert.match(afterUnverified.problems.join("\n"), /could not verify/i);
});

test("entries someone else added during the run are left alone and reported", async () => {
  const registry = fakeRegistry({
    during: {
      "create-key": (fake, key) => {
        if (key === POLICY_KEYS[1]) fake.machine.set(`${key}\\UserDataFolder`, new Map());
      },
    },
  });
  const { exitCode, problems } = await runUnderPolicy({
    registry,
    runSmoke: async () => 0,
    log: () => {},
  });
  assert.equal(exitCode, 1);
  assert.match(problems.join("\n"), /WebView2 .*other entries/);
  assert.ok(registry.machine.has(`${POLICY_KEYS[1]}\\UserDataFolder`));
  assert.ok(!registry.machine.has(POLICY_KEYS[2]), "the owned leaf key is still removed");

  // A changed value is no longer ours to remove, even when its name matched our policy.
  const changed = fakeRegistry();
  const result = await runUnderPolicy({
    registry: changed,
    runSmoke: async () => {
      changed.machine.get(POLICY_KEY).set(POLICY_VALUE_NAME, "--changed");
      return 0;
    },
    log: () => {},
  });
  assert.equal(result.exitCode, 1);
  assert.match(result.problems.join("\n"), /changed during the run; left in place/);
  assert.equal(changed.machine.get(POLICY_KEY).get(POLICY_VALUE_NAME), "--changed");
  assert.ok(!changed.calls.some(([op]) => op === "delete-value"));
});

test("a foreign policy appearing before the write is not removed", async () => {
  const registry = fakeRegistry();
  registry.setValue = async (key, name) => {
    registry.machine.get(key).set(name, "--foreign");
    throw new Error("Value already exists");
  };
  const result = await runUnderPolicy({ registry, runSmoke: noSmoke, log: () => {} });
  assert.equal(result.exitCode, 1);
  assert.equal(registry.machine.get(POLICY_KEY).get(POLICY_VALUE_NAME), "--foreign");
  assert.match(result.problems.join("\n"), /left in place/);
  assert.ok(!registry.calls.some(([op]) => op === "delete-value"));
});

test("a key whose creation was refused is not owned by cleanup", async () => {
  const registry = fakeRegistry();
  const create = registry.createKey;
  registry.createKey = async (key) => {
    if (key === POLICY_KEYS[1]) {
      registry.machine.set(key, new Map());
      return { created: false };
    }
    return create(key);
  };
  const result = await runUnderPolicy({ registry, runSmoke: noSmoke, log: () => {} });
  assert.equal(result.exitCode, 1);
  assert.match(result.problems.join("\n"), /already exists/);
  assert.ok(registry.machine.has(POLICY_KEYS[1]));
  assert.ok(
    !registry.calls.some(([op, key]) => op === "delete-key-if-empty" && key === POLICY_KEYS[1]),
  );
});

test("the real CLI refuses ordinary context through direct and junction paths", () => {
  const root = mkdtempSync(join(tmpdir(), "maddentool-policy-entry-"));
  const link = join(root, "scripts");
  let linked = false;
  try {
    const scripts = dirname(fileURLToPath(import.meta.url));
    symlinkSync(scripts, link, process.platform === "win32" ? "junction" : "dir");
    linked = true;
    const env = Object.fromEntries(
      Object.entries(process.env).filter(([key]) => key.toUpperCase() !== "GITHUB_ACTIONS"),
    );
    env.GITHUB_ACTIONS = "false";
    for (const directory of [scripts, link]) {
      const result = spawnSync(
        process.execPath,
        [join(directory, "ci-windows-webview2-policy.mjs")],
        {
          env,
          encoding: "utf8",
          timeout: 10_000,
        },
      );
      assert.equal(result.error, undefined);
      assert.equal(result.status, 1, result.stdout + result.stderr);
      assert.match(result.stderr, /Refused before any registry access/);
    }
  } finally {
    if (linked) {
      assert.ok(lstatSync(link).isSymbolicLink());
      unlinkSync(link);
    }
    rmSync(root, { recursive: true, force: true });
  }
});

test("an unreadable registry fails before anything is written", async () => {
  const registry = fakeRegistry({ failOn: { inspect: true } });
  const { exitCode, problems } = await runUnderPolicy({
    registry,
    runSmoke: noSmoke,
    log: () => {},
  });
  assert.equal(exitCode, 1);
  assert.match(problems.join("\n"), /injected inspect failure/);
  assert.deepEqual(mutations(registry), []);
});

test("the smoke runs as the real native smoke with the policy as its argument source", async () => {
  const launches = [];
  const fakeSpawn = (command, args, options) => {
    launches.push({ command, args, options });
    const child = new EventEmitter();
    setImmediate(() => child.emit("exit", 5, null));
    return child;
  };
  assert.equal(await runNativeSmoke({ spawnImpl: fakeSpawn, log: () => {} }), 5);
  assert.equal(launches.length, 1);
  assert.equal(launches[0].command, process.execPath);
  assert.match(launches[0].args[0], /[\\/]scripts[\\/]native-smoke\.mjs$/);
  assert.deepEqual(launches[0].args.slice(1), ["--webview2-arguments", "policy"]);
  assert.equal(launches[0].options.stdio, "inherit");

  const signalled = (command, args) => {
    const child = new EventEmitter();
    setImmediate(() => child.emit("exit", null, "SIGTERM"));
    return Object.assign(child, { command, args });
  };
  assert.equal(await runNativeSmoke({ spawnImpl: signalled, log: () => {} }), 1);

  const broken = () => {
    const child = new EventEmitter();
    setImmediate(() => child.emit("error", new Error("spawn EACCES")));
    return child;
  };
  await assert.rejects(runNativeSmoke({ spawnImpl: broken, log: () => {} }), /spawn EACCES/);
});

test("the real adapter checks scope before it would start PowerShell", async () => {
  const refuseToRun = () => assert.fail("PowerShell was started");
  const registry = windowsRegistry({ run: refuseToRun });
  await assert.rejects(registry.createKey(POLICY_BASE), /outside the policy scope/);
  await assert.rejects(
    registry.setValue(POLICY_KEY, "*", POLICY_VALUE_DATA),
    /outside the policy scope/,
  );
  await assert.rejects(registry.deleteValue(POLICY_KEY, "other.exe"), /outside the policy scope/);
  await assert.rejects(registry.deleteKeyIfEmpty(POLICY_BASE), /outside the policy scope/);

  const runs = [];
  const recording = windowsRegistry({
    run: async (script, env) => {
      runs.push({ script, env });
      return { result: "absent" };
    },
  });
  await recording.deleteKeyIfEmpty(POLICY_KEY);
  assert.equal(runs[0].script, REGISTRY_SCRIPT);
  assert.deepEqual(runs[0].env, {
    MADDENTOOL_POLICY_OP: "delete-key-if-empty",
    MADDENTOOL_POLICY_KEY: POLICY_KEY,
  });
});

test("the registry script names only the owned scope, the 64-bit view and no recursive delete", () => {
  for (const key of [POLICY_BASE, ...POLICY_KEYS])
    assert.ok(REGISTRY_SCRIPT.includes(`'${key}'`), key);
  assert.ok(REGISTRY_SCRIPT.includes(`'${POLICY_VALUE_NAME}'`));
  assert.ok(REGISTRY_SCRIPT.includes(`'${POLICY_VALUE_DATA}'`));
  assert.match(REGISTRY_SCRIPT, /RegistryView\]::Registry64/);
  assert.match(REGISTRY_SCRIPT, /RegistryValueKind\]::String/);
  assert.doesNotMatch(REGISTRY_SCRIPT, /DeleteSubKeyTree|Remove-Item|reg(\.exe)? (add|delete)/i);
  assert.doesNotMatch(REGISTRY_SCRIPT, /'\*'/);
  // HKCU is only read, to refuse an override that would make the result ambiguous.
  const userLines = REGISTRY_SCRIPT.split("\n").filter((line) => /CurrentUser/.test(line));
  assert.equal(userLines.length, 1);
  assert.doesNotMatch(userLines[0], /\$true|SetValue|Create|Delete/);
});

const parsePowerShell = (script) =>
  new Promise((done, fail) => {
    const parser = `$errors = $null; $tokens = $null
[void][System.Management.Automation.Language.Parser]::ParseInput([Console]::In.ReadToEnd(), [ref]$tokens, [ref]$errors)
@($errors | ForEach-Object { $_.Message }) | ConvertTo-Json -Compress`;
    const child = spawn("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", parser], {
      windowsHide: true,
    });
    let stdout = "";
    child.stdout.on("data", (chunk) => (stdout += chunk));
    child.on("error", fail);
    child.on("exit", () => done(stdout.trim()));
    child.stdin.end(script);
  });

test("both PowerShell scripts parse", async (t) => {
  if (process.platform !== "win32") return t.skip("Windows only");
  for (const script of [REGISTRY_SCRIPT, ELEVATION_SCRIPT]) {
    const errors = await parsePowerShell(script);
    assert.ok(errors === "" || errors === "[]", errors);
  }
});

test("the read-only inspection runs against this machine's real registry", async (t) => {
  if (process.platform !== "win32") return t.skip("Windows only");
  const state = await windowsRegistry().inspect();
  assert.equal(state.keys[POLICY_BASE], true);
  for (const key of POLICY_KEYS) assert.equal(typeof state.keys[key], "boolean");
  assert.ok(Array.isArray(state.machineValues) && Array.isArray(state.userValues));
});

test("the local smoke never loads the machine-policy helper", () => {
  for (const file of ["scripts/native-smoke.mjs", "scripts/smoke-harness.mjs"]) {
    const source = readFileSync(file, "utf8");
    assert.doesNotMatch(source, /(from|import\()\s*["'][^"']*ci-windows-webview2-policy/);
    // The startup diagnosis reads policy names; nothing here may write the registry.
    assert.doesNotMatch(
      source,
      /(Set|New|Remove)-Item(Property)?|SetValue|CreateSubKey|DeleteSubKey|DeleteValue|reg(\.exe)? (add|delete)/i,
    );
  }
});
