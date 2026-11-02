#!/usr/bin/env node
// XP-015: every tracked file needs a recorded origin and right to ship, and real game
// data or reference material must never be tracked. See docs/provenance/README.md.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

export const MANIFEST_PATH = "docs/provenance/manifest.json";
const MAX_FILE_BYTES = 1024 * 1024;

const FORBIDDEN = [
  {
    pattern: /(^|\/)(CAREER(DRAFT)?|ROSTER|PROFILE)-/i,
    reason: "looks like a real Madden save, roster, profile or draft class",
  },
  {
    pattern: /\.(asar|fbmod|frt|dds|sqlite3?|db|exe|dll|msi|dmg|deb|appimage)$/i,
    reason: "binary or game data type that never belongs in this repository",
  },
  { pattern: /\.(zip|7z|rar|tar|tgz|gz)$/i, reason: "archive that could hide third-party content" },
  { pattern: /(^|\/)\.env(\.|$)/i, reason: "local environment file" },
  { pattern: /\.(pem|key|p12|pfx)$/i, reason: "private key material" },
];

// A home folder with a real name after it. Placeholders such as %USERNAME%, $USER, <you> and
// {user} start with a character a name cannot, and the shared folders hold nobody's data.
const NAMED = "(?!(?:Public|Default|Shared|All Users)(?:[\\\\/\"'\\s]|$))[^%$<{\\s\\\\/\"'`)]";
// Assembled so this file does not match its own scan.
const PERSONAL_PATHS = [
  // Any case, either slash, escaped or not: a single-letter drive, so not a URL scheme.
  new RegExp(["(?<![A-Za-z])[A-Za-z]:[", `\\\\/]+Users[`, `\\\\/]+${NAMED}`].join(""), "i"),
  // Git Bash and WSL mounts of the same folder.
  new RegExp(["(?<![\\w.-])(?:/mnt)?/[a-z]/", `Users/${NAMED}`].join(""), "i"),
  // macOS and Linux homes, starting a path rather than continuing a URL or a route.
  new RegExp(["(?<![\\w.:/%-])/", `(?:Users|home)/${NAMED}`].join("")),
];

export const hasPersonalPath = (text) => PERSONAL_PATHS.some((pattern) => pattern.test(text));

// Rules that may use a wildcard, because review of a diff can read them as source. Everything
// else, meaning media, data and configuration, needs an entry naming the exact file.
const REVIEWABLE_SOURCE = /\.(md|ts|tsx|css|rs|mjs|ps1|yml)$/;
const REFERENCE_TREE = "docs/reference/";

const escapeRegExp = (text) => text.replace(/[.+^${}()|[\]\\]/g, "\\$&");

/** Supports `*` within a path segment and `**` across segments. */
export function globToRegExp(glob) {
  const source = glob
    .split("**/")
    .map((part) => part.split("**").map((p) => p.split("*").map(escapeRegExp).join("[^/]*")))
    .map((parts) => parts.join(".*"))
    .join("(?:.*/)?");
  return new RegExp(`^${source}$`);
}

export function validateManifest(manifest) {
  const problems = [];
  if (manifest?.schemaVersion !== 1) problems.push("manifest schemaVersion must be 1");
  if (!Array.isArray(manifest?.entries) || manifest.entries.length === 0) {
    return [...problems, "manifest needs at least one entry"];
  }
  manifest.entries.forEach((entry, index) => {
    for (const field of ["path", "origin", "license", "ships"]) {
      if (entry[field] === undefined || entry[field] === "") {
        problems.push(`manifest entry ${index} is missing ${field}`);
      }
    }
    if (typeof entry.ships !== "boolean") {
      problems.push(`manifest entry ${index}: ships must be true or false`);
    }
    if (entry.origin === "third-party" && !entry.source) {
      problems.push(`manifest entry ${index}: third-party material needs a source`);
    }
    if (typeof entry.path === "string" && entry.path.includes("*")) {
      if (entry.path.startsWith(REFERENCE_TREE)) {
        problems.push(`manifest entry ${index}: "${entry.path}" must name one reference file`);
      } else if (!REVIEWABLE_SOURCE.test(entry.path) || /\*\.?\*?$/.test(entry.path)) {
        problems.push(
          `manifest entry ${index}: "${entry.path}" is a catch-all. Name the file, or limit the rule to a reviewable source type`,
        );
      }
    }
  });
  return problems;
}

/**
 * @param files tracked files as `{ path, bytes, text }`; text is null for binary files.
 * @returns a list of human-readable problems; empty means the check passes.
 */
export function evaluate(files, manifest) {
  const problems = validateManifest(manifest);
  if (problems.length > 0) return problems;

  const rules = manifest.entries.map((entry) => ({
    entry,
    regex: globToRegExp(entry.path),
    used: false,
  }));

  for (const file of files) {
    for (const { pattern, reason } of FORBIDDEN) {
      if (pattern.test(file.path)) problems.push(`${file.path}: forbidden, ${reason}`);
    }
    const rule = rules.find(({ regex }) => regex.test(file.path));
    if (rule) rule.used = true;
    else problems.push(`${file.path}: not listed in ${MANIFEST_PATH}`);
    if (file.bytes > MAX_FILE_BYTES && !rule?.entry.allowLarge) {
      problems.push(`${file.path}: larger than 1 MiB without allowLarge`);
    }
    if (file.text !== null && hasPersonalPath(file.text)) {
      problems.push(`${file.path}: contains a personal absolute path`);
    }
  }
  for (const { entry, used } of rules) {
    if (!used) problems.push(`${MANIFEST_PATH}: "${entry.path}" matches no tracked file`);
  }
  return problems;
}

function trackedFiles() {
  const paths = execFileSync("git", ["ls-files", "-z"], {
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  })
    .split("\0")
    .filter(Boolean);
  return paths.map((path) => {
    const buffer = readFileSync(path);
    const binary = buffer.subarray(0, 8000).includes(0);
    return { path, bytes: buffer.length, text: binary ? null : buffer.toString("utf8") };
  });
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  const manifest = JSON.parse(readFileSync(MANIFEST_PATH, "utf8"));
  const files = trackedFiles();
  const problems = evaluate(files, manifest);
  if (problems.length > 0) {
    console.error(problems.join("\n"));
    console.error(`\nProvenance check failed: ${problems.length} problem(s).`);
    process.exit(1);
  }
  console.log(`Provenance check passed: ${files.length} tracked files accounted for.`);
}
