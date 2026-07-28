#!/usr/bin/env node
/**
 * Output Panel — headless actions.
 *
 * Same provider registry as the interactive panel (src/tui.js), so every
 * channel the UI can show is also scriptable:
 *
 *   node src/index.js list-channels [--group MCP] [--json]
 *   node src/index.js read-channel <channel-id> [--lines N] [--source recent]
 *   node src/index.js groups
 *   node src/index.js open-pane [--placement overlay]
 */
import { herdrRun } from "./lib/herdr.js";
import { makeContext, PLUGIN_ID } from "./lib/paths.js";
import { listChannels, readChannel } from "./providers/index.js";

function parseFlags(argv) {
  const flags = {};
  const positional = [];
  for (let i = 0; i < argv.length; i += 1) {
    const token = argv[i];
    if (token.startsWith("--")) {
      const key = token.slice(2);
      const next = argv[i + 1];
      if (next === undefined || next.startsWith("--")) flags[key] = true;
      else {
        flags[key] = next;
        i += 1;
      }
    } else {
      positional.push(token);
    }
  }
  return { flags, positional };
}

function write(obj) {
  process.stdout.write(`${JSON.stringify(obj, null, 2)}\n`);
}

async function cmdListChannels(ctx, flags) {
  let channels = await listChannels(ctx);
  if (typeof flags.group === "string") {
    const want = flags.group.toLowerCase();
    channels = channels.filter((c) => c.group.toLowerCase() === want);
  }
  const groups = {};
  for (const ch of channels) {
    groups[ch.group] = (groups[ch.group] || 0) + 1;
  }
  write({
    ok: true,
    plugin: PLUGIN_ID,
    count: channels.length,
    groups,
    channels: channels.map((c) => ({
      id: c.id,
      group: c.group,
      title: c.title,
      subtitle: c.subtitle,
      source: c.source,
      kind: c.kind,
      provider: c.provider,
    })),
  });
}

async function cmdGroups(ctx) {
  const channels = await listChannels(ctx);
  const groups = {};
  for (const ch of channels) {
    groups[ch.group] = groups[ch.group] || [];
    groups[ch.group].push(ch.id);
  }
  write({ ok: true, groups });
}

async function cmdReadChannel(ctx, positional, flags) {
  const id = positional[0] || process.env.HERDR_PANE_ID;
  if (!id) {
    process.stderr.write(
      "usage: read-channel <channel-id> [--lines N] [--source recent|visible|detection] [--format text|ansi] [--json]\n" +
        "       run `list-channels` for valid ids (pane ids also work directly)\n",
    );
    process.exit(2);
  }
  const channels = await listChannels(ctx);
  // Bare pane ids (wV:p1) stay usable for backwards compatibility.
  const resolved = channels.some((c) => c.id === id) ? id : `pane:${id}`;
  const opts = {};
  if (flags.lines) opts.lines = Number(flags.lines);
  if (typeof flags.source === "string") opts.source = flags.source;
  if (typeof flags.format === "string") opts.format = flags.format;

  const res = await readChannel(channels, resolved, ctx, opts);
  if (flags.json) {
    write({
      ok: !res.error,
      channel: resolved,
      source: res.source,
      note: res.note,
      error: res.error || null,
      truncated: res.truncated,
      lines: res.text ? res.text.split(/\r?\n/).length : 0,
      text: res.text,
    });
  } else {
    if (res.note) process.stdout.write(`# ${res.note}\n`);
    if (res.source) process.stdout.write(`# source: ${res.source}\n`);
    process.stdout.write(res.text.endsWith("\n") ? res.text : `${res.text}\n`);
  }
  process.exit(res.error ? 1 : 0);
}

async function cmdOpenPane(flags) {
  const placement = typeof flags.placement === "string" ? flags.placement : "overlay";
  const args = [
    "plugin",
    "pane",
    "open",
    "--plugin",
    PLUGIN_ID,
    "--entrypoint",
    "output",
    "--placement",
    placement,
    "--focus",
  ];
  if (typeof flags.direction === "string") args.push("--direction", flags.direction);
  const res = await herdrRun(args, { timeoutMs: 10000 });
  process.stdout.write(res.stdout);
  process.stderr.write(res.stderr);
  process.exit(res.code ?? 1);
}

async function main() {
  const [, , action = "list-channels", ...rest] = process.argv;
  const { flags, positional } = parseFlags(rest);
  const ctx = makeContext(flags.lines ? { lines: Number(flags.lines) } : {});

  switch (action) {
    case "list-channels":
      await cmdListChannels(ctx, flags);
      return;
    case "groups":
      await cmdGroups(ctx);
      return;
    case "read-channel":
      await cmdReadChannel(ctx, positional, flags);
      return;
    case "open-pane":
      await cmdOpenPane(flags);
      return;
    default:
      process.stderr.write(
        `unknown action: ${action}\nknown: list-channels, groups, read-channel, open-pane\n`,
      );
      process.exit(2);
  }
}

main().catch((err) => {
  process.stderr.write(`output-panel: ${err?.stack || err?.message || err}\n`);
  process.exit(1);
});
