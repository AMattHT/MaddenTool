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

test("Windows still has to pass the real smoke", () => {
  const step = /- name: Native smoke \(Windows[^\n]*\r?\n\s+if:[^\n]*\r?\n\s+run: ([^\r\n]+)/.exec(
    ci,
  );
  assert.equal(step?.[1], "npm run smoke:native");
  assert.doesNotMatch(ci, /continue-on-error/);
});
