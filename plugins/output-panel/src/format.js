/**
 * Text presentation: ANSI-aware width handling, log-level colour, highlights.
 *
 * Colour only ever carries state (severity, selection, match). Nothing here is
 * decorative, and NO_COLOR turns all of it off.
 */

const COLOR_ENABLED = !process.env.NO_COLOR && process.env.TERM !== "dumb";

const CODES = {
  reset: "\x1b[0m",
  bold: "\x1b[1m",
  dim: "\x1b[2m",
  inverse: "\x1b[7m",
  red: "\x1b[31m",
  yellow: "\x1b[33m",
  cyan: "\x1b[36m",
  grey: "\x1b[90m",
};

function wrap(code, text) {
  if (!COLOR_ENABLED || !text) return text;
  return `${code}${text}${CODES.reset}`;
}

export const bold = (t) => wrap(CODES.bold, t);
export const dim = (t) => wrap(CODES.dim, t);
export const inverse = (t) => wrap(CODES.inverse, t);
export const red = (t) => wrap(CODES.red, t);
export const yellow = (t) => wrap(CODES.yellow, t);
export const cyan = (t) => wrap(CODES.cyan, t);
export const grey = (t) => wrap(CODES.grey, t);

export const ANSI_RE = /\x1b\[[0-9;?]*[ -/]*[@-~]/g;

export function stripAnsi(text) {
  return String(text).replace(ANSI_RE, "");
}

/** Printable width, ignoring escape sequences. Control chars count as zero. */
export function displayWidth(text) {
  return stripAnsi(text).replace(/[\x00-\x1f\x7f]/g, "").length;
}

/**
 * Clip to `width` printable columns while preserving escape sequences.
 * Emits a reset when the clipped text still had styling open.
 */
export function clip(text, width) {
  if (width <= 0) return "";
  const str = String(text);
  let out = "";
  let visible = 0;
  let sawEscape = false;
  let i = 0;
  while (i < str.length) {
    if (str[i] === "\x1b") {
      ANSI_RE.lastIndex = i;
      const match = ANSI_RE.exec(str);
      if (match && match.index === i) {
        out += match[0];
        sawEscape = true;
        i += match[0].length;
        continue;
      }
    }
    const ch = str[i];
    // Drop other control characters; they corrupt frame geometry.
    if (ch < " " && ch !== "\t") {
      i += 1;
      continue;
    }
    if (visible >= width) break;
    out += ch === "\t" ? " " : ch;
    visible += 1;
    i += 1;
  }
  if (sawEscape && COLOR_ENABLED) out += CODES.reset;
  return out;
}

export function pad(text, width) {
  const w = displayWidth(text);
  return w >= width ? text : text + " ".repeat(width - w);
}

const LEVEL_RULES = [
  [/\b(?:ERROR|ERR|FATAL|CRITICAL|PANIC|FAIL(?:ED|URE)?|EXCEPTION|denied|refused|timed? ?out)\b/i, "error"],
  [/\b(?:WARN(?:ING)?|DEPRECATED|retry|retrying|unavailable|missing)\b/i, "warn"],
  [/\b(?:DEBUG|TRACE|VERBOSE)\b/, "debug"],
  [/\b(?:INFO|NOTICE)\b/, "info"],
];

/** Classify a log line. Returns "error" | "warn" | "info" | "debug" | "". */
export function levelOf(line) {
  const plain = stripAnsi(line);
  for (const [re, level] of LEVEL_RULES) {
    if (re.test(plain)) return level;
  }
  return "";
}

/** Colour a line by severity. Lines that already carry ANSI are left alone. */
export function colorByLevel(line, level) {
  if (!COLOR_ENABLED) return line;
  switch (level) {
    case "error":
      return `${CODES.red}${line}${CODES.reset}`;
    case "warn":
      return `${CODES.yellow}${line}${CODES.reset}`;
    case "debug":
      return `${CODES.grey}${line}${CODES.reset}`;
    default:
      return line;
  }
}

/**
 * Inverse-highlight every case-insensitive occurrence of `query`.
 * Only applied to plain (non-ANSI) lines so it cannot corrupt passthrough.
 */
export function highlight(line, query) {
  if (!COLOR_ENABLED || !query) return line;
  const lower = line.toLowerCase();
  const needle = query.toLowerCase();
  let out = "";
  let from = 0;
  for (;;) {
    const at = lower.indexOf(needle, from);
    if (at < 0) break;
    out += line.slice(from, at) + CODES.inverse + line.slice(at, at + needle.length) + CODES.reset;
    from = at + needle.length;
  }
  return out + line.slice(from);
}

export function hasColor() {
  return COLOR_ENABLED;
}
