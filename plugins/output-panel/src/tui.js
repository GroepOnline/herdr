#!/usr/bin/env node
/**
 * Output Panel — a universal output/log browser for Herdr.
 *
 * Channels come from the provider registry (panes, herdr logs, plugin logs,
 * plugin state, MCP, network, systemd, agent sessions, user config). This file
 * owns only the terminal: input, frame writes, terminal restoration and the
 * side effects (focus a pane, open a path, export a buffer).
 *
 * With no TTY on stdin it dumps the channel catalogue as JSON and exits, which
 * is what smoke tests use.
 */
import { spawn } from "node:child_process";
import { writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import readline from "node:readline";
import { stdin as input, stdout as output } from "node:process";

import { stripAnsi } from "./format.js";
import { herdrRpc } from "./lib/herdr.js";
import { makeContext } from "./lib/paths.js";
import { resolveKey } from "./keys.js";
import { bodyHeight, renderFrame } from "./render.js";
import { listChannels, readChannel } from "./providers/index.js";
import {
  createState,
  cyclePaneSource,
  isAtBottom,
  jumpMatch,
  moveSelection,
  recomputeMatches,
  scrollOutput,
  scrollOutputTo,
  selectEdge,
  selectedChannel,
  selectedGroup,
  setOutputLines,
  setStatus,
} from "./state.js";

const ctx = makeContext();
const state = createState();

let followTimer = null;
let rendering = false;
let suspended = false;

/* ---------------------------------------------------------------- terminal */

function size() {
  return { cols: output.columns || 100, rows: output.rows || 30 };
}

function paint() {
  if (suspended || rendering) return;
  rendering = true;
  const { cols, rows } = size();
  try {
    output.write(renderFrame(state, cols, rows));
  } finally {
    rendering = false;
  }
}

function enterTui() {
  readline.emitKeypressEvents(input);
  if (input.isTTY) input.setRawMode(true);
  output.write("\x1b[?1049h\x1b[?25l");
  suspended = false;
}

function leaveTui() {
  output.write("\x1b[?25h\x1b[?1049l");
  try {
    if (input.isTTY) input.setRawMode(false);
  } catch {
    // terminal already gone
  }
}

function stopFollow() {
  if (followTimer) {
    clearInterval(followTimer);
    followTimer = null;
  }
  state.follow = false;
}

let exiting = false;
function cleanup(code = 0) {
  if (exiting) return;
  exiting = true;
  stopFollow();
  leaveTui();
  output.write("\n");
  process.exit(code);
}

/* ---------------------------------------------------------------- channels */

async function refreshChannels(reason = "") {
  state.busy = true;
  paint();
  try {
    state.channels = await listChannels(ctx);
    if (!state.selectedId && state.channels.length) state.selectedId = state.channels[0].id;
    setStatus(state, reason || `${state.channels.length} channels`);
  } catch (err) {
    setStatus(state, `channel list failed: ${err?.message ?? err}`, "error");
  } finally {
    state.busy = false;
    paint();
  }
}

function readOptionsFor(ch) {
  const opts = { lines: ctx.lines };
  if (ch.id.startsWith("pane:")) {
    opts.source = state.output.paneSource;
    opts.format = state.output.paneFormat;
  }
  return opts;
}

async function openChannel(ch, { keepScroll = false } = {}) {
  if (!ch) return;
  const { rows } = size();
  const height = bodyHeight(rows);
  const wasBottom = keepScroll ? isAtBottom(state, height) : true;
  const prevTop = state.output.top;

  state.busy = true;
  state.view = "output";
  state.output.channelId = ch.id;
  state.output.title = `${ch.group} · ${ch.title}`;
  paint();

  const res = await readChannel(state.channels, ch.id, ctx, readOptionsFor(ch));
  state.output.source = res.source || ch.source;
  state.output.note = res.note || "";
  state.output.error = res.error || "";
  state.output.ansi = Boolean(res.ansi);
  state.output.truncated = Boolean(res.truncated);
  state.output.readAt = Date.now();
  const lines = res.text ? res.text.split(/\r?\n/) : [];
  setOutputLines(state, lines);
  recomputeMatches(state);

  if (keepScroll && !wasBottom) {
    state.output.top = Math.min(prevTop, Math.max(0, state.output.lines.length - height));
  } else {
    scrollOutputTo(state, "bottom", height);
  }
  state.busy = false;
  setStatus(
    state,
    res.error ? res.error : `${state.output.lines.length} lines · ${res.source || ch.source}`,
    res.error ? "error" : "",
  );
  paint();
}

function toggleFollow() {
  if (state.follow) {
    stopFollow();
    setStatus(state, "follow off");
    paint();
    return;
  }
  const id = state.output.channelId;
  if (!id) {
    setStatus(state, "open a channel first, then press t", "warn");
    paint();
    return;
  }
  state.follow = true;
  followTimer = setInterval(() => {
    if (state.view !== "output" || !state.follow) return;
    const ch = state.channels.find((c) => c.id === state.output.channelId);
    if (!ch) {
      stopFollow();
      setStatus(state, "channel gone; follow stopped", "warn");
      paint();
      return;
    }
    openChannel(ch, { keepScroll: true }).catch(() => {});
  }, state.followMs);
  followTimer.unref?.();
  setStatus(state, `follow on · polling every ${state.followMs}ms`);
  paint();
}

/* ------------------------------------------------------------ side effects */

async function focusSelectedPane() {
  const ch = selectedChannel(state);
  if (!ch || !ch.id.startsWith("pane:")) {
    setStatus(state, "f only works on a Panes channel", "warn");
    paint();
    return;
  }
  try {
    const resp = await herdrRpc("pane.focus", { pane_id: ch.meta.pane_id });
    if (resp.error) setStatus(state, `focus failed: ${JSON.stringify(resp.error)}`, "error");
    else setStatus(state, `focused ${ch.meta.pane_id}`);
  } catch (err) {
    setStatus(state, `focus failed: ${err?.message ?? err}`, "error");
  }
  await refreshChannels("");
}

function detachedOpen(cmd, args, label) {
  try {
    const child = spawn(cmd, args, { stdio: "ignore", detached: true });
    child.on("error", (err) => {
      setStatus(state, `${label} failed: ${err?.message ?? err}`, "error");
      paint();
    });
    child.unref();
    setStatus(state, `${label}: ${args[args.length - 1]}`);
  } catch (err) {
    setStatus(state, `${label} failed: ${err?.message ?? err}`, "error");
  }
  paint();
}

/** Hand the terminal to an interactive child, then take it back. */
function runInteractive(cmd, args) {
  suspended = true;
  stopFollow();
  leaveTui();
  const child = spawn(cmd, args, { stdio: "inherit" });
  child.on("error", (err) => {
    enterTui();
    suspended = false;
    setStatus(state, `${cmd} failed: ${err?.message ?? err}`, "error");
    paint();
  });
  child.on("exit", () => {
    enterTui();
    suspended = false;
    paint();
  });
}

async function exportBuffer() {
  const id = state.output.channelId || "output";
  const safe = id.replace(/[^\w.-]+/g, "_").slice(0, 60);
  const file = path.join(tmpdir(), `herdr-output-panel-${safe}-${Date.now()}.log`);
  const headerLines = [
    `# channel: ${id}`,
    `# title:   ${state.output.title}`,
    `# source:  ${state.output.source}`,
    `# read at: ${new Date(state.output.readAt || Date.now()).toISOString()}`,
    "",
  ];
  const body = state.output.lines.map(stripAnsi).join("\n");
  await writeFile(file, `${headerLines.join("\n")}${body}\n`, { mode: 0o600 });
  state.exportPath = file;
  return file;
}

async function copyToClipboard(text) {
  const candidates = [
    ["wl-copy", []],
    ["xclip", ["-selection", "clipboard"]],
    ["xsel", ["--clipboard", "--input"]],
  ];
  for (const [cmd, args] of candidates) {
    const ok = await new Promise((resolve) => {
      let child;
      try {
        child = spawn(cmd, args, { stdio: ["pipe", "ignore", "ignore"] });
      } catch {
        resolve(false);
        return;
      }
      child.on("error", () => resolve(false));
      child.on("exit", (code) => resolve(code === 0));
      child.stdin.end(text);
    });
    if (ok) return cmd;
  }
  return null;
}

/* -------------------------------------------------------------- input mode */

function beginInput(target, prompt, initial = "") {
  state.input = { target, prompt, value: initial };
  paint();
}

function handleInputKey(str, key) {
  const inp = state.input;
  if (key.name === "escape" || (key.ctrl && key.name === "c")) {
    state.input = null;
    if (inp.target === "search") {
      state.output.search = "";
      recomputeMatches(state);
    }
    paint();
    return;
  }
  if (key.name === "return" || key.name === "enter") {
    state.input = null;
    commitInput(inp);
    return;
  }
  if (key.name === "backspace") {
    inp.value = inp.value.slice(0, -1);
  } else if (key.ctrl && key.name === "u") {
    inp.value = "";
  } else if (str && !key.ctrl && !key.meta && str >= " " && str !== "\x7f") {
    inp.value += str;
  }
  if (inp.target === "search") {
    state.output.search = inp.value;
    recomputeMatches(state);
    if (state.output.matches.length) jumpMatch(state, 0, bodyHeight(size().rows));
  }
  paint();
}

function commitInput(inp) {
  if (inp.target === "filter") {
    state.filter = inp.value.trim();
    state.listTop = 0;
    setStatus(state, state.filter ? `filter: ${state.filter}` : "filter cleared");
  } else if (inp.target === "search") {
    state.output.search = inp.value.trim();
    recomputeMatches(state);
    const height = bodyHeight(size().rows);
    if (state.output.matches.length) {
      state.output.matchIdx = -1;
      jumpMatch(state, 1, height);
      setStatus(state, `${state.output.matches.length} matches for "${state.output.search}"`);
    } else if (state.output.search) {
      setStatus(state, `no match for "${state.output.search}"`, "warn");
    }
  }
  paint();
}

/* ---------------------------------------------------------------- dispatch */

async function handleListAction(action) {
  const height = bodyHeight(size().rows);
  switch (action) {
    case "moveDown":
      moveSelection(state, 1);
      break;
    case "moveUp":
      moveSelection(state, -1);
      break;
    case "pageDown":
      moveSelection(state, height);
      break;
    case "pageUp":
      moveSelection(state, -height);
      break;
    case "top":
      selectEdge(state, "top");
      break;
    case "bottom":
      selectEdge(state, "bottom");
      break;
    case "toggleGroup": {
      const group = selectedGroup(state);
      if (group) {
        if (state.collapsed.has(group)) state.collapsed.delete(group);
        else state.collapsed.add(group);
      }
      break;
    }
    case "collapseAll":
      for (const ch of state.channels) state.collapsed.add(ch.group);
      break;
    case "expandAll":
      state.collapsed.clear();
      break;
    case "open": {
      const ch = selectedChannel(state);
      if (!ch) {
        setStatus(state, "select a channel (groups toggle with space)", "warn");
        break;
      }
      await openChannel(ch);
      return;
    }
    case "tail": {
      const ch = selectedChannel(state);
      if (!ch) {
        setStatus(state, "select a channel to follow", "warn");
        break;
      }
      await openChannel(ch);
      toggleFollow();
      return;
    }
    case "focusPane":
      await focusSelectedPane();
      return;
    case "openCwd":
    case "editCwd": {
      const ch = selectedChannel(state);
      const cwd = ch?.meta?.cwd;
      if (!cwd) {
        setStatus(state, "this channel has no cwd", "warn");
        break;
      }
      if (action === "openCwd") detachedOpen("xdg-open", [cwd], "xdg-open");
      else detachedOpen(process.env.VISUAL || process.env.EDITOR || "cursor", [cwd], "editor");
      return;
    }
    case "openSession": {
      const ch = selectedChannel(state);
      const session = ch?.meta?.session;
      if (!session) {
        setStatus(state, "no agent session path on this channel", "warn");
        break;
      }
      runInteractive(process.env.PAGER || "less", [session]);
      return;
    }
    case "filter":
      beginInput("filter", "filter> ", state.filter);
      return;
    case "clearFilter":
      state.filter = "";
      setStatus(state, "filter cleared");
      break;
    case "refresh":
      await refreshChannels("channel list refreshed");
      return;
    case "help":
      state.view = "help";
      break;
    case "quit":
      cleanup(0);
      return;
    default:
      return;
  }
  paint();
}

async function handleOutputAction(action) {
  const height = bodyHeight(size().rows);
  switch (action) {
    case "lineDown":
      scrollOutput(state, 1, height);
      break;
    case "lineUp":
      scrollOutput(state, -1, height);
      break;
    case "pageDown":
      scrollOutput(state, height, height);
      break;
    case "pageUp":
      scrollOutput(state, -height, height);
      break;
    case "top":
      scrollOutputTo(state, "top", height);
      break;
    case "bottom":
      scrollOutputTo(state, "bottom", height);
      break;
    case "refresh": {
      const ch = state.channels.find((c) => c.id === state.output.channelId);
      if (ch) {
        await openChannel(ch, { keepScroll: true });
        return;
      }
      setStatus(state, "channel no longer listed; press b then R", "warn");
      break;
    }
    case "tail":
      toggleFollow();
      return;
    case "search":
      beginInput("search", "search> ", state.output.search);
      return;
    case "searchNext":
      if (!jumpMatch(state, 1, height)) setStatus(state, "no matches", "warn");
      break;
    case "searchPrev":
      if (!jumpMatch(state, -1, height)) setStatus(state, "no matches", "warn");
      break;
    case "cycleSource": {
      if (!state.output.channelId?.startsWith("pane:")) {
        setStatus(state, "snapshot sources only apply to pane channels", "warn");
        break;
      }
      const src = cyclePaneSource(state);
      setStatus(state, `pane source: ${src}`);
      const ch = state.channels.find((c) => c.id === state.output.channelId);
      if (ch) {
        await openChannel(ch);
        return;
      }
      break;
    }
    case "toggleAnsi": {
      if (!state.output.channelId?.startsWith("pane:")) {
        setStatus(state, "ANSI passthrough only applies to pane channels", "warn");
        break;
      }
      state.output.paneFormat = state.output.paneFormat === "ansi" ? "text" : "ansi";
      const ch = state.channels.find((c) => c.id === state.output.channelId);
      if (ch) {
        await openChannel(ch);
        return;
      }
      break;
    }
    case "export":
      try {
        const file = await exportBuffer();
        setStatus(state, `wrote ${file}  (y copies the path)`);
      } catch (err) {
        setStatus(state, `export failed: ${err?.message ?? err}`, "error");
      }
      break;
    case "openEditor":
      try {
        const file = await exportBuffer();
        detachedOpen(process.env.VISUAL || process.env.EDITOR || "cursor", [file], "editor");
        return;
      } catch (err) {
        setStatus(state, `export failed: ${err?.message ?? err}`, "error");
      }
      break;
    case "openPager":
      try {
        const file = await exportBuffer();
        runInteractive(process.env.PAGER || "less", ["-R", file]);
        return;
      } catch (err) {
        setStatus(state, `export failed: ${err?.message ?? err}`, "error");
      }
      break;
    case "copyPath": {
      if (!state.exportPath) {
        setStatus(state, "nothing exported yet — press w first", "warn");
        break;
      }
      const via = await copyToClipboard(state.exportPath);
      setStatus(
        state,
        via ? `copied path via ${via}` : `no clipboard tool found · ${state.exportPath}`,
        via ? "" : "warn",
      );
      break;
    }
    case "escape":
      if (state.output.search) {
        state.output.search = "";
        recomputeMatches(state);
        setStatus(state, "search cleared");
        break;
      }
      state.view = "list";
      break;
    case "back":
      state.view = "list";
      break;
    case "help":
      state.view = "help";
      break;
    case "quit":
      cleanup(0);
      return;
    default:
      return;
  }
  paint();
}

function onKey(str, key = {}) {
  if (suspended) return;
  if (state.input) {
    handleInputKey(str, key);
    return;
  }
  const action = resolveKey(state.view, str, key);
  if (!action) return;
  if (state.view === "help") {
    state.view = "list";
    paint();
    return;
  }
  const handler = state.view === "output" ? handleOutputAction : handleListAction;
  handler(action).catch((err) => {
    setStatus(state, `action failed: ${err?.message ?? err}`, "error");
    paint();
  });
}

/* -------------------------------------------------------------------- main */

async function headlessDump() {
  const channels = await listChannels(ctx);
  process.stdout.write(
    `${JSON.stringify(
      {
        ok: true,
        mode: "headless",
        count: channels.length,
        groups: [...new Set(channels.map((c) => c.group))],
        channels: channels.map((c) => ({
          id: c.id,
          title: c.title,
          group: c.group,
          subtitle: c.subtitle,
          source: c.source,
          kind: c.kind,
        })),
      },
      null,
      2,
    )}\n`,
  );
}

async function main() {
  if (!input.isTTY) {
    await headlessDump();
    process.exit(0);
  }

  process.on("SIGINT", () => cleanup(0));
  process.on("SIGTERM", () => cleanup(0));
  process.on("SIGHUP", () => cleanup(0));
  process.on("exit", () => {
    stopFollow();
  });
  output.on("resize", () => paint());
  process.on("SIGWINCH", () => paint());

  enterTui();
  setStatus(state, "loading channels…");
  paint();
  await refreshChannels("");
  input.on("keypress", onKey);
}

main().catch((err) => {
  leaveTui();
  process.stderr.write(`output-panel: ${err?.stack || err?.message || err}\n`);
  process.exit(1);
});
