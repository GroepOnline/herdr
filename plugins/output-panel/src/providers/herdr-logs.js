/**
 * Herdr's own runtime output: server log, client log, config, live status.
 */
import path from "node:path";

import { channel, failure, result } from "../lib/channel.js";
import { explainFailure } from "../lib/exec.js";
import { exists, humanAge, humanSize, readTail, statOrNull } from "../lib/files.js";
import { herdrRun } from "../lib/herdr.js";
import { herdrConfigDir, tilde } from "../lib/paths.js";

const FILES = [
  { id: "herdr:server-log", file: "herdr-server.log", title: "herdr server log" },
  { id: "herdr:client-log", file: "herdr-client.log", title: "herdr client log" },
  { id: "herdr:config", file: "config.toml", title: "herdr config.toml" },
];

const STATUS_CHANNELS = [
  { id: "herdr:status", title: "herdr status", args: ["status"] },
  { id: "herdr:status-server", title: "herdr status server", args: ["status", "server"] },
];

export const herdrLogsProvider = {
  id: "herdr",
  group: "Herdr",

  async list(ctx) {
    const dir = herdrConfigDir();
    const channels = [];
    for (const spec of FILES) {
      const full = path.join(dir, spec.file);
      const st = await statOrNull(full);
      if (!st) continue;
      channels.push(
        channel({
          id: spec.id,
          title: spec.title,
          group: "Herdr",
          subtitle: `${humanSize(st.size)}  ${humanAge(st.mtimeMs)}  ${tilde(full)}`,
          source: full,
          kind: "file",
          meta: { path: full },
        }),
      );
    }
    for (const spec of STATUS_CHANNELS) {
      channels.push(
        channel({
          id: spec.id,
          title: spec.title,
          group: "Herdr",
          subtitle: `${ctx.herdrBin} ${spec.args.join(" ")}`,
          source: `${ctx.herdrBin} ${spec.args.join(" ")}`,
          kind: "command",
          meta: { args: spec.args },
        }),
      );
    }
    if (!channels.some((c) => c.kind === "file")) {
      channels.unshift(
        channel({
          id: "herdr:__missing",
          title: "no herdr logs found",
          group: "Herdr",
          subtitle: tilde(dir),
          source: dir,
          kind: "error",
          meta: { dir },
        }),
      );
    }
    return channels;
  },

  async read(ch, ctx, opts = {}) {
    if (ch.kind === "error") {
      return failure({
        tried: ch.meta.dir,
        reason: "no herdr-server.log / herdr-client.log in the Herdr config directory",
        hint: "start Herdr once, or set HERDR_CONFIG_DIR if it lives elsewhere",
      });
    }
    if (ch.kind === "command") {
      const res = await herdrRun(ch.meta.args, { timeoutMs: ctx.timeoutMs });
      const cmd = ch.source;
      const body = `${res.stdout}${res.stderr}`.trimEnd();
      if (!res.ok && !body) {
        return failure({ tried: cmd, reason: explainFailure(res), source: cmd });
      }
      return result(body || "(no output)", { source: cmd });
    }
    const file = ch.meta.path;
    if (!(await exists(file))) {
      return failure({
        tried: file,
        reason: "file disappeared since the channel list was built",
        hint: "press R to refresh the channel list",
      });
    }
    const tail = await readTail(file, opts.lines || ctx.lines);
    return result(tail.text, {
      source: file,
      truncated: tail.truncated,
      note: `tail ${opts.lines || ctx.lines} lines of ${humanSize(tail.size)}`,
    });
  },
};
