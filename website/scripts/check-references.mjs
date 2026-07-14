#!/usr/bin/env node
/**
 * Check built website output for stale upstream references.
 *
 * After the OnlineChefGroep rebrand, no built page should contain:
 *   - herdr.dev                 (old canonical domain → herdr.chefgroep.nl)
 *   - ogulcancelik/herdr        (old upstream repo  → OnlineChefGroep/herdr)
 *   - github.com/ogulcancelik   (old GitHub user)
 *   - hey@herdr.dev             (old contact email)
 *
 * Historical changelog entries are exempt.
 *
 * This check is non-blocking in CI until the rebrand PRs (3B/3C) land.
 * Once the rebrand is complete, remove the `continue-on-error` from the
 * website workflow to make this a hard gate.
 */
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const DIST = fileURLToPath(new URL("../dist/", import.meta.url));

const STALE_PATTERNS = [
  { pattern: "herdr.dev", label: "old canonical domain" },
  { pattern: "ogulcancelik/herdr", label: "old upstream repo" },
  { pattern: "github.com/ogulcancelik", label: "old GitHub user" },
  { pattern: "hey@herdr.dev", label: "old contact email" },
];

// Directories and files exempt from the check.
const EXEMPT_SUBSTRINGS = ["/changelog/", "/history/", "CHANGELOG"];

function walkHtml(dir) {
  const results = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    const stat = statSync(full);
    if (stat.isDirectory()) {
      results.push(...walkHtml(full));
    } else if (entry.endsWith(".html")) {
      results.push(full);
    }
  }
  return results;
}

const files = walkHtml(DIST);
const findings = [];

for (const file of files) {
  const rel = relative(DIST, file).replaceAll("\\", "/");
  if (EXEMPT_SUBSTRINGS.some((s) => rel.includes(s))) continue;

  const html = readFileSync(file, "utf8");
  for (const { pattern, label } of STALE_PATTERNS) {
    if (html.includes(pattern)) {
      findings.push({ file: rel, pattern, label });
    }
  }
}

if (findings.length > 0) {
  console.error(`\n  ${findings.length} stale reference(s) found:\n`);
  for (const { file, pattern, label } of findings) {
    console.error(`  ${file}: "${pattern}" (${label})`);
  }
  console.error(
    "\n  These references will be replaced by the rebrand PRs (3B/3C).\n" +
      "  This check is non-blocking until then.\n",
  );
  process.exit(1);
} else {
  console.log("  No stale upstream references found.");
}
