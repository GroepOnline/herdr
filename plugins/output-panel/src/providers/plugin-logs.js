/**
 * Herdr plugin invocation logs — one channel for everything, one per plugin.
 *
 * list: `herdr plugin list` (text) + `herdr plugin log list --limit N` (JSON)
 * read: `herdr plugin log list [--plugin ID] --limit N`, with the stdout and
 *       stderr of each invocation expanded instead of one-lined.
 */
import { channel, failure, result } from "../lib/channel.js";
import { explainFailure } from "../lib/exec.js";
import { herdrJson, herdrRun } from "../lib/herdr.js";

const ALL_ID = "plugin:logs:all";
const PREFIX = "plugin:logs:";

/** `- com.chefgroep.ops (Ops) enabled [local:/path]` */
const PLUGIN_LINE_RE = /^-\s+(\S+)\s+\(([^)]*)\)\s+(\S+)/;

async function installedPlugins(ctx) {
  const res = await herdrRun(["plugin", "list"], { timeoutMs: ctx.timeoutMs });
  if (!res.ok) return [];
  const out = [];
  for (const line of res.stdout.split(/\r?\n/)) {
    const m = PLUGIN_LINE_RE.exec(line.trim());
    if (m) out.push({ id: m[1], name: m[2], state: m[3] });
  }
  return out;
}

function formatLogEntry(entry) {
  const started = entry.started_unix_ms
    ? new Date(entry.started_unix_ms).toISOString().replace("T", " ").slice(0, 19)
    : "";
  const dur =
    entry.finished_unix_ms && entry.started_unix_ms
      ? `${entry.finished_unix_ms - entry.started_unix_ms}ms`
      : "";
  const head = [
    started,
    entry.status || "",
    entry.plugin_id || "",
    entry.event || "-",
    Array.isArray(entry.command) ? entry.command.join(" ") : "",
    dur,
    `exit=${entry.exit_code ?? "?"}`,
    entry.log_id || "",
  ]
    .filter(Boolean)
    .join("  ");
  const parts = [head];
  const stdout = (entry.stdout || "").trimEnd();
  const stderr = (entry.stderr || "").trimEnd();
  if (stdout) parts.push(indent(stdout, "  out| "));
  if (stderr) parts.push(indent(stderr, "  err| "));
  if (!stdout && !stderr) parts.push("  (no output)");
  return parts.join("\n");
}

function indent(text, prefix) {
  return text
    .split(/\r?\n/)
    .map((l) => prefix + l)
    .join("\n");
}

export const pluginLogsProvider = {
  id: "plugin-logs",
  group: "Plugins",

  async list(ctx) {
    const [{ result: data, res }, plugins] = await Promise.all([
      herdrJson(["plugin", "log", "list", "--limit", "200"], { timeoutMs: ctx.timeoutMs }),
      installedPlugins(ctx),
    ]);
    const logs = data?.logs || [];
    if (!data) {
      return [
        channel({
          id: ALL_ID,
          title: "plugin logs unavailable",
          group: "Plugins",
          subtitle: "herdr plugin log list failed",
          source: `${ctx.herdrBin} plugin log list --limit 200`,
          kind: "error",
          meta: { error: explainFailure(res, "is the Herdr server running?") },
        }),
      ];
    }

    const counts = new Map();
    const failures = new Map();
    for (const entry of logs) {
      const id = entry.plugin_id || "unknown";
      counts.set(id, (counts.get(id) || 0) + 1);
      if (entry.status && entry.status !== "succeeded") {
        failures.set(id, (failures.get(id) || 0) + 1);
      }
    }

    const channels = [
      channel({
        id: ALL_ID,
        title: "all plugin invocations",
        group: "Plugins",
        subtitle: `${logs.length} recent entries`,
        source: `${ctx.herdrBin} plugin log list --limit N`,
        kind: "plugin-log",
        meta: { plugin: null },
      }),
    ];

    const ids = new Set([...counts.keys(), ...plugins.map((p) => p.id)]);
    for (const id of [...ids].sort()) {
      const meta = plugins.find((p) => p.id === id);
      const n = counts.get(id) || 0;
      const bad = failures.get(id) || 0;
      channels.push(
        channel({
          id: `${PREFIX}${id}`,
          title: id.replace(/^com\.chefgroep\./, ""),
          group: "Plugins",
          subtitle: [
            meta?.name || "",
            meta?.state || "",
            `${n} recent`,
            bad ? `${bad} failed` : "",
          ]
            .filter(Boolean)
            .join("  "),
          source: `${ctx.herdrBin} plugin log list --plugin ${id} --limit N`,
          kind: "plugin-log",
          meta: { plugin: id, failures: bad },
        }),
      );
    }
    return channels;
  },

  async read(ch, ctx, opts = {}) {
    if (ch.kind === "error") {
      return failure({
        tried: ch.source,
        reason: ch.meta.error,
        hint: "start Herdr, then press R",
      });
    }
    const limit = String(opts.limit || 60);
    const args = ["plugin", "log", "list", "--limit", limit];
    if (ch.meta.plugin) args.splice(3, 0, "--plugin", ch.meta.plugin);
    const cmd = `${ctx.herdrBin} ${args.join(" ")}`;
    const { result: data, res } = await herdrJson(args, { timeoutMs: ctx.timeoutMs });
    if (!data) {
      return failure({ tried: cmd, reason: explainFailure(res), source: cmd });
    }
    const logs = data.logs || [];
    if (!logs.length) {
      return result("", {
        source: cmd,
        note: ch.meta.plugin
          ? `${ch.meta.plugin} has not been invoked recently; Herdr only keeps the newest entries`
          : "no plugin invocations recorded yet",
      });
    }
    return result(logs.map(formatLogEntry).join("\n\n"), {
      source: cmd,
      note: `${logs.length} invocations`,
    });
  },
};
