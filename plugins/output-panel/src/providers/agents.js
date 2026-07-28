/**
 * Agent session transcripts.
 *
 * Pi writes newline-delimited JSON under ~/.pi/agent/sessions/<cwd-slug>/,
 * Cursor writes the same shape under
 * ~/.cursor/projects/<project>/agent-transcripts/<uuid>/<uuid>.jsonl.
 *
 * Raw JSONL is unreadable in a pane, so each record is decoded into one line:
 * timestamp, record type, role, stop reason / error, and a text preview.
 */
import path from "node:path";

import { channel, failure, result } from "../lib/channel.js";
import { exists, findFiles, humanAge, humanSize, readTail } from "../lib/files.js";
import { cursorProjectsRoot, piSessionsRoot, tilde } from "../lib/paths.js";

const MAX_SESSIONS = 12;
const PREVIEW_CHARS = 220;

function collectText(content) {
  if (typeof content === "string") return content;
  if (!Array.isArray(content)) return "";
  const parts = [];
  for (const block of content) {
    if (typeof block === "string") parts.push(block);
    else if (block?.type === "text" && block.text) parts.push(block.text);
    else if (block?.type === "thinking") parts.push("[thinking]");
    else if (block?.type === "tool_use" || block?.type === "tool-call") {
      parts.push(`[tool ${block.name || block.toolName || "?"}]`);
    } else if (block?.type === "tool_result") parts.push("[tool result]");
    else if (block?.type) parts.push(`[${block.type}]`);
  }
  return parts.join(" ");
}

function oneLine(text) {
  return String(text).replace(/\s+/g, " ").trim();
}

/** Decode one JSONL record into a single readable line. */
function decodeRecord(raw, index) {
  let obj;
  try {
    obj = JSON.parse(raw);
  } catch {
    return `${String(index).padStart(4)}  (unparsable line) ${oneLine(raw).slice(0, 120)}`;
  }
  const ts = (obj.timestamp || obj.createdAt || "").toString().slice(11, 19);
  const type = obj.type || (obj.role ? "message" : "record");
  const role = obj.role || obj.message?.role || "";
  const bits = [];
  if (obj.provider || obj.modelId) bits.push(`${obj.provider || ""}${obj.modelId ? `/${obj.modelId}` : ""}`);
  if (obj.thinkingLevel) bits.push(`thinking=${obj.thinkingLevel}`);
  if (obj.cwd) bits.push(`cwd=${tilde(obj.cwd)}`);
  const stop = obj.message?.stopReason || obj.stopReason;
  if (stop) bits.push(`stop=${stop}`);
  const err = obj.errorMessage || obj.message?.errorMessage || obj.error;
  if (err) bits.push(`error=${oneLine(typeof err === "string" ? err : JSON.stringify(err))}`);
  const text = oneLine(collectText(obj.message?.content ?? obj.content));
  const head = [
    String(index).padStart(4),
    ts.padEnd(8),
    String(type).padEnd(14),
    role ? role.padEnd(9) : "".padEnd(9),
  ].join(" ");
  const tail = [bits.join(" "), text.slice(0, PREVIEW_CHARS)].filter(Boolean).join("  ");
  return `${head} ${tail}`.trimEnd();
}

async function piSessions() {
  const root = piSessionsRoot();
  if (!(await exists(root))) return { root, files: [], missing: true };
  const files = await findFiles(root, (name) => name.endsWith(".jsonl"), {
    maxDepth: 3,
    maxFiles: 500,
  });
  return { root, files: files.slice(0, MAX_SESSIONS), missing: false };
}

async function cursorTranscripts() {
  const root = cursorProjectsRoot();
  if (!(await exists(root))) return { root, files: [], missing: true };
  const files = await findFiles(
    root,
    (name, full) => name.endsWith(".jsonl") && full.includes(`${path.sep}agent-transcripts${path.sep}`),
    { maxDepth: 4, maxFiles: 800 },
  );
  return { root, files: files.slice(0, MAX_SESSIONS), missing: false };
}

/** Ids stay short and stable: the session file's own name, not its full path. */
function sessionChannel(prefix, file, label, taken) {
  const base = file.name.replace(/\.jsonl$/, "");
  let id = `${prefix}:${base}`;
  let n = 2;
  while (taken.has(id)) {
    id = `${prefix}:${base}#${n}`;
    n += 1;
  }
  taken.add(id);
  return channel({
    id,
    title: `${label} · ${base.slice(0, 44)}`,
    group: "Agents",
    subtitle: `${humanAge(file.mtimeMs)}  ${humanSize(file.size)}  ${tilde(path.dirname(file.path))}`,
    source: file.path,
    kind: "jsonl",
    meta: { path: file.path },
  });
}

export const agentsProvider = {
  id: "agents",
  group: "Agents",

  async list() {
    const [pi, cursor] = await Promise.all([piSessions(), cursorTranscripts()]);
    const channels = [];
    const taken = new Set();
    for (const file of pi.files) channels.push(sessionChannel("agent:pi", file, "pi", taken));
    for (const file of cursor.files) {
      channels.push(sessionChannel("agent:cursor", file, "cursor", taken));
    }
    if (!channels.length) {
      channels.push(
        channel({
          id: "agent:__empty",
          title: "no agent sessions found",
          group: "Agents",
          subtitle: `${tilde(pi.root)}  ${tilde(cursor.root)}`,
          source: `${pi.root} ${cursor.root}`,
          kind: "error",
          meta: { roots: [pi.root, cursor.root] },
        }),
      );
    }
    return channels;
  },

  async read(ch, ctx, opts = {}) {
    if (ch.kind === "error") {
      return failure({
        tried: ch.meta.roots.join("  "),
        reason: "no *.jsonl agent transcripts under either root",
        hint: "run a pi or Cursor agent session once, then press R",
      });
    }
    if (!(await exists(ch.meta.path))) {
      return failure({
        tried: ch.meta.path,
        reason: "session file no longer exists",
        hint: "press R to refresh the channel list",
      });
    }
    const lines = opts.lines || ctx.lines;
    const tail = await readTail(ch.meta.path, lines);
    const rows = tail.text
      .split(/\r?\n/)
      .filter((l) => l.trim())
      .map((l, i) => decodeRecord(l, i + 1));
    if (!rows.length) {
      return result("", {
        source: ch.meta.path,
        note: "session file is empty (agent started but produced no records yet)",
      });
    }
    return result(rows.join("\n"), {
      source: ch.meta.path,
      truncated: tail.truncated,
      note: `${rows.length} records decoded from the last ${lines} lines`,
    });
  },
};
