/**
 * Panel state and the pure transforms over it.
 *
 * No I/O and no rendering here: this module decides which rows exist, what is
 * selected and where the viewport sits, which keeps it trivially testable.
 */
import { PANE_SOURCES } from "./providers/panes.js";
import { stripAnsi } from "./format.js";

export const MAX_BUFFER_LINES = 5000;
export const DEFAULT_FOLLOW_MS = 2000;

export function createState() {
  return {
    channels: [],
    collapsed: new Set(),
    selectedId: null,
    filter: "",
    view: "list", // list | output | help
    status: "",
    statusLevel: "", // "" | "error" | "warn"
    busy: false,
    listTop: 0,
    follow: false,
    followMs: DEFAULT_FOLLOW_MS,
    input: null, // { prompt, value, target: "filter" | "search" }
    exportPath: "",
    output: {
      channelId: null,
      title: "",
      source: "",
      note: "",
      error: "",
      lines: [],
      ansi: false,
      top: 0,
      truncated: false,
      search: "",
      matches: [],
      matchIdx: -1,
      paneSource: "recent",
      paneFormat: "text",
      readAt: 0,
    },
  };
}

export function setStatus(state, text, level = "") {
  state.status = text || "";
  state.statusLevel = level;
}

function matchesFilter(ch, query) {
  if (!query) return true;
  return [ch.id, ch.title, ch.subtitle, ch.group, ch.source]
    .filter(Boolean)
    .join(" ")
    .toLowerCase()
    .includes(query);
}

/**
 * Flatten channels into renderable rows: a header per non-empty group followed
 * by its channels, unless the group is collapsed.
 */
export function buildRows(state) {
  const query = state.filter.trim().toLowerCase();
  const rows = [];
  const seen = [];
  for (const ch of state.channels) {
    if (!matchesFilter(ch, query)) continue;
    if (!seen.includes(ch.group)) seen.push(ch.group);
  }
  for (const group of seen) {
    const items = state.channels.filter((c) => c.group === group && matchesFilter(c, query));
    const collapsed = state.collapsed.has(group);
    rows.push({ type: "group", group, count: items.length, collapsed });
    if (collapsed) continue;
    for (const ch of items) rows.push({ type: "channel", channel: ch, group });
  }
  return rows;
}

export function selectedRowIndex(rows, state) {
  if (!state.selectedId) return rows.findIndex((r) => r.type === "channel");
  const idx = rows.findIndex((r) => r.type === "channel" && r.channel.id === state.selectedId);
  if (idx >= 0) return idx;
  const groupIdx = rows.findIndex((r) => r.type === "group" && r.group === state.selectedId);
  if (groupIdx >= 0) return groupIdx;
  return rows.findIndex((r) => r.type === "channel");
}

export function rowKey(row) {
  return row.type === "group" ? row.group : row.channel.id;
}

/** Move the selection by `delta` rows, skipping nothing (groups are selectable). */
export function moveSelection(state, delta) {
  const rows = buildRows(state);
  if (!rows.length) return;
  const current = selectedRowIndex(rows, state);
  const next = Math.min(rows.length - 1, Math.max(0, (current < 0 ? 0 : current) + delta));
  state.selectedId = rowKey(rows[next]);
}

export function selectEdge(state, edge) {
  const rows = buildRows(state);
  if (!rows.length) return;
  state.selectedId = rowKey(rows[edge === "top" ? 0 : rows.length - 1]);
}

export function selectedChannel(state) {
  const rows = buildRows(state);
  const idx = selectedRowIndex(rows, state);
  const row = rows[idx];
  return row && row.type === "channel" ? row.channel : null;
}

export function selectedGroup(state) {
  const rows = buildRows(state);
  const idx = selectedRowIndex(rows, state);
  const row = rows[idx];
  if (!row) return null;
  return row.type === "group" ? row.group : row.group;
}

/** Keep the selected row inside the visible window. */
export function clampListViewport(state, rows, height) {
  const idx = selectedRowIndex(rows, state);
  if (idx < 0) {
    state.listTop = 0;
    return;
  }
  if (idx < state.listTop) state.listTop = idx;
  if (idx >= state.listTop + height) state.listTop = idx - height + 1;
  state.listTop = Math.max(0, Math.min(state.listTop, Math.max(0, rows.length - height)));
}

/** Replace the output buffer, honouring the line cap. */
export function setOutputLines(state, lines) {
  const capped = lines.length > MAX_BUFFER_LINES ? lines.slice(-MAX_BUFFER_LINES) : lines;
  state.output.lines = capped;
  state.output.truncated = state.output.truncated || lines.length > MAX_BUFFER_LINES;
  recomputeMatches(state);
}

export function scrollOutput(state, delta, height) {
  const max = Math.max(0, state.output.lines.length - height);
  state.output.top = Math.min(max, Math.max(0, state.output.top + delta));
}

export function scrollOutputTo(state, where, height) {
  const max = Math.max(0, state.output.lines.length - height);
  state.output.top = where === "top" ? 0 : max;
}

export function isAtBottom(state, height) {
  const max = Math.max(0, state.output.lines.length - height);
  return state.output.top >= max;
}

export function recomputeMatches(state) {
  const q = state.output.search.trim().toLowerCase();
  if (!q) {
    state.output.matches = [];
    state.output.matchIdx = -1;
    return;
  }
  const matches = [];
  state.output.lines.forEach((line, i) => {
    if (stripAnsi(line).toLowerCase().includes(q)) matches.push(i);
  });
  state.output.matches = matches;
  state.output.matchIdx = matches.length ? 0 : -1;
}

export function jumpMatch(state, delta, height) {
  const { matches } = state.output;
  if (!matches.length) return false;
  state.output.matchIdx = (state.output.matchIdx + delta + matches.length) % matches.length;
  const line = matches[state.output.matchIdx];
  state.output.top = Math.max(0, Math.min(line - Math.floor(height / 2), Math.max(0, state.output.lines.length - height)));
  return true;
}

export function cyclePaneSource(state) {
  const i = PANE_SOURCES.indexOf(state.output.paneSource);
  state.output.paneSource = PANE_SOURCES[(i + 1) % PANE_SOURCES.length];
  return state.output.paneSource;
}
