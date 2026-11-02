#!/usr/bin/env node
// DEVELOPMENT ORACLE ONLY. Never shipped, never a runtime dependency.
//
// Runs the legacy Madden-Draft-Generator M27 parser (M27Parser.js, pinned by SHA-256)
// on one draft-class file and prints its view as JSON for tests/differential.rs.
// It reads the class file and never writes anything.
//
// Usage: node legacy-m27-oracle.cjs <path/to/M27Parser.js> <class file>
'use strict';

const crypto = require('crypto');
const fs = require('fs');
const path = require('path');

const PINNED_PARSER_SHA256 = 'ce24b0063c24e1ca7ce8be8a94717dc96716a49073fa4f0ed6f0e4430c1f067b';
const DATA_START = 0x46; // as legacy Mdc27Service passes it

function sha256(buf) {
  return crypto.createHash('sha256').update(buf).digest('hex');
}

function main() {
  const [parserPath, classPath] = process.argv.slice(2);
  if (!parserPath || !classPath) {
    console.error('usage: legacy-m27-oracle.cjs <M27Parser.js> <class file>');
    process.exit(2);
  }
  const parserSha = sha256(fs.readFileSync(parserPath));
  if (parserSha !== PINNED_PARSER_SHA256) {
    console.error(`M27Parser.js SHA-256 ${parserSha} is not the pinned ${PINNED_PARSER_SHA256}`);
    process.exit(3);
  }
  const parser = require(path.resolve(parserPath));
  const buf = fs.readFileSync(classPath);
  const prospects = parser.parseM27Prospects(buf, { dataStartOffset: DATA_START });

  // The legacy field map as it declares it, for an offset and width comparison.
  const fieldMap = {};
  for (const [name, spec] of Object.entries(parser.M27_FIELDS)) {
    const width = spec.len ?? (spec.u16x5 ? 10 : spec.u16 ? 2 : 1);
    fieldMap[name] = { offset: spec.off, width, ...(spec.value !== undefined ? { value: spec.value } : {}) };
  }
  for (const [name, off] of Object.entries(parser.M27_RATINGS)) {
    fieldMap[name] = { offset: off, width: 1 };
  }

  const blockSize = parser.M27_BLOCK_SIZE;
  const records = prospects.map((p, i) => {
    const start = DATA_START + i * blockSize;
    const used = buf.subarray(start + parser.M27_ATTRIBUTE_OFFSET, start + blockSize).some((b) => b !== 0);
    const { visuals, ...attrs } = p;
    return {
      index: i,
      used,
      // Computed here from the file itself, independently of either parser.
      recordSha256: sha256(buf.subarray(start, start + blockSize)),
      attrs,
      visuals,
    };
  });

  process.stdout.write(JSON.stringify({
    parserSha256: parserSha,
    fileSha256: sha256(buf),
    blockSize,
    visualSize: parser.M27_VISUAL_SIZE,
    attributeOffset: parser.M27_ATTRIBUTE_OFFSET,
    attributeSize: parser.M27_ATTRIBUTE_SIZE,
    fieldMap,
    records,
  }));
}

main();
