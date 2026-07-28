/**
 * User-defined channels from
 * $HERDR_PLUGIN_CONFIG_DIR/channels.json
 * (default ~/.config/herdr/plugins/config/com.chefgroep.output-panel/channels.json)
 *
 * Shape:
 *   { "channels": [
 *       { "id": "nginx", "title": "nginx error log", "kind": "file",
 *         "path": "/var/log/nginx/error.log", "group": "Custom", "lines": 200 },
 *       { "id": "docker", "title": "docker ps", "kind": "command",
 *         "argv": ["docker", "ps", "--all"], "timeout_ms": 5000 }
 *   ] }
 *
 * A bare array is accepted too. Everything is redacted like any other channel.
 */
import path from "node:path";

import { channel, failure, result } from "../lib/channel.js";
import { explainFailure, run } from "../lib/exec.js";
import { exists, humanAge, humanSize, readJson, readTail, statOrNull } from "../lib/files.js";
import { pluginConfigDir, tilde } from "../lib/paths.js";

const PREFIX = "custom:";
export const CONFIG_FILE_NAME = "channels.json";

export function customConfigPath() {
  return path.join(pluginConfigDir(), CONFIG_FILE_NAME);
}

function normalize(entry, index) {
  if (!entry || typeof entry !== "object") return null;
  const kind = entry.kind === "command" ? "command" : "file";
  const id = String(entry.id || `${kind}-${index}`).replace(/[^\w.:-]/g, "-");
  if (kind === "file" && !entry.path) return null;
  if (kind === "command" && !(Array.isArray(entry.argv) && entry.argv.length)) return null;
  return {
    id,
    kind,
    title: String(entry.title || id),
    group: String(entry.group || "Custom"),
    path: entry.path ? String(entry.path) : null,
    argv: Array.isArray(entry.argv) ? entry.argv.map(String) : null,
    lines: Number(entry.lines) > 0 ? Number(entry.lines) : null,
    timeoutMs: Number(entry.timeout_ms) > 0 ? Number(entry.timeout_ms) : null,
  };
}

export const customProvider = {
  id: "custom",
  group: "Custom",

  async list() {
    const file = customConfigPath();
    if (!(await exists(file))) {
      return [
        channel({
          id: `${PREFIX}__setup`,
          title: "add your own channels",
          group: "Custom",
          subtitle: tilde(file),
          source: file,
          kind: "setup",
          meta: { path: file },
        }),
      ];
    }
    let parsed;
    try {
      parsed = await readJson(file);
    } catch (err) {
      return [
        channel({
          id: `${PREFIX}__invalid`,
          title: "channels.json is invalid",
          group: "Custom",
          subtitle: String(err?.message ?? err).slice(0, 80),
          source: file,
          kind: "error",
          meta: { path: file, error: String(err?.message ?? err) },
        }),
      ];
    }
    const raw = Array.isArray(parsed) ? parsed : parsed?.channels;
    const entries = (Array.isArray(raw) ? raw : []).map(normalize).filter(Boolean);
    if (!entries.length) {
      return [
        channel({
          id: `${PREFIX}__empty`,
          title: "channels.json has no usable entries",
          group: "Custom",
          subtitle: tilde(file),
          source: file,
          kind: "setup",
          meta: { path: file },
        }),
      ];
    }
    const channels = [];
    for (const entry of entries) {
      let subtitle = entry.kind === "command" ? entry.argv.join(" ") : tilde(entry.path);
      if (entry.kind === "file") {
        const st = await statOrNull(entry.path);
        subtitle = st
          ? `${humanSize(st.size)}  ${humanAge(st.mtimeMs)}  ${tilde(entry.path)}`
          : `missing: ${tilde(entry.path)}`;
      }
      channels.push(
        channel({
          id: `${PREFIX}${entry.id}`,
          title: entry.title,
          group: entry.group,
          subtitle,
          source: entry.kind === "command" ? entry.argv.join(" ") : entry.path,
          kind: entry.kind === "command" ? "command" : "file",
          meta: entry,
        }),
      );
    }
    return channels;
  },

  async read(ch, ctx, opts = {}) {
    if (ch.kind === "setup") {
      return result(
        [
          "No custom channels configured.",
          "",
          `Create: ${ch.meta.path}`,
          "",
          JSON.stringify(
            {
              channels: [
                {
                  id: "syslog",
                  title: "system log",
                  kind: "file",
                  path: "/var/log/syslog",
                  group: "Custom",
                  lines: 300,
                },
                {
                  id: "docker-ps",
                  title: "docker containers",
                  kind: "command",
                  argv: ["docker", "ps", "--all"],
                  group: "Custom",
                  timeout_ms: 5000,
                },
              ],
            },
            null,
            2,
          ),
          "",
          "See plugins/output-panel/README.md and channels.example.json.",
        ].join("\n"),
        { raw: true, source: ch.meta.path },
      );
    }
    if (ch.kind === "error") {
      return failure({
        tried: ch.meta.path,
        reason: ch.meta.error,
        hint: "fix the JSON syntax, then press R",
      });
    }

    if (ch.kind === "command") {
      const argv = ch.meta.argv;
      const res = await run(argv[0], argv.slice(1), {
        timeoutMs: ch.meta.timeoutMs || ctx.timeoutMs,
      });
      const body = `${res.stdout}${res.stderr ? `\n${res.stderr}` : ""}`.trimEnd();
      if (!res.ok && !body) {
        return failure({
          tried: argv.join(" "),
          reason: explainFailure(res),
          hint: `check the argv for "${ch.meta.id}" in ${tilde(customConfigPath())}`,
          source: argv.join(" "),
        });
      }
      return result(body || "(no output)", { source: argv.join(" ") });
    }

    if (!(await exists(ch.meta.path))) {
      return failure({
        tried: ch.meta.path,
        reason: "file does not exist or is not readable by this user",
        hint: `fix "path" for "${ch.meta.id}" in ${tilde(customConfigPath())}`,
      });
    }
    const lines = ch.meta.lines || opts.lines || ctx.lines;
    const tail = await readTail(ch.meta.path, lines);
    return result(tail.text, {
      source: ch.meta.path,
      truncated: tail.truncated,
      note: `tail ${lines} lines of ${humanSize(tail.size)}`,
    });
  },
};
