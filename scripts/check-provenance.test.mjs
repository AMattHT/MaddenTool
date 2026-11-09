import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { evaluate, globToRegExp, hasPersonalPath, validateManifest } from "./check-provenance.mjs";

const manifest = (paths) => ({
  schemaVersion: 1,
  entries: paths.map((path) => ({ path, origin: "original", license: "UNLICENSED", ships: true })),
});
// A rule that accepts anything, for tests about the file checks rather than the rule checks.
const anyFile = (path) => manifest([path]);
const file = (path, text = "hello", bytes = 5) => ({ path, text, bytes });

const realManifest = JSON.parse(readFileSync("docs/provenance/manifest.json", "utf8"));
const trackedFiles = execFileSync("git", ["ls-files", "-z"], { encoding: "utf8" })
  .split("\0")
  .filter(Boolean)
  .map((path) => file(path));

test("listed files pass", () => {
  assert.deepEqual(evaluate([file("ui/src/App.ts")], manifest(["ui/src/**/*.ts"])), []);
});

test("a file missing from the manifest fails", () => {
  const problems = evaluate(
    [file("ui/a.ts"), file("art/logo.png", null)],
    manifest(["ui/**/*.ts"]),
  );
  assert.equal(problems.length, 1);
  assert.match(problems[0], /art\/logo\.png: not listed/);
});

test("a stale manifest entry fails", () => {
  const problems = evaluate([file("ui/a.ts")], manifest(["ui/**/*.ts", "gone/a.json"]));
  assert.match(problems[0], /"gone\/a\.json" matches no tracked file/);
});

test("the real manifest accounts for every tracked file", () => {
  assert.deepEqual(evaluate(trackedFiles, realManifest), []);
});

// New material must arrive with its own record, whatever its folder or extension.
for (const path of [
  "ui/public/madden-27-logo.png",
  "app/src-tauri/resources/tuning.json",
  "docs/reference/myfranchise-extract/index-CqbtHSxo.js",
  "docs/reference/notes.md",
  "docs/reference/audit/extracted.js",
  "docs/reference/audit/probes/index-CqbtHSxo.js",
  "docs/reference/tstack/vendored.js",
  "docs/art/cover.png",
  "ui/src/assets/portrait.png",
  "ui/src/tuning.json",
  "app/src-tauri/icons/madden-logo.png",
  "app/src-tauri/capabilities/extra.json",
  "scripts/tuning.json",
  ".hidden-data",
  "ratings.json",
  "bundle.js",
  "data.toml",
  "crates/mt-store/data/ratings.json",
  "crates/mt-store/tests/fixtures/draft_picks.csv",
  "crates/mt-store/Cargo.lock",
  "docs/design/concepts/dg05-concept-b2/assembled/A-dark-1440x900.png",
  "docs/design/concepts/dg05-concept-a/assembled/dev-icon-superstar.png",
  "docs/design/concepts/dg05-concept-a/source/build_extra.py",
  "docs/design/concepts/dg05-concept-b1/comparison-inline.html",
]) {
  test(`needs a specific record in the real manifest: ${path}`, () => {
    const problems = evaluate([...trackedFiles, file(path)], realManifest);
    assert.deepEqual(problems, [`${path}: not listed in docs/provenance/manifest.json`]);
  });
}

for (const path of [
  "ui/src/feature/View.tsx",
  "app/src-tauri/src/feature/mod.rs",
  "docs/prompts/02-next.md",
  "scripts/another.mjs",
  "crates/mt-store/src/ratings.rs",
  "crates/mt-store/tests/ratings.rs",
  "docs/reviews/dg05-concept-b2-sol-loop.md",
  "docs/design/concepts/dg05-concept-b2/DECISIONS.md",
]) {
  test(`reviewable source stays accepted in the real manifest: ${path}`, () => {
    assert.deepEqual(evaluate([...trackedFiles, file(path)], realManifest), []);
  });
}

for (const path of [
  "ui/**",
  "ui/**/*",
  "ui/*",
  "*.json",
  "*.js",
  ".*",
  "ui/**/*.png",
  "ui/**/*.json",
  "app/src-tauri/**/*.toml",
  "docs/reference/**/*.md",
  "docs/reference/*-inventory.json",
]) {
  test(`a catch-all rule is refused: ${path}`, () => {
    const problems = validateManifest(manifest([path]));
    assert.equal(problems.length, 1, problems.join());
    assert.match(problems[0], /catch-all|one reference file/);
  });
}

test("exact media and reference records are accepted once listed", () => {
  const paths = ["ui/public/art.png", "docs/reference/extra.js"];
  assert.deepEqual(
    evaluate(
      paths.map((path) => file(path)),
      manifest(paths),
    ),
    [],
  );
});

for (const path of [
  "ui/CAREER-MY_SAVE",
  "ui/fixtures/CAREERDRAFT-2027",
  "ui/ROSTER-2027",
  "ui/fixtures/roster-Official",
  "ui/PROFILE-1234",
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
    const problems = evaluate([file(path)], anyFile(path));
    assert.equal(problems.length, 1);
    assert.match(problems[0], /forbidden/);
  });
}

test("save names are ignored by Git as well as refused by the check", () => {
  const names = ["CAREER-X", "CAREERDRAFT-X", "ROSTER-X", "PROFILE-X", "ui/fixtures/ROSTER-X"];
  const ignored = execFileSync("git", ["check-ignore", "--no-index", ...names], {
    encoding: "utf8",
  });
  assert.deepEqual(ignored.trim().split(/\r?\n/), names);
});

// Assembled so these synthetic examples do not trip the scan of this file.
const join = (...parts) => parts.join("");
for (const [name, path] of [
  ["Windows", join("C:\\", "Users\\someone\\save")],
  ["Windows, lower case", join("c:\\", "users\\someone\\save")],
  ["Windows, upper case", join("C:\\", "USERS\\someone")],
  ["Windows, forward slashes", join("C:/", "Users/someone/save")],
  ["Windows, escaped in JSON", join("C:\\\\", "Users\\\\someone\\\\save")],
  ["Windows, file URL", join("file:///c:/", "Users/someone/save")],
  ["Windows, other drive", join("D:\\", "Users\\someone")],
  ["macOS", join("/", "Users/someone/Library")],
  ["macOS, quoted", join('"/', 'Users/someone"')],
  ["Linux", join("/", "home/someone/.config")],
  ["Linux, after a space", join("cd /", "home/someone")],
  ["Git Bash", join("/c/", "Users/someone/save")],
  ["WSL", join("/mnt/c/", "Users/someone/save")],
]) {
  test(`a personal path is found: ${name}`, () => {
    assert.ok(hasPersonalPath(path), path);
    const problems = evaluate([file("ui/a.ts", `const p = "${path}"`)], anyFile("ui/a.ts"));
    assert.match(problems[0], /personal absolute path/);
  });
}

for (const [name, text] of [
  ["environment placeholder", "%USERPROFILE%\\Downloads"],
  ["Windows name placeholder", join("C:\\", "Users\\%USERNAME%\\Downloads")],
  ["Windows angle placeholder", join("C:\\", "Users\\<you>\\Downloads")],
  ["shell home", "$HOME/.cargo and ~/.cargo"],
  ["macOS placeholder", join("/", "Users/<you>/Library")],
  ["Linux variable", join("/", "home/$USER/.config")],
  ["Linux braces", join("/", "home/{user}/.config")],
  ["shared Windows folder", join("C:\\", "Users\\Public\\Documents")],
  ["shared macOS folder", join("/", "Users/Shared/data")],
  ["a URL route", join("https://example.com/", "home/page and /api/", "Users/list")],
  ["a URL host and path", join("https://", "Users/someone")],
  ["an app route", join('path: "/users/', ':id"')],
  ["a relative path", join("src/", "home/view.ts and ./", "Users/list.ts")],
]) {
  test(`a portable form is not a personal path: ${name}`, () => {
    assert.equal(hasPersonalPath(text), false, text);
  });
}

test("an oversized file fails unless explicitly allowed", () => {
  const big = file("ui/big.bin", null, 2 * 1024 * 1024);
  assert.match(evaluate([big], anyFile("ui/big.bin"))[0], /larger than 1 MiB/);
  const allowed = anyFile("ui/big.bin");
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
