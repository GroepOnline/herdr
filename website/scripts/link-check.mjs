#!/usr/bin/env node
/**
 * Internal link checker for the built Astro site.
 *
 * Crawls all HTML files in dist/, extracts <a href> values, and verifies
 * that every internal link (starting with "/") resolves to a generated
 * page or static asset.  Anchor-only links and external links are skipped.
 */
import { readdirSync, readFileSync, statSync, existsSync } from "node:fs";
import { join, relative } from "node:path";

const DIST = new URL("../dist/", import.meta.url).pathname;

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

// Collect all valid internal paths: HTML files (as clean URLs) + static assets.
function collectPaths(dir, base = "") {
  const paths = new Set();
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    const rel = base ? `${base}/${entry}` : entry;
    const stat = statSync(full);
    if (stat.isDirectory()) {
      for (const p of collectPaths(full, rel)) paths.add(p);
    } else {
      if (entry === "index.html") {
        paths.add(`/${base}/`);
        paths.add(`/${base}`);
      } else if (entry.endsWith(".html")) {
        const slug = entry.slice(0, -5);
        paths.add(`/${base ? base + "/" : ""}${slug}`);
        paths.add(`/${base ? base + "/" : ""}${slug}/`);
      } else {
        paths.add(`/${rel}`);
      }
    }
  }
  return paths;
}

const files = walkHtml(DIST);
const validPaths = collectPaths(DIST);

const HREF_RE = /href="([^"]+)"/gi;
const broken = [];

for (const file of files) {
  const rel = relative(DIST, file);
  // Pages under docs/preview/ are generated from docs/next/ by prepare-docs.mjs.
  // Their locale path structure does not match Starlight's routing; link
  // integrity for preview docs is verified in the prepare-docs script.
  if (rel.startsWith("docs/preview/")) continue;
  const html = readFileSync(file, "utf8");
  let match;
  while ((match = HREF_RE.exec(html)) !== null) {
    let href = match[1];

    // Skip external, anchor-only, mailto, tel, and non-http schemes.
    if (
      href.startsWith("http") ||
      href.startsWith("//") ||
      href.startsWith("#") ||
      href.startsWith("mailto:") ||
      href.startsWith("tel:") ||
      !href.startsWith("/")
    ) {
      continue;
    }

    // Strip query strings and anchors.
    href = href.split(/[?#]/)[0];

    // Skip empty after stripping.
    if (!href) continue;

    // Normalise: ensure trailing slash for directory-like paths.
    const candidates = [href, href.endsWith("/") ? href.slice(0, -1) : href + "/"];

    const found = candidates.some((c) => validPaths.has(c));
    if (!found) {
      broken.push({ file: relative(DIST, file), href });
    }
  }
}

if (broken.length > 0) {
  console.error(`\n  ${broken.length} broken internal link(s):\n`);
  for (const { file, href } of broken) {
    console.error(`  ${file} → ${href}`);
  }
  console.error("");
  process.exit(1);
} else {
  console.log(`  All internal links resolve (${files.length} pages checked).`);
}
