#!/usr/bin/env node
const { execFileSync } = require("child_process");
const { existsSync } = require("fs");
const { join } = require("path");
const os = require("os");

const BINARY_NAME = os.platform() === "win32" ? "herdr.exe" : "herdr";

const paths = [
  join(__dirname, BINARY_NAME),
  join(__dirname, "..", "bin", BINARY_NAME),
  join(__dirname, "..", "..", "bin", BINARY_NAME),
  BINARY_NAME,
];

for (const p of paths) {
  if (existsSync(p)) {
    try {
      execFileSync(p, process.argv.slice(2), { stdio: "inherit" });
      return;
    } catch {
      continue;
    }
  }
}

console.error("herdr binary not found. Install via:");
console.error("  npm install -g onlinechefgroep-herdr");
console.error("Or: https://github.com/OnlineChefGroep/herdr/releases");
process.exit(1);
