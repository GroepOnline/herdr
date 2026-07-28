/**
 * Filesystem helpers: existence checks, bounded tail reads, recursive scans.
 *
 * `readTail` walks a file backwards in chunks so tailing an 800 KB server log
 * never pulls the whole thing into memory.
 */
import { constants } from "node:fs";
import { access, open, readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";

export async function exists(p) {
  try {
    await access(p, constants.R_OK);
    return true;
  } catch {
    return false;
  }
}

export async function statOrNull(p) {
  try {
    return await stat(p);
  } catch {
    return null;
  }
}

/** Read the last `maxLines` lines of a file without loading the whole file. */
export async function readTail(file, maxLines = 400, maxBytes = 4 * 1024 * 1024) {
  const st = await stat(file);
  if (st.size === 0) return { text: "", size: 0, truncated: false };
  const fh = await open(file, "r");
  try {
    const chunkSize = 64 * 1024;
    let pos = st.size;
    let collected = Buffer.alloc(0);
    let newlines = 0;
    while (pos > 0 && newlines <= maxLines && collected.length < maxBytes) {
      const size = Math.min(chunkSize, pos);
      pos -= size;
      const chunk = Buffer.alloc(size);
      await fh.read(chunk, 0, size, pos);
      collected = Buffer.concat([chunk, collected]);
      newlines = countNewlines(collected);
    }
    const lines = collected.toString("utf8").split(/\r?\n/);
    // A partial first line is likely mid-sequence; drop it when we truncated.
    if (pos > 0 && lines.length > 1) lines.shift();
    return {
      text: lines.slice(-maxLines).join("\n"),
      size: st.size,
      truncated: pos > 0,
    };
  } finally {
    await fh.close();
  }
}

function countNewlines(buf) {
  let count = 0;
  for (let i = 0; i < buf.length; i += 1) {
    if (buf[i] === 0x0a) count += 1;
  }
  return count;
}

export async function readJson(file) {
  const raw = await readFile(file, "utf8");
  return JSON.parse(raw);
}

export async function listDir(dir) {
  try {
    return await readdir(dir, { withFileTypes: true });
  } catch {
    return [];
  }
}

/** Directories inside `dir`, newest mtime first. */
export async function newestDirs(dir, limit = 5) {
  const entries = await listDir(dir);
  const dirs = [];
  for (const entry of entries) {
    if (!entry.isDirectory()) continue;
    const full = path.join(dir, entry.name);
    const st = await statOrNull(full);
    if (st) dirs.push({ name: entry.name, path: full, mtimeMs: st.mtimeMs });
  }
  dirs.sort((a, b) => b.mtimeMs - a.mtimeMs);
  return dirs.slice(0, limit);
}

/**
 * Collect files matching `test` under `root`, newest first.
 * Depth- and count-bounded so a huge tree cannot stall channel listing.
 */
export async function findFiles(root, test, { maxDepth = 4, maxFiles = 4000 } = {}) {
  const found = [];
  const queue = [{ dir: root, depth: 0 }];
  while (queue.length && found.length < maxFiles) {
    const { dir, depth } = queue.shift();
    const entries = await listDir(dir);
    for (const entry of entries) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        if (depth < maxDepth) queue.push({ dir: full, depth: depth + 1 });
        continue;
      }
      if (!entry.isFile()) continue;
      if (!test(entry.name, full)) continue;
      const st = await statOrNull(full);
      if (st) found.push({ name: entry.name, path: full, mtimeMs: st.mtimeMs, size: st.size });
      if (found.length >= maxFiles) break;
    }
  }
  found.sort((a, b) => b.mtimeMs - a.mtimeMs);
  return found;
}

export function humanSize(bytes) {
  if (!Number.isFinite(bytes)) return "?";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function humanAge(mtimeMs) {
  const secs = Math.max(0, Math.round((Date.now() - mtimeMs) / 1000));
  if (secs < 60) return `${secs}s ago`;
  if (secs < 3600) return `${Math.round(secs / 60)}m ago`;
  if (secs < 86400) return `${Math.round(secs / 3600)}h ago`;
  return `${Math.round(secs / 86400)}d ago`;
}
