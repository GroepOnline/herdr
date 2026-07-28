/**
 * systemd user units and the journal.
 *
 * The unit list is discovered at runtime; only units relevant to this machine's
 * agent/fleet stack get their own channel so the sidebar stays readable.
 */
import { channel, failure, result } from "../lib/channel.js";
import { explainFailure, run } from "../lib/exec.js";

const RELEVANT_UNIT_RE =
  /^(?:pipewire|wireplumber|filter-chain|cursor|kater|ocx|opencodex|herdr|codex|chef|chefbar|chefnotify|joep-ops|moshi|cg-share|ghost-reaper)/i;

const JOURNAL_TIMEOUT_MS = 6000;

/** `unit.service loaded active running Description` */
const UNIT_LINE_RE = /^(\S+\.(?:service|timer|socket))\s+(\S+)\s+(\S+)\s+(\S+)\s*(.*)$/;

async function listUnits(ctx) {
  const res = await run(
    "systemctl",
    ["--user", "list-units", "--type=service,timer", "--all", "--no-pager", "--plain", "--no-legend"],
    { timeoutMs: ctx.timeoutMs },
  );
  if (!res.ok) return { units: [], res };
  const units = [];
  for (const line of res.stdout.split(/\r?\n/)) {
    const m = UNIT_LINE_RE.exec(line.trim().replace(/^●\s*/, ""));
    if (!m) continue;
    units.push({ name: m[1], load: m[2], active: m[3], sub: m[4], description: m[5] });
  }
  return { units, res };
}

export const systemProvider = {
  id: "system",
  group: "System",

  async list(ctx) {
    const channels = [
      channel({
        id: "sys:units",
        title: "systemd user units",
        group: "System",
        subtitle: "systemctl --user list-units",
        source: "systemctl --user list-units --type=service,timer --all",
        kind: "units",
        meta: {},
      }),
      channel({
        id: "sys:journal",
        title: "user journal (all)",
        group: "System",
        subtitle: "journalctl --user -n N",
        source: "journalctl --user -n N --no-pager",
        kind: "journal",
        meta: { unit: null },
      }),
      channel({
        id: "sys:journal-errors",
        title: "user journal (errors)",
        group: "System",
        subtitle: "journalctl --user -p err -n N",
        source: "journalctl --user -p err -n N --no-pager",
        kind: "journal",
        meta: { unit: null, priority: "err" },
      }),
      channel({
        id: "sys:failed",
        title: "failed user units",
        group: "System",
        subtitle: "systemctl --user --failed",
        source: "systemctl --user --failed --no-pager --plain",
        kind: "command",
        meta: {
          argv: ["systemctl", "--user", "--failed", "--no-pager", "--plain"],
          hint: "systemd user session may not be running",
        },
      }),
    ];

    const { units } = await listUnits(ctx);
    for (const unit of units) {
      if (!RELEVANT_UNIT_RE.test(unit.name)) continue;
      channels.push(
        channel({
          id: `sys:unit:${unit.name}`,
          title: unit.name,
          group: "System",
          subtitle: `${unit.active}/${unit.sub}  ${unit.description}`.trim(),
          source: `journalctl --user -u ${unit.name} -n N --no-pager`,
          kind: "journal",
          meta: { unit: unit.name, active: unit.active, sub: unit.sub },
        }),
      );
    }
    return channels;
  },

  async read(ch, ctx, opts = {}) {
    const lines = String(opts.lines || ctx.lines);

    if (ch.kind === "units") {
      const { units, res } = await listUnits(ctx);
      if (!units.length) {
        return failure({
          tried: ch.source,
          reason: explainFailure(res),
          hint: "this needs a systemd user session (loginctl / systemd --user)",
          source: ch.source,
        });
      }
      const body = units
        .map(
          (u) =>
            `${u.active.padEnd(9)} ${u.sub.padEnd(9)} ${u.name.padEnd(46)} ${u.description}`.trimEnd(),
        )
        .join("\n");
      return result(`${units.length} user units\n\n${body}`, { source: ch.source });
    }

    if (ch.kind === "journal") {
      const args = ["--user", "-n", lines, "--no-pager", "--output", "short-iso"];
      if (ch.meta.unit) args.push("-u", ch.meta.unit);
      if (ch.meta.priority) args.push("-p", ch.meta.priority);
      const cmd = `journalctl ${args.join(" ")}`;
      const res = await run("journalctl", args, { timeoutMs: JOURNAL_TIMEOUT_MS });
      const body = res.stdout.trimEnd();
      if (!res.ok && !body) {
        return failure({
          tried: cmd,
          reason: explainFailure(res),
          hint: ch.meta.unit
            ? `check the unit name with \`systemctl --user status ${ch.meta.unit}\``
            : "journalctl needs a systemd user session with a readable journal",
          source: cmd,
        });
      }
      if (!body || /^-- No entries --$/m.test(body)) {
        return result("", {
          source: cmd,
          note: ch.meta.unit
            ? `no journal entries for ${ch.meta.unit}; the unit may never have logged`
            : "journal is empty for this range",
        });
      }
      return result(body, { source: cmd });
    }

    const res = await run(ch.meta.argv[0], ch.meta.argv.slice(1), { timeoutMs: ctx.timeoutMs });
    const body = `${res.stdout}${res.stderr ? `\n${res.stderr}` : ""}`.trimEnd();
    if (!res.ok && !body) {
      return failure({
        tried: ch.source,
        reason: explainFailure(res),
        hint: ch.meta.hint,
        source: ch.source,
      });
    }
    return result(body || "(none)", { source: ch.source });
  },
};
