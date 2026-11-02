import assert from "node:assert/strict";
import { test } from "node:test";
import { evaluate, globToRegExp, validateManifest } from "./check-provenance.mjs";

const manifest = (paths = ["ui/**"]) => ({
  schemaVersion: 1,
  entries: paths.map((path) => ({ path, origin: "original", license: "UNLICENSED", ships: true })),
});
const file = (path, text = "hello", bytes = 5) => ({ path, text, bytes });

test("listed files pass", () => {
  assert.deepEqual(evaluate([file("ui/src/App.tsx")], manifest()), []);
});

test("a file missing from the manifest fails", () => {
  const problems = evaluate([file("ui/a.ts"), file("art/logo.png", null)], manifest());
  assert.equal(problems.length, 1);
  assert.match(problems[0], /art\/logo\.png: not listed/);
});

test("a stale manifest entry fails", () => {
  const problems = evaluate([file("ui/a.ts")], manifest(["ui/**", "gone/**"]));
  assert.match(problems[0], /"gone\/\*\*" matches no tracked file/);
});

for (const path of [
  "ui/CAREER-MY_SAVE",
  "ui/fixtures/CAREERDRAFT-2027",
  "ui/app.asar",
  "ui/pack.fbmod",
  "ui/portrait.DDS",
  "ui/setup.exe",
  "ui/bundle.zip",
  "ui/.env",
  "ui/.env.local",
  "ui/signing.pem",
]) {
  test(`forbidden even when listed: ${path}`, () => {
    const problems = evaluate([file(path)], manifest());
    assert.equal(problems.length, 1);
    assert.match(problems[0], /forbidden/);
  });
}

test("a personal absolute path in a text file fails", () => {
  const windowsPath = ["C:", "Users", "someone", "save"].join("\\");
  const problems = evaluate([file("ui/a.ts", `const p = "${windowsPath}"`)], manifest());
  assert.match(problems[0], /personal absolute path/);
});

test("the sanitized placeholder form is allowed", () => {
  assert.deepEqual(evaluate([file("ui/a.md", "%USERPROFILE%\\Downloads")], manifest()), []);
});

test("an oversized file fails unless explicitly allowed", () => {
  const big = file("ui/big.bin", null, 2 * 1024 * 1024);
  assert.match(evaluate([big], manifest())[0], /larger than 1 MiB/);
  const allowed = manifest();
  allowed.entries[0].allowLarge = true;
  assert.deepEqual(evaluate([big], allowed), []);
});

test("third-party material must name its source", () => {
  const problems = validateManifest({
    schemaVersion: 1,
    entries: [{ path: "x", origin: "third-party", license: "MIT", ships: false }],
  });
  assert.match(problems[0], /needs a source/);
});

test("an incomplete or unversioned manifest fails", () => {
  assert.ok(validateManifest({ schemaVersion: 2, entries: [] }).length >= 2);
  assert.match(validateManifest({ schemaVersion: 1, entries: [{ path: "x" }] })[0], /origin/);
});

test("globs respect path segments", () => {
  assert.ok(globToRegExp("ui/**").test("ui/src/a.ts"));
  assert.ok(globToRegExp("*.json").test("package.json"));
  assert.ok(!globToRegExp("*.json").test("ui/package.json"));
  assert.ok(globToRegExp("docs/**/*.md").test("docs/README.md"));
  assert.ok(globToRegExp("docs/**/*.md").test("docs/a/b/c.md"));
  assert.ok(!globToRegExp("Cargo.toml").test("CargoXtoml"));
});
