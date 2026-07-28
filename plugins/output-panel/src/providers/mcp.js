/**
 * MCP output.
 *
 * Cursor writes one log per MCP server into the current session log directory
 * (`~/.config/Cursor/logs/<stamp>/mcp-server-<name>.log`), plus a transport
 * log (`mcpprocess.log`) and per-window extension-host channels under
 * `window<N>/exthost/anysphere.cursor-mcp/` named `MCP Logs.<id>.log`.
 *
 * Configured servers come from `~/.cursor/mcp.json`; its `env` and `headers`
 * values are masked. stdio wrapper logs under /tmp and the local Kater
 * gateway are folded into the same group.
 */
import path from "node:path";

import { channel, failure, result } from "../lib/channel.js";
import {
  exists,
  findFiles,
  humanAge,
  humanSize,
  listDir,
  newestDirs,
  readJson,
  readTail,
  statOrNull,
} from "../lib/files.js";
import { cursorLogsRoot, cursorMcpConfig, tilde } from "../lib/paths.js";
import { redactedJsonString } from "../lib/redact.js";
import { LOCAL_SERVICES, probeService } from "../lib/probe.js";

const SERVER_LOG_RE = /^mcp-server-(.+)\.log$/;

async function loadMcpConfig() {
  const file = cursorMcpConfig();
  if (!(await exists(file))) return { file, servers: {}, missing: true };
  try {
    const data = await readJson(file);
    return { file, servers: data?.mcpServers || {}, raw: data, missing: false };
  } catch (err) {
    return { file, servers: {}, missing: false, error: String(err?.message ?? err) };
  }
}

/** `user-kater` -> mcp.json key `kater`; longest key wins. */
function matchConfigKey(logName, keys) {
  const exact = keys.find((k) => logName === `user-${k}` || logName === k);
  if (exact) return exact;
  const sorted = [...keys].sort((a, b) => b.length - a.length);
  return sorted.find((k) => logName.endsWith(`-${k}`)) || null;
}

export const mcpProvider = {
  id: "mcp",
  group: "MCP",

  async list(ctx) {
    const channels = [];
    const cfg = await loadMcpConfig();
    const keys = Object.keys(cfg.servers);

    channels.push(
      channel({
        id: "mcp:config",
        title: "mcp.json (redacted)",
        group: "MCP",
        subtitle: cfg.missing
          ? `not found: ${tilde(cfg.file)}`
          : `${keys.length} configured servers  ${tilde(cfg.file)}`,
        source: cfg.file,
        kind: "mcp-config",
        meta: { path: cfg.file, missing: cfg.missing },
      }),
    );

    const sessions = await newestDirs(cursorLogsRoot(), 6);
    channels.push(
      channel({
        id: "mcp:sessions",
        title: "cursor log sessions",
        group: "MCP",
        subtitle: sessions.length
          ? `${sessions.length} session dirs, newest ${humanAge(sessions[0].mtimeMs)}`
          : `none under ${tilde(cursorLogsRoot())}`,
        source: cursorLogsRoot(),
        kind: "mcp-sessions",
        meta: { root: cursorLogsRoot() },
      }),
    );

    const newest = sessions[0] || null;
    const seenKeys = new Set();

    if (newest) {
      const entries = await listDir(newest.path);
      const logs = entries
        .filter((e) => e.isFile() && SERVER_LOG_RE.test(e.name))
        .map((e) => ({ name: SERVER_LOG_RE.exec(e.name)[1], file: path.join(newest.path, e.name) }))
        .sort((a, b) => a.name.localeCompare(b.name));

      for (const log of logs) {
        const st = await statOrNull(log.file);
        const key = matchConfigKey(log.name, keys);
        if (key) seenKeys.add(key);
        channels.push(
          channel({
            id: `mcp:server:${log.name}`,
            title: log.name,
            group: "MCP",
            subtitle: [
              key ? `configured as "${key}"` : "cursor-managed",
              st ? `${humanSize(st.size)}  ${humanAge(st.mtimeMs)}` : "",
            ]
              .filter(Boolean)
              .join("  "),
            source: log.file,
            kind: "mcp-server",
            meta: { path: log.file, configKey: key, server: log.name },
          }),
        );
      }

      const processLog = path.join(newest.path, "mcpprocess.log");
      if (await exists(processLog)) {
        const st = await statOrNull(processLog);
        channels.push(
          channel({
            id: "mcp:process",
            title: "mcp process transport",
            group: "MCP",
            subtitle: `${st ? `${humanSize(st.size)}  ${humanAge(st.mtimeMs)}  ` : ""}${tilde(processLog)}`,
            source: processLog,
            kind: "file",
            meta: { path: processLog },
          }),
        );
      }

      const exthost = await findFiles(newest.path, (name) => name.startsWith("MCP Logs."), {
        maxDepth: 4,
        maxFiles: 40,
      });
      for (const file of exthost.slice(0, 6)) {
        const window = path.relative(newest.path, file.path).split(path.sep)[0];
        channels.push(
          channel({
            id: `mcp:exthost:${window}`,
            title: `exthost MCP · ${window}`,
            group: "MCP",
            subtitle: `${humanSize(file.size)}  ${humanAge(file.mtimeMs)}`,
            source: file.path,
            kind: "file",
            meta: { path: file.path },
          }),
        );
      }
    }

    // Configured servers Cursor has not (yet) logged: still show their config.
    for (const key of keys) {
      if (seenKeys.has(key)) continue;
      channels.push(
        channel({
          id: `mcp:server:user-${key}`,
          title: key,
          group: "MCP",
          subtitle: "configured, no cursor log in the newest session",
          source: cfg.file,
          kind: "mcp-server",
          meta: { path: null, configKey: key, server: key },
        }),
      );
    }

    // stdio wrapper logs that live outside Cursor's log tree.
    const wrappers = await findFiles(
      "/tmp",
      (name) => /(mcp|cdp)/i.test(name) && name.endsWith(".log"),
      { maxDepth: 1, maxFiles: 30 },
    );
    for (const file of wrappers.slice(0, 10)) {
      channels.push(
        channel({
          id: `mcp:wrapper:${file.name.replace(/\.log$/, "")}`,
          title: `wrapper · ${file.name}`,
          group: "MCP",
          subtitle: `${humanSize(file.size)}  ${humanAge(file.mtimeMs)}  ${file.path}`,
          source: file.path,
          kind: "file",
          meta: { path: file.path },
        }),
      );
    }

    channels.push(
      channel({
        id: "mcp:gateways",
        title: "mcp gateway health",
        group: "MCP",
        subtitle: "kater sse/rest + brave cdp endpoints",
        source: LOCAL_SERVICES.filter((s) => s.mcp)
          .map((s) => s.url)
          .join(" "),
        kind: "mcp-gateways",
        meta: {},
      }),
    );

    return channels;
  },

  async read(ch, ctx, opts = {}) {
    const lines = opts.lines || ctx.lines;

    if (ch.kind === "mcp-config") {
      if (ch.meta.missing) {
        return failure({
          tried: ch.meta.path,
          reason: "no Cursor MCP config on this machine",
          hint: "Cursor writes ~/.cursor/mcp.json once you configure an MCP server",
        });
      }
      try {
        const data = await readJson(ch.meta.path);
        return result(redactedJsonString(data), {
          raw: true,
          source: ch.meta.path,
          note: "env and header values are masked; structure is preserved",
        });
      } catch (err) {
        return failure({ tried: ch.meta.path, reason: String(err?.message ?? err) });
      }
    }

    if (ch.kind === "mcp-sessions") {
      const sessions = await newestDirs(ch.meta.root, 20);
      if (!sessions.length) {
        return failure({
          tried: ch.meta.root,
          reason: "no Cursor log session directories",
          hint: "run Cursor once; it creates ~/.config/Cursor/logs/<timestamp>/",
        });
      }
      const body = [];
      for (const s of sessions) {
        const entries = await listDir(s.path);
        const servers = entries.filter((e) => SERVER_LOG_RE.test(e.name)).length;
        body.push(
          `${s.name}  ${humanAge(s.mtimeMs).padEnd(10)} ${String(servers).padStart(3)} mcp server logs  ${s.path}`,
        );
      }
      return result(body.join("\n"), {
        source: ch.meta.root,
        note: "newest first; the panel reads MCP logs from the newest session",
      });
    }

    if (ch.kind === "mcp-gateways") {
      const svcs = LOCAL_SERVICES.filter((s) => s.mcp);
      const rows = await Promise.all(svcs.map((s) => probeService(s, 1500)));
      return result(
        [
          "MCP gateway / transport endpoints",
          "",
          ...rows,
          "",
          "kater sse  = Kater MCP gateway (profiles, chains, PR gate)",
          "brave cdp  = DevTools endpoint used by chrome-devtools MCP (read-only /json/version)",
        ].join("\n"),
        { source: svcs.map((s) => s.url).join(" ") },
      );
    }

    if (ch.kind === "mcp-server") {
      const cfg = await loadMcpConfig();
      const header = [];
      if (ch.meta.configKey && cfg.servers[ch.meta.configKey]) {
        header.push(`# ~/.cursor/mcp.json → mcpServers.${ch.meta.configKey}`);
        header.push(redactedJsonString(cfg.servers[ch.meta.configKey]));
        header.push("");
      }
      if (!ch.meta.path) {
        return result(
          [
            ...header,
            `no Cursor log for this server in the newest session directory`,
            `tried: ${tilde(cursorLogsRoot())}/<newest>/mcp-server-user-${ch.meta.configKey}.log`,
            `hint: Cursor only creates the log once the server starts; open a chat that uses it`,
          ].join("\n"),
          { raw: true, source: cfg.file, note: "config only" },
        );
      }
      if (!(await exists(ch.meta.path))) {
        return failure({
          tried: ch.meta.path,
          reason: "log file gone (Cursor rotated its log session)",
          hint: "press R to rebuild the channel list against the newest session",
        });
      }
      const tail = await readTail(ch.meta.path, lines);
      return result([...header, tail.text].join("\n"), {
        source: ch.meta.path,
        truncated: tail.truncated,
        note: `tail ${lines} lines of ${humanSize(tail.size)}`,
      });
    }

    // plain file channels (mcpprocess, exthost, /tmp wrappers)
    if (!(await exists(ch.meta.path))) {
      return failure({
        tried: ch.meta.path,
        reason: "file no longer exists",
        hint: "press R to refresh the channel list",
      });
    }
    const tail = await readTail(ch.meta.path, lines);
    return result(tail.text, {
      source: ch.meta.path,
      truncated: tail.truncated,
      note: `tail ${lines} lines of ${humanSize(tail.size)}`,
    });
  },
};
