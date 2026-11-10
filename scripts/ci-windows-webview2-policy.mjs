#!/usr/bin/env node
// Hosted CI only: runs the Windows native smoke with the browser's debug flags supplied by the
// machine policy, the one override source Microsoft documents an elevated WebView2 host honours.
//
//   node scripts/ci-windows-webview2-policy.mjs      (the Windows CI smoke step; no arguments)
//
// GitHub documents its Windows runners as administrators with UAC disabled, so the application
// runs elevated there and ignores WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS. This helper:
//   1. refuses anything but a GitHub-hosted Windows job whose process is actually elevated,
//      before any registry access, and never asks for elevation itself;
//   2. refuses if any value already exists in either hive's AdditionalBrowserArguments key,
//      because the AppUserModelID, executable-name or "*" entry could apply instead of ours;
//   3. creates only the missing keys below SOFTWARE\Policies\Microsoft and one REG_SZ value,
//      "maddentool.exe" = WEBVIEW2_DEBUG_ARGUMENTS, in HKLM (a shared key under WOW64);
//   4. runs the real native smoke with no arguments variable, then in `finally` removes the value
//      and every key it created that is still empty, verifies that, and keeps the smoke's exit code.
// Local `npm run smoke:native` never loads this file and never touches the registry.
import { execFile, spawn } from "node:child_process";
import { realpathSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { WEBVIEW2_DEBUG_ARGUMENTS } from "./smoke-harness.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const NATIVE_SMOKE = join(ROOT, "scripts", "native-smoke.mjs");

/** Must already exist; nothing above the Edge key is ever created. */
export const POLICY_BASE = "SOFTWARE\\Policies\\Microsoft";
/** The keys this helper may create and later remove when empty, shallowest first. */
export const POLICY_KEYS = [
  `${POLICY_BASE}\\Edge`,
  `${POLICY_BASE}\\Edge\\WebView2`,
  `${POLICY_BASE}\\Edge\\WebView2\\AdditionalBrowserArguments`,
];
export const POLICY_KEY = POLICY_KEYS.at(-1);
/** Microsoft's executable-name fallback; existing policies are refused before writing it. */
export const POLICY_VALUE_NAME = "maddentool.exe";
export const POLICY_VALUE_DATA = WEBVIEW2_DEBUG_ARGUMENTS;

const SAFE_LITERAL = /^[\w .=\\-]+$/;
for (const literal of [POLICY_BASE, ...POLICY_KEYS, POLICY_VALUE_NAME, POLICY_VALUE_DATA]) {
  if (!SAFE_LITERAL.test(literal)) throw new Error(`Unsafe policy literal: ${literal}`);
}

/** Throws for any registry operation outside the owned key chain, value name and flags. */
export function assertPolicyOperation({ op, key, name, data } = {}) {
  const rules = {
    inspect: key === undefined && name === undefined && data === undefined,
    "create-key": POLICY_KEYS.includes(key) && name === undefined,
    "delete-key-if-empty": POLICY_KEYS.includes(key) && name === undefined,
    "set-value": key === POLICY_KEY && name === POLICY_VALUE_NAME && data === POLICY_VALUE_DATA,
    "read-value": key === POLICY_KEY && name === POLICY_VALUE_NAME && data === undefined,
    "delete-value": key === POLICY_KEY && name === POLICY_VALUE_NAME && data === undefined,
  };
  // Own keys only: an inherited name such as "toString" is not an operation.
  if (!Object.hasOwn(rules, op) || rules[op] !== true) {
    throw new Error(
      `Registry operation outside the policy scope: ${JSON.stringify({ op, key, name })}`,
    );
  }
}

/** Why this process must not touch the machine policy, from its context alone; null if it may. */
export function contextRefusal({ platform, env }) {
  if (platform !== "win32") return `this helper runs on Windows only, not ${platform}`;
  if (env.GITHUB_ACTIONS !== "true") return "GITHUB_ACTIONS is not true: not a GitHub Actions job";
  if (env.CI !== "true") return "CI is not true";
  if (env.RUNNER_ENVIRONMENT !== "github-hosted") {
    return `RUNNER_ENVIRONMENT is ${env.RUNNER_ENVIRONMENT ?? "unset"}, not github-hosted: self-hosted and local machines are never changed`;
  }
  if (env.RUNNER_OS !== "Windows") return `RUNNER_OS is ${env.RUNNER_OS ?? "unset"}, not Windows`;
  return null;
}

const INTEGRITY = {
  "S-1-16-4096": "Low",
  "S-1-16-8192": "Medium",
  "S-1-16-12288": "High",
  "S-1-16-16384": "System",
};

/** Why the process's real token rules the machine policy out; null when it is elevated. */
export function elevationRefusal({ integritySid, administrator } = {}) {
  const level = INTEGRITY[integritySid] ?? "unknown";
  // WebView2 cannot run as System, and anything below High could not write HKLM anyway.
  if (level !== "High") {
    return `process integrity is ${level} (${integritySid || "no label"}), not High`;
  }
  if (administrator !== true) return "process is not in the Administrators role";
  return null;
}

// Read-only. The integrity label comes from the same whoami output the startup diagnosis uses.
export const ELEVATION_SCRIPT = `$ErrorActionPreference = 'Stop'
$sid = [regex]::Match((@(& "$env:SystemRoot\\System32\\whoami.exe" /groups) -join ' '), 'S-1-16-\\d+').Value
$principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
@{ integritySid = $sid; administrator = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator) } | ConvertTo-Json -Compress`;

// Every name is a literal: the only inputs are the operation and a key the script itself checks
// against the owned chain. DeleteSubKey without a tree, and only after the key is seen empty.
export const REGISTRY_SCRIPT = `$ErrorActionPreference = 'Stop'
$base = '${POLICY_BASE}'
$keys = @(${POLICY_KEYS.map((key) => `'${key}'`).join(", ")})
$leaf = $keys[-1]
$name = '${POLICY_VALUE_NAME}'
$data = '${POLICY_VALUE_DATA}'
$op = $env:MADDENTOOL_POLICY_OP
$key = $env:MADDENTOOL_POLICY_KEY
$view = [Microsoft.Win32.RegistryView]::Registry64
$machine = [Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::LocalMachine, $view)
function Test-Key([string]$path) { $found = $machine.OpenSubKey($path); if ($found) { $found.Dispose(); $true } else { $false } }
function Split-Key([string]$path) { $at = $path.LastIndexOf('\\'); @($path.Substring(0, $at), $path.Substring($at + 1)) }
if ($op -ne 'inspect' -and $keys -cnotcontains $key) { throw 'Key outside the policy scope' }
switch ($op) {
  'inspect' {
    $present = [ordered]@{}
    foreach ($path in @($base) + $keys) { $present[$path] = Test-Key $path }
    $policy = $machine.OpenSubKey($leaf)
    $user = [Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::CurrentUser, $view).OpenSubKey($leaf)
    $out = [ordered]@{ keys = $present; machineValues = @(if ($policy) { $policy.GetValueNames() }); userValues = @(if ($user) { $user.GetValueNames() }) }
  }
  'create-key' {
    $parentPath, $child = Split-Key $key
    $parent = $machine.OpenSubKey($parentPath, $true)
    if (-not $parent) { throw 'Parent key is missing' }
    if ($parent.GetSubKeyNames() -contains $child) { $out = @{ created = $false } }
    else {
      $parent.CreateSubKey($child).Dispose()
      $out = @{ created = $true }
    }
  }
  'set-value' {
    if ($key -cne $leaf) { throw 'Values are only set in the policy key' }
    $policy = $machine.OpenSubKey($leaf, $true)
    if (-not $policy) { throw 'Policy key is missing' }
    if ($policy.GetValueNames() -contains $name) { throw 'Value already exists' }
    $policy.SetValue($name, $data, [Microsoft.Win32.RegistryValueKind]::String)
    $out = @{ set = $true }
  }
  'read-value' {
    $policy = $machine.OpenSubKey($leaf)
    if ($policy -and ($policy.GetValueNames() -contains $name)) {
      $out = @{ exists = $true; kind = [string]$policy.GetValueKind($name); data = [string]$policy.GetValue($name, $null, [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames) }
    } else { $out = @{ exists = $false } }
  }
  'delete-value' {
    $policy = $machine.OpenSubKey($leaf, $true)
    if ($policy) { $policy.DeleteValue($name, $false) }
    $out = @{ deleted = $true }
  }
  'delete-key-if-empty' {
    $target = $machine.OpenSubKey($key)
    if (-not $target) { $out = @{ result = 'absent' } }
    elseif ($target.ValueCount -gt 0 -or $target.SubKeyCount -gt 0) { $target.Dispose(); $out = @{ result = 'not-empty' } }
    else {
      $target.Dispose()
      $parentPath, $child = Split-Key $key
      $machine.OpenSubKey($parentPath, $true).DeleteSubKey($child, $true)
      $out = @{ result = 'deleted' }
    }
  }
  default { throw 'Unknown registry operation' }
}
$out | ConvertTo-Json -Compress -Depth 4`;

const POWERSHELL = join(
  process.env.SystemRoot ?? "C:\\Windows",
  "System32",
  "WindowsPowerShell",
  "v1.0",
  "powershell.exe",
);

/** Runs a fixed script with extra environment variables and returns its JSON answer. */
function runPowerShell(script, extraEnv = {}, { timeoutMs = 30_000 } = {}) {
  return new Promise((done, fail) => {
    execFile(
      POWERSHELL,
      ["-NoProfile", "-NonInteractive", "-Command", script],
      { env: { ...process.env, ...extraEnv }, timeout: timeoutMs, windowsHide: true },
      (error, stdout, stderr) => {
        if (error) {
          const detail = (stderr || error.message).trim().split(/\r?\n/)[0];
          fail(new Error(error.killed ? "PowerShell timed out" : detail));
          return;
        }
        try {
          done(JSON.parse(stdout));
        } catch {
          fail(new Error(`PowerShell answer unreadable: ${stdout.trim().slice(0, 200)}`));
        }
      },
    );
  });
}

export const probeElevation = () => runPowerShell(ELEVATION_SCRIPT);

/** The real HKLM adapter. Every call is scope-checked here and again inside the script. */
export function windowsRegistry({ run = runPowerShell } = {}) {
  const call = async (operation) => {
    assertPolicyOperation(operation);
    const env = { MADDENTOOL_POLICY_OP: operation.op };
    if (operation.key !== undefined) env.MADDENTOOL_POLICY_KEY = operation.key;
    return run(REGISTRY_SCRIPT, env);
  };
  const list = (value) => (Array.isArray(value) ? value : value ? [value] : []);
  return {
    async inspect() {
      const state = await call({ op: "inspect" });
      return {
        keys: state.keys,
        machineValues: list(state.machineValues),
        userValues: list(state.userValues),
      };
    },
    createKey: (key) => call({ op: "create-key", key }),
    setValue: (key, name, data) => call({ op: "set-value", key, name, data }),
    readValue: (key, name) => call({ op: "read-value", key, name }),
    deleteValue: (key, name) => call({ op: "delete-value", key, name }),
    deleteKeyIfEmpty: async (key) => (await call({ op: "delete-key-if-empty", key })).result,
  };
}

const shown = (names) =>
  names
    .slice(0, 8)
    .map((name) => JSON.stringify(String(name).slice(0, 80)))
    .join(", ") + (names.length > 8 ? `, and ${names.length - 8} more` : "");

/** Why the starting state cannot be owned safely; null if it can. Reads names, never data. */
function preStateRefusal(before) {
  if (!before.keys?.[POLICY_BASE]) return `HKLM\\${POLICY_BASE} is missing; it is never created`;
  if (before.machineValues.length > 0) {
    return `an existing machine policy in HKLM\\${POLICY_KEY} could apply to this app (AppUserModelID, executable name or "*"): ${shown(before.machineValues)}. It is not shadowed or changed.`;
  }
  if (before.userValues.length > 0) {
    return `an existing user policy in HKCU\\${POLICY_KEY} would make the result ambiguous: ${shown(before.userValues)}. It is not changed.`;
  }
  return null;
}

/** Removes the value and the keys this run created, then checks the registry agrees. */
async function removeOwned(registry, before, attemptedKeys, valueAttempted) {
  const problems = [];
  if (valueAttempted) {
    try {
      // A failed write may have raced with another writer. Delete only the exact value we wrote.
      const current = await registry.readValue(POLICY_KEY, POLICY_VALUE_NAME);
      if (current.exists && (current.kind !== "String" || current.data !== POLICY_VALUE_DATA)) {
        problems.push(
          `HKLM\\${POLICY_KEY} "${POLICY_VALUE_NAME}" changed during the run; left in place`,
        );
      } else if (current.exists) {
        await registry.deleteValue(POLICY_KEY, POLICY_VALUE_NAME);
      }
    } catch (error) {
      problems.push(`Could not read or remove the policy value: ${error.message}`);
    }
  }
  const leftByChoice = new Set();
  for (const key of [...attemptedKeys].reverse()) {
    try {
      if ((await registry.deleteKeyIfEmpty(key)) === "not-empty") {
        leftByChoice.add(key);
        problems.push(
          `HKLM\\${key} was created for this run but now holds other entries; left in place`,
        );
      }
    } catch (error) {
      problems.push(`Could not remove HKLM\\${key}: ${error.message}`);
    }
  }
  try {
    const after = await registry.inspect();
    const name = POLICY_VALUE_NAME.toLowerCase();
    if (after.machineValues.some((value) => value.toLowerCase() === name)) {
      problems.push(`HKLM\\${POLICY_KEY} "${POLICY_VALUE_NAME}" is still present after cleanup`);
    }
    for (const key of attemptedKeys) {
      if (after.keys[key] && !leftByChoice.has(key)) {
        problems.push(`HKLM\\${key} is still present after cleanup`);
      }
    }
    for (const key of [POLICY_BASE, ...POLICY_KEYS]) {
      if (before.keys[key] && !after.keys[key]) {
        problems.push(`HKLM\\${key} existed before and is now missing`);
      }
    }
  } catch (error) {
    problems.push(`Could not verify the cleanup: ${error.message}`);
  }
  return problems;
}

/**
 * Owns the policy for exactly one smoke. Returns the smoke's own exit code when it failed, 1 for a
 * setup or cleanup problem, and 0 only when the smoke passed and cleanup was verified.
 */
export async function runUnderPolicy({ registry, runSmoke, log = console.log }) {
  let before;
  try {
    before = await registry.inspect();
  } catch (error) {
    return { exitCode: 1, problems: [`Could not read the policy state: ${error.message}`] };
  }
  const refusal = preStateRefusal(before);
  if (refusal) return { exitCode: 1, problems: [`Refused before writing: ${refusal}`] };

  const problems = [];
  const attemptedKeys = [];
  let valueAttempted = false;
  let smokeExit;
  try {
    for (const key of POLICY_KEYS) {
      if (before.keys[key]) continue;
      attemptedKeys.push(key);
      const result = await registry.createKey(key);
      if (result?.created === false) {
        attemptedKeys.pop();
        throw new Error(`HKLM\\${key} already exists; left in place`);
      }
    }
    valueAttempted = true;
    await registry.setValue(POLICY_KEY, POLICY_VALUE_NAME, POLICY_VALUE_DATA);
    const written = await registry.readValue(POLICY_KEY, POLICY_VALUE_NAME);
    if (!written.exists || written.kind !== "String" || written.data !== POLICY_VALUE_DATA) {
      throw new Error("the policy value did not read back as written");
    }
    log(
      `WebView2 policy set for this run: HKLM\\${POLICY_KEY} "${POLICY_VALUE_NAME}" = "${POLICY_VALUE_DATA}"` +
        ` (created: ${attemptedKeys.map((key) => key.split("\\").at(-1)).join(", ") || "none"})`,
    );
    smokeExit = await runSmoke();
  } catch (error) {
    problems.push(`Policy setup or smoke launch failed: ${error.message}`);
  } finally {
    const cleanup = await removeOwned(registry, before, attemptedKeys, valueAttempted);
    if (cleanup.length === 0) log("WebView2 policy removed and verified; the prior state is back.");
    problems.push(...cleanup);
  }
  const exitCode = smokeExit ? smokeExit : problems.length > 0 || smokeExit !== 0 ? 1 : 0;
  return { exitCode, problems };
}

/** The real smoke, from the same freshly built binary, with the policy as its argument source. */
export function runNativeSmoke({ spawnImpl = spawn, log = console.log } = {}) {
  return new Promise((done, fail) => {
    // A console interrupt also reaches the smoke, which cleans up and exits; staying alive until
    // then lets `finally` remove the policy. A forced kill cannot, and ends with the hosted VM.
    const signals = process.platform === "win32" ? ["SIGINT", "SIGBREAK"] : ["SIGINT", "SIGTERM"];
    const wait = (signal) =>
      log(`${signal}: waiting for the smoke to stop before removing the policy`);
    const release = () => signals.forEach((signal) => process.off(signal, wait));
    signals.forEach((signal) => process.on(signal, wait));
    const child = spawnImpl(process.execPath, [NATIVE_SMOKE, "--webview2-arguments", "policy"], {
      cwd: ROOT,
      stdio: "inherit",
    });
    child.once("error", (error) => {
      release();
      fail(error);
    });
    child.once("exit", (code, signal) => {
      release();
      if (code === null) log(`The smoke ended on ${signal}`);
      done(code ?? 1);
    });
  });
}

export async function main({
  argv = process.argv.slice(2),
  platform = process.platform,
  env = process.env,
  probe = probeElevation,
  registry,
  runSmoke = runNativeSmoke,
  log = console.log,
  error = console.error,
} = {}) {
  if (argv.length > 0) {
    error("ci-windows-webview2-policy takes no arguments; it always runs the default smoke.");
    return 1;
  }
  const context = contextRefusal({ platform, env });
  if (context) {
    error(`Refused before any registry access: ${context}.`);
    return 1;
  }
  let elevation;
  try {
    elevation = await probe();
  } catch (problem) {
    error(
      `Refused before any registry access: could not read this process's token: ${problem.message}`,
    );
    return 1;
  }
  const notElevated = elevationRefusal(elevation);
  if (notElevated) {
    error(`Refused before any registry access: ${notElevated}. Elevation is never requested.`);
    return 1;
  }
  log(
    `Hosted context: RUNNER_ENVIRONMENT=${env.RUNNER_ENVIRONMENT}, RUNNER_OS=${env.RUNNER_OS}, ` +
      `integrity=${INTEGRITY[elevation.integritySid]}, administrator=${elevation.administrator}`,
  );
  const { exitCode, problems } = await runUnderPolicy({
    registry: registry ?? windowsRegistry(),
    runSmoke,
    log,
  });
  for (const problem of problems) error(problem);
  return exitCode;
}

if (
  process.argv[1] &&
  realpathSync(process.argv[1]) === realpathSync(fileURLToPath(import.meta.url))
) {
  process.exitCode = await main();
}
