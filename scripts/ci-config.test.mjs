// Guards on the gate's own definition, which no other test would notice being weakened.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const pkg = JSON.parse(readFileSync("package.json", "utf8"));
const ci = readFileSync(".github/workflows/ci.yml", "utf8");
const readme = readFileSync("README.md", "utf8");

/** Commands in a chained script or a workflow's run lines. */
const commands = (text) => text.split(/&&|\r?\n/).map((part) => part.trim());

test("clippy and tests refuse a stale Cargo.lock", () => {
  const rust = commands(pkg.scripts["check:rust"]);
  for (const tool of ["cargo clippy", "cargo test"]) {
    const command = rust.find((part) => part.startsWith(tool));
    assert.ok(command, `${tool} is part of check:rust`);
    const beforeSeparator = command.split(" -- ")[0];
    assert.match(beforeSeparator, /(^| )--locked( |$)/, command);
  }
});

test("every release build passes --locked through to Cargo", () => {
  assert.match(pkg.scripts.build, /^tauri build .*-- --locked$/);
  const builds = commands(ci).filter((line) => /(tauri build|run build|-- build)\b/.test(line));
  assert.ok(builds.length > 0, "CI builds the app");
  for (const line of builds) assert.match(line, / -- --locked$/, line);
});

test("no CI step builds through a form that would give Cargo the Tauri options", () => {
  assert.doesNotMatch(ci, /npm run build -- /);
});

const aptPackages = (text) => {
  const match = /apt-get install(?: -y)?\s+((?:[\w.+-]+(?:\s*\\\r?\n\s*|[ \t]+)?)+)/.exec(text);
  assert.ok(match, "an apt-get install line exists");
  return match[1].split(/[\s\\]+/).filter(Boolean);
};

test("documented Linux prerequisites match CI, apart from CI's own smoke tools", () => {
  const documented = aptPackages(readme);
  assert.ok(documented.includes("build-essential"), "the compiler and linker are documented");
  const smokeOnly = ["webkit2gtk-driver", "xvfb"];
  assert.deepEqual([...documented, ...smokeOnly].sort(), aptPackages(ci).sort());
});

test("the Windows smoke runs only after the runtime has been verified", () => {
  const verify = ci.indexOf("./scripts/ensure-webview2.ps1");
  const smoke = ci.indexOf("Native smoke (Windows");
  assert.ok(verify !== -1 && verify < smoke);
});

// Diagnostic evidence for the hosted Windows smoke, which a local run cannot stand in for.
const stepBlock = (name) => {
  const start = ci.indexOf(`- name: ${name}`);
  assert.ok(start !== -1, `step "${name}" exists`);
  const next = ci.indexOf("\n      - ", start + 1);
  return ci.slice(start, next === -1 ? undefined : next);
};

test("the Windows binary's hash is logged before its smoke runs", () => {
  const record = stepBlock("Record the built binary (Windows)");
  assert.match(record, /if: runner\.os == 'Windows'/);
  assert.match(record, /Get-FileHash -Algorithm SHA256 target[\\/]release[\\/]maddentool\.exe/);
  assert.ok(ci.indexOf("Record the built binary (Windows)") < ci.indexOf("Native smoke (Windows"));
});

test("a failed Windows smoke keeps only the built executable, briefly", () => {
  const upload = stepBlock("Keep the built binary of a failed Windows smoke");
  // Only when the smoke itself failed: an earlier failure leaves no binary to keep.
  assert.match(upload, /if: failure\(\) && steps\.windows-smoke\.outcome == 'failure'/);
  assert.match(stepBlock("Native smoke (Windows, DOM and IPC)"), /\n\s+id: windows-smoke\b/);
  assert.match(upload, /uses: actions\/upload-artifact@[0-9a-f]{40} # v\d+\.\d+\.\d+/);
  const path = /\n\s+path: ([^\r\n]+)/.exec(upload)?.[1];
  assert.equal(path, "target/release/maddentool.exe");
  assert.match(upload, /if-no-files-found: error/);
  const days = Number(/retention-days: (\d+)/.exec(upload)?.[1]);
  assert.ok(days >= 1 && days <= 7, `retention-days ${days}`);
  assert.ok(ci.indexOf("Native smoke (Windows") < ci.indexOf("Keep the built binary"));
});

const POLICY_HELPER = "node scripts/ci-windows-webview2-policy.mjs";

test("Windows still has to pass the real smoke, run inside the machine-policy helper", () => {
  const smoke = stepBlock("Native smoke (Windows, DOM and IPC)");
  assert.match(smoke, /\n\s+if: runner\.os == 'Windows'\r?\n/);
  const run = /\n\s+run: ([^\r\n]+)/.exec(smoke)?.[1];
  assert.equal(run, POLICY_HELPER);
  // The helper's exit code is the smoke's; nothing may soften or replace it.
  assert.doesNotMatch(ci, /continue-on-error/);
  assert.doesNotMatch(smoke, /\|\||; *exit 0|\$LASTEXITCODE|env:/);
});

test("the machine-policy helper runs only in the Windows smoke step", () => {
  assert.equal(ci.split("ci-windows-webview2-policy").length - 1, 1);
  assert.equal(pkg.scripts["smoke:native"], "node scripts/native-smoke.mjs");
  for (const name of [
    "Native smoke (Linux, DOM under xvfb)",
    "Native smoke (macOS, process start only)",
  ]) {
    assert.match(stepBlock(name), /npm run smoke:native/);
    assert.doesNotMatch(stepBlock(name), /webview2-arguments|policy/i);
  }
});

test("no workflow step writes the registry or a WebView2 override itself", () => {
  assert.doesNotMatch(ci, /HKLM|HKCU|HKEY_|Registry::|reg(\.exe)? (add|delete)|ItemProperty/i);
  assert.doesNotMatch(ci, /WEBVIEW2_|AdditionalBrowserArguments|--webview2-arguments/);
});
