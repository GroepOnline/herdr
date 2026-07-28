/**
 * Frame rendering.
 *
 * `renderFrame` returns one string for the whole screen. The caller writes it
 * in a single `write()` after homing the cursor, and every line ends with
 * erase-to-end-of-line, so nothing is ever cleared-then-redrawn and the pane
 * does not flicker.
 */
import {
  bold,
  clip,
  colorByLevel,
  cyan,
  dim,
  displayWidth,
  grey,
  highlight,
  inverse,
  levelOf,
  pad,
  red,
  yellow,
} from "./format.js";
import { footerHints, LIST_KEYS, OUTPUT_KEYS } from "./keys.js";
import { buildRows, clampListViewport, selectedRowIndex } from "./state.js";

const CHROME_ROWS = 4; // header, rule, status, footer

export function bodyHeight(rows) {
  return Math.max(1, rows - CHROME_ROWS);
}

export function renderFrame(state, cols, rows) {
  const height = bodyHeight(rows);
  const lines = [];
  lines.push(header(state, cols));
  lines.push(dim("─".repeat(cols)));
  const body =
    state.view === "help"
      ? helpBody(height, cols)
      : state.view === "output"
        ? outputBody(state, height, cols)
        : listBody(state, height, cols);
  for (let i = 0; i < height; i += 1) lines.push(body[i] ?? "");
  lines.push(statusLine(state, cols));
  lines.push(dim(clip(footerHints(state.view === "help" ? "list" : state.view), cols)));

  return `\x1b[H${lines.map((l) => `${clip(l, cols)}\x1b[K`).join("\r\n")}\x1b[J`;
}

function header(state, cols) {
  if (state.view === "help") return bold(clip("OUTPUT PANEL · help", cols));
  if (state.view === "output") {
    const o = state.output;
    const total = o.lines.length;
    const pos = total ? `${Math.min(total, o.top + 1)}-${Math.min(total, o.top + bodyHeightGuess(state))}/${total}` : "0/0";
    const bits = [
      bold("OUTPUT"),
      o.title,
      dim(pos),
      state.follow ? inverse(" FOLLOW ") : "",
      o.search ? cyan(`/${o.search} ${o.matches.length ? `${o.matchIdx + 1}/${o.matches.length}` : "0/0"}`) : "",
      o.paneSource && o.channelId?.startsWith("pane:")
        ? dim(`src=${o.paneSource}${o.paneFormat === "ansi" ? " ansi" : ""}`)
        : "",
      o.truncated ? yellow("truncated") : "",
    ].filter(Boolean);
    return clip(bits.join("  "), cols);
  }
  const visible = buildRows(state).filter((r) => r.type === "channel").length;
  const bits = [
    bold("OUTPUT PANEL"),
    dim(`${visible}/${state.channels.length} channels`),
    state.filter ? cyan(`filter:${state.filter}`) : "",
    state.follow ? inverse(" FOLLOW ") : "",
  ].filter(Boolean);
  return clip(bits.join("  "), cols);
}

// Header is rendered before we know the exact body height; this mirrors it.
function bodyHeightGuess(state) {
  return state.lastBodyHeight || 1;
}

function listBody(state, height, cols) {
  const rows = buildRows(state);
  state.lastBodyHeight = height;
  if (!rows.length) {
    return [
      dim(
        state.filter
          ? `no channels match "${state.filter}" — press esc to clear the filter`
          : "no channels — press R to refresh, or check that the Herdr server is running",
      ),
    ];
  }
  clampListViewport(state, rows, height);
  const selected = selectedRowIndex(rows, state);
  const out = [];
  for (let i = state.listTop; i < Math.min(rows.length, state.listTop + height); i += 1) {
    const row = rows[i];
    const isSelected = i === selected;
    out.push(renderRow(row, isSelected, cols));
  }
  return out;
}

function renderRow(row, isSelected, cols) {
  if (row.type === "group") {
    const marker = row.collapsed ? "+" : "-";
    const text = pad(`${marker} ${row.group.toUpperCase()} (${row.count})`, cols);
    return isSelected ? inverse(text) : bold(text);
  }
  const ch = row.channel;
  const flag = ch.meta?.focused ? "*" : ch.kind === "error" ? "!" : " ";
  const title = clip(ch.title, Math.max(10, Math.floor(cols * 0.45)));
  const left = `  ${flag} ${title}`;
  const gap = Math.max(1, Math.floor(cols * 0.45) + 5 - displayWidth(left));
  const subtitle = ch.subtitle || ch.source || "";
  const line = pad(`${left}${" ".repeat(gap)}${subtitle}`, cols);
  if (isSelected) return inverse(clip(line, cols));
  if (ch.kind === "error") return red(clip(line, cols));
  return clip(`${left}${" ".repeat(gap)}${grey(subtitle)}`, cols);
}

function outputBody(state, height, cols) {
  const o = state.output;
  state.lastBodyHeight = height;
  if (!o.lines.length) {
    const msg = o.error
      ? [red(o.error), "", ...(o.note ? [o.note] : []), o.source ? dim(`source: ${o.source}`) : ""]
      : [
          dim(o.note || "this channel produced no output"),
          "",
          o.source ? dim(`source: ${o.source}`) : "",
          dim("press r to re-read, t to follow, b to go back"),
        ];
    return msg.filter((l) => l !== "");
  }
  const slice = o.lines.slice(o.top, o.top + height);
  const currentMatch = o.matchIdx >= 0 ? o.matches[o.matchIdx] : -1;
  return slice.map((line, i) => {
    const absolute = o.top + i;
    if (o.ansi) return clip(line, cols);
    let text = colorByLevel(line, levelOf(line));
    if (o.search) text = highlight(text, o.search);
    if (absolute === currentMatch) text = `${inverse(">")}${text}`;
    return clip(text, cols);
  });
}

function helpBody(height, cols) {
  const out = [bold("channel list"), ""];
  for (const k of LIST_KEYS) out.push(`  ${pad(k.keys, 12)} ${k.desc}`);
  out.push("", bold("output view"), "");
  for (const k of OUTPUT_KEYS) out.push(`  ${pad(k.keys, 12)} ${k.desc}`);
  out.push(
    "",
    dim("channels are grouped by source: panes, herdr, plugins, mcp, network, system, agents, custom"),
    dim("secrets matching token/key/password patterns are masked before anything is displayed"),
    "",
    dim("press any key to close"),
  );
  return out.slice(0, height).map((l) => clip(l, cols));
}

function statusLine(state, cols) {
  if (state.input) {
    return clip(`${cyan(state.input.prompt)}${state.input.value}${inverse(" ")}`, cols);
  }
  if (state.busy) return clip(dim("working…"), cols);
  if (!state.status) {
    return dim("─".repeat(cols));
  }
  const text = clip(state.status, cols);
  if (state.statusLevel === "error") return red(text);
  if (state.statusLevel === "warn") return yellow(text);
  return dim(text);
}
