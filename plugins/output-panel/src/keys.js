/**
 * Keymap.
 *
 * The tables are the single source of truth for dispatch, the footer hints and
 * the help overlay, so those three can never drift apart.
 */

/** @type {Array<{keys: string, action: string, desc: string, footer?: boolean}>} */
export const LIST_KEYS = [
  { keys: "j/k ↑/↓", action: "move", desc: "move selection", footer: true },
  { keys: "enter", action: "open", desc: "read selected channel", footer: true },
  { keys: "space", action: "toggleGroup", desc: "collapse/expand group", footer: true },
  { keys: "[ ]", action: "collapseAll", desc: "collapse all / expand all" },
  { keys: "g/G", action: "top", desc: "first / last row" },
  { keys: "pgup/pgdn", action: "pageUp", desc: "page the list" },
  { keys: "/", action: "filter", desc: "filter channels", footer: true },
  { keys: "esc", action: "clearFilter", desc: "clear filter" },
  { keys: "t", action: "tail", desc: "open in follow mode", footer: true },
  { keys: "f", action: "focusPane", desc: "focus this pane in Herdr", footer: true },
  { keys: "o", action: "openCwd", desc: "open cwd with xdg-open" },
  { keys: "e", action: "editCwd", desc: "open cwd in $EDITOR" },
  { keys: "s", action: "openSession", desc: "open agent session in $PAGER" },
  { keys: "R", action: "refresh", desc: "rebuild the channel list", footer: true },
  { keys: "?", action: "help", desc: "help", footer: true },
  { keys: "q", action: "quit", desc: "quit", footer: true },
];

/** @type {Array<{keys: string, action: string, desc: string, footer?: boolean}>} */
export const OUTPUT_KEYS = [
  { keys: "j/k ↑/↓", action: "move", desc: "scroll a line", footer: true },
  { keys: "pgup/pgdn", action: "pageUp", desc: "scroll a page", footer: true },
  { keys: "g/G", action: "top", desc: "top / bottom", footer: true },
  { keys: "r", action: "refresh", desc: "re-read the channel", footer: true },
  { keys: "t", action: "tail", desc: "toggle follow", footer: true },
  { keys: "/", action: "search", desc: "search in output", footer: true },
  { keys: "n/N", action: "searchNext", desc: "next / previous match", footer: true },
  { keys: "a", action: "cycleSource", desc: "cycle pane snapshot source" },
  { keys: "A", action: "toggleAnsi", desc: "toggle ANSI passthrough (panes)" },
  { keys: "w", action: "export", desc: "write buffer to a temp file" },
  { keys: "E", action: "openEditor", desc: "export and open in $EDITOR" },
  { keys: "P", action: "openPager", desc: "export and open in $PAGER" },
  { keys: "y", action: "copyPath", desc: "copy export path to clipboard" },
  { keys: "b/esc", action: "back", desc: "back to channels", footer: true },
  { keys: "?", action: "help", desc: "help" },
  { keys: "q", action: "quit", desc: "quit", footer: true },
];

export function footerHints(view) {
  const table = view === "output" ? OUTPUT_KEYS : LIST_KEYS;
  return table
    .filter((k) => k.footer)
    .map((k) => `${k.keys} ${k.desc}`)
    .join("  ·  ");
}

/**
 * Map a keypress to an action name for the given view.
 * Returns null when the key is not bound.
 */
export function resolveKey(view, str, key = {}) {
  if (key.ctrl && key.name === "c") return "quit";
  if (view === "help") return "back";

  if (view === "output") {
    if (key.name === "down" || str === "j") return "lineDown";
    if (key.name === "up" || str === "k") return "lineUp";
    if (key.name === "pagedown" || (key.ctrl && key.name === "d")) return "pageDown";
    if (key.name === "pageup" || (key.ctrl && key.name === "u")) return "pageUp";
    if (str === "g") return "top";
    if (str === "G") return "bottom";
    if (key.name === "home") return "top";
    if (key.name === "end") return "bottom";
    if (str === "r") return "refresh";
    if (str === "t") return "tail";
    if (str === "/") return "search";
    if (str === "n") return "searchNext";
    if (str === "N") return "searchPrev";
    if (str === "a") return "cycleSource";
    if (str === "A") return "toggleAnsi";
    if (str === "w") return "export";
    if (str === "E") return "openEditor";
    if (str === "P") return "openPager";
    if (str === "y") return "copyPath";
    if (str === "?") return "help";
    if (key.name === "escape") return "escape";
    if (str === "b" || str === "h" || key.name === "left") return "back";
    if (str === "q") return "quit";
    return null;
  }

  if (key.name === "down" || str === "j") return "moveDown";
  if (key.name === "up" || str === "k") return "moveUp";
  if (key.name === "pagedown" || (key.ctrl && key.name === "d")) return "pageDown";
  if (key.name === "pageup" || (key.ctrl && key.name === "u")) return "pageUp";
  if (str === "g" || key.name === "home") return "top";
  if (str === "G" || key.name === "end") return "bottom";
  if (key.name === "return" || key.name === "enter" || str === "l" || key.name === "right") {
    return "open";
  }
  if (str === " " || key.name === "space") return "toggleGroup";
  if (str === "[") return "collapseAll";
  if (str === "]") return "expandAll";
  if (str === "/") return "filter";
  if (key.name === "escape") return "clearFilter";
  if (str === "t") return "tail";
  if (str === "f") return "focusPane";
  if (str === "o") return "openCwd";
  if (str === "e") return "editCwd";
  if (str === "s") return "openSession";
  if (str === "R" || str === "r") return "refresh";
  if (str === "?") return "help";
  if (str === "q") return "quit";
  return null;
}
