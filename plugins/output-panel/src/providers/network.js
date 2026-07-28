/**
 * Network snapshots: listening sockets, established connections, service
 * probes, interface addresses and Tailscale state.
 *
 * These are live channels — every read re-runs the command or probe, so tail
 * mode turns them into a poor man's watch(1).
 */
import { channel, failure, result } from "../lib/channel.js";
import { explainFailure, run } from "../lib/exec.js";
import { LOCAL_SERVICES, probeService } from "../lib/probe.js";

const COMMANDS = {
  "net:listening": {
    title: "listening sockets",
    subtitle: "ss -ltnp",
    argv: ["ss", "-ltnp"],
    hint: "install iproute2 (`ss`) to see listeners",
  },
  "net:established": {
    title: "established connections",
    subtitle: "ss -tnp state established",
    argv: ["ss", "-tnp", "state", "established"],
    hint: "install iproute2 (`ss`) to see connections",
  },
  "net:udp": {
    title: "udp sockets",
    subtitle: "ss -lunp",
    argv: ["ss", "-lunp"],
    hint: "install iproute2 (`ss`)",
  },
  "net:interfaces": {
    title: "interfaces",
    subtitle: "ip -brief address",
    argv: ["ip", "-brief", "address"],
    hint: "install iproute2 (`ip`)",
  },
};

export const networkProvider = {
  id: "network",
  group: "Network",

  async list() {
    const channels = Object.entries(COMMANDS).map(([id, spec]) =>
      channel({
        id,
        title: spec.title,
        group: "Network",
        subtitle: spec.subtitle,
        source: spec.argv.join(" "),
        kind: "command",
        meta: { argv: spec.argv, hint: spec.hint },
      }),
    );
    channels.push(
      channel({
        id: "net:probes",
        title: "local service probes",
        group: "Network",
        subtitle: `${LOCAL_SERVICES.length} known endpoints (kater, cdp, vault, ops…)`,
        source: LOCAL_SERVICES.map((s) => `${s.host}:${s.port}`).join(" "),
        kind: "probes",
        meta: {},
      }),
    );
    channels.push(
      channel({
        id: "net:tailscale",
        title: "tailscale status",
        group: "Network",
        subtitle: "tailscale status (falls back to sudo -n)",
        source: "tailscale status",
        kind: "tailscale",
        meta: {},
      }),
    );
    return channels;
  },

  async read(ch, ctx) {
    if (ch.kind === "probes") {
      const rows = await Promise.all(LOCAL_SERVICES.map((s) => probeService(s, 1500)));
      return result(
        [
          `probed ${LOCAL_SERVICES.length} local endpoints at ${new Date().toLocaleTimeString()}`,
          "",
          ...rows,
        ].join("\n"),
        { source: ch.source, note: "TCP connect + read-only HTTP GET, 1.5s timeout each" },
      );
    }

    if (ch.kind === "tailscale") {
      // Plain first: on this machine the CLI reads state without elevation.
      const plain = await run("tailscale", ["status"], { timeoutMs: ctx.timeoutMs });
      if (plain.ok) {
        return result(plain.stdout.trimEnd(), { source: "tailscale status" });
      }
      if (plain.missing) {
        return failure({
          tried: "tailscale status",
          reason: "tailscale is not installed or not on PATH",
          hint: "install Tailscale, or ignore this channel on hosts without it",
        });
      }
      // `sudo -n` never prompts: it fails immediately when a password is needed.
      const elevated = await run("sudo", ["-n", "tailscale", "status"], {
        timeoutMs: ctx.timeoutMs,
      });
      if (elevated.ok) {
        return result(elevated.stdout.trimEnd(), {
          source: "sudo -n tailscale status",
          note: "read via passwordless sudo",
        });
      }
      return failure({
        tried: "tailscale status  /  sudo -n tailscale status",
        reason: "unavailable (needs sudo)",
        hint: "run `sudo tailscale status` in a normal terminal; the panel never prompts for a password",
        source: "tailscale status",
      });
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
    return result(body || "(no rows)", {
      source: ch.source,
      note: `ran at ${new Date().toLocaleTimeString()}`,
    });
  },
};
