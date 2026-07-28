/**
 * Live Herdr panes — the terminal output of every pane in the session.
 *
 * list: `herdr pane list`
 * read: `herdr pane read <id> --source <s> --lines <n> --format <text|ansi>`
 */
import { channel, failure, result } from "../lib/channel.js";
import { explainFailure } from "../lib/exec.js";
import { herdrJson, herdrRun } from "../lib/herdr.js";
import { tilde } from "../lib/paths.js";

export const PANE_SOURCES = ["recent", "visible", "recent-unwrapped", "detection"];

const PREFIX = "pane:";

export const panesProvider = {
  id: "panes",
  group: "Panes",

  async list(ctx) {
    const { result: data, res } = await herdrJson(["pane", "list"], {
      timeoutMs: ctx.timeoutMs,
    });
    if (!data) {
      return [
        channel({
          id: "pane:__unavailable",
          title: "panes unavailable",
          group: "Panes",
          subtitle: res.missing ? "herdr not on PATH" : "herdr pane list failed",
          source: `${ctx.herdrBin} pane list`,
          kind: "error",
          meta: { error: explainFailure(res, "is the Herdr server running?") },
        }),
      ];
    }
    const panes = data.panes || [];
    return panes.map((p) => {
      const title = p.terminal_title_stripped || p.terminal_title || p.pane_id;
      const agent = p.agent ? `${p.agent}/${p.agent_status || "unknown"}` : p.agent_status || "shell";
      return channel({
        id: `${PREFIX}${p.pane_id}`,
        title: `${p.pane_id}  ${title}`,
        group: "Panes",
        subtitle: `${agent}${p.cwd ? `  ${tilde(p.cwd)}` : ""}`,
        source: `${ctx.herdrBin} pane read ${p.pane_id} --source recent`,
        kind: "pane",
        meta: {
          pane_id: p.pane_id,
          workspace_id: p.workspace_id,
          tab_id: p.tab_id,
          agent: p.agent || null,
          status: p.agent_status || "unknown",
          cwd: p.cwd || null,
          session: p.agent_session?.value || null,
          focused: Boolean(p.focused),
        },
      });
    });
  },

  async read(ch, ctx, opts = {}) {
    if (ch.kind === "error") {
      return failure({
        tried: ch.source,
        reason: ch.meta.error,
        hint: "start Herdr (`herdr`) or check HERDR_BIN_PATH",
      });
    }
    const paneId = ch.meta.pane_id;
    const source = PANE_SOURCES.includes(opts.source) ? opts.source : "recent";
    const format = opts.format === "ansi" ? "ansi" : "text";
    const args = [
      "pane",
      "read",
      paneId,
      "--source",
      source,
      "--lines",
      String(opts.lines || ctx.lines),
      "--format",
      format,
    ];
    const res = await herdrRun(args, { timeoutMs: ctx.timeoutMs });
    const cmd = `${ctx.herdrBin} ${args.join(" ")}`;
    if (!res.ok) {
      return failure({
        tried: cmd,
        reason: explainFailure(res, "the pane may have closed; press R to refresh the list"),
        source: cmd,
      });
    }
    const text = res.stdout.trimEnd();
    if (!text) {
      return result("", {
        source: cmd,
        note: `pane ${paneId} produced no output for --source ${source}`,
      });
    }
    return result(text, { ansi: format === "ansi", source: cmd, note: `--source ${source}` });
  },
};
