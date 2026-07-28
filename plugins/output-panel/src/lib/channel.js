/**
 * Channel and read-result constructors shared by every provider.
 *
 * A channel is a description of where output comes from; a result is the
 * output itself plus enough provenance to explain an empty or failed read.
 */
import { redactText } from "./redact.js";

export const GROUPS = [
  "Panes",
  "Herdr",
  "Plugins",
  "MCP",
  "Network",
  "System",
  "Agents",
  "Custom",
];

/**
 * @param {object} spec
 * @param {string} spec.id      stable channel id, namespaced (`mcp:server:kater`)
 * @param {string} spec.title   short label for the sidebar
 * @param {string} spec.group   one of GROUPS
 * @param {string} spec.source  the exact path or command behind this channel
 */
export function channel(spec) {
  return {
    id: spec.id,
    title: spec.title,
    group: spec.group,
    subtitle: spec.subtitle || "",
    source: spec.source || "",
    kind: spec.kind || "text",
    meta: spec.meta || {},
  };
}

/** Successful (or partially successful) read. Text is redacted here, once. */
export function result(text, opts = {}) {
  return {
    text: opts.raw ? String(text ?? "") : redactText(String(text ?? "")),
    ansi: Boolean(opts.ansi),
    source: opts.source || "",
    note: opts.note || "",
    error: opts.error || "",
    truncated: Boolean(opts.truncated),
  };
}

/**
 * Failed read. `tried` is the path or command attempted and `hint` is the
 * concrete next step — an empty state should always teach.
 */
export function failure({ tried, reason, hint = "", source = "" }) {
  const body = [
    reason ? redactText(reason) : "",
    tried ? `tried: ${tried}` : "",
    hint ? `hint: ${hint}` : "",
  ]
    .filter(Boolean)
    .join("\n");
  return {
    text: body,
    ansi: false,
    source: source || tried || "",
    note: "",
    error: reason || "unavailable",
    truncated: false,
  };
}
