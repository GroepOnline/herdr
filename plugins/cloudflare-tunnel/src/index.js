#!/usr/bin/env node
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

const SOURCE = "cloudflare-tunnel";
const TTL_SECONDS = 300;

function stateDir() {
  const dir = process.env.HERDR_PLUGIN_STATE_DIR;
  if (!dir) throw new Error("HERDR_PLUGIN_STATE_DIR is required");
  return dir;
}

function fragment(cloudflare = {}) {
  return {
    source: SOURCE,
    updated_at: new Date().toISOString(),
    ttl_seconds: TTL_SECONDS,
    cloudflare: {
      tunnels_healthy: cloudflare.tunnels_healthy ?? null,
      summary: cloudflare.summary ?? "",
    },
  };
}

async function writeFleetOps(data) {
  const dir = stateDir();
  await mkdir(dir, { recursive: true });
  const target = path.join(dir, "fleet_ops.json");
  await writeFile(target, `${JSON.stringify(data, null, 2)}\n`, "utf8");
  return target;
}

async function main() {
  const action = process.argv[2] || "check-tunnels";

  switch (action) {
    case "check-tunnels":
    case "check-dns":
    case "dig-probe":
    case "on-workspace-focused":
      break;
    default:
      console.error(`unknown action: ${action}`);
      process.exitCode = 1;
      return;
  }

  const summary =
    action === "check-dns" || action === "dig-probe"
      ? `${action}: unknown (scaffold)`
      : "tunnels: unknown (scaffold)";
  const data = fragment({ tunnels_healthy: null, summary });
  const target = await writeFleetOps(data);
  console.log(
    JSON.stringify({ ok: true, action, path: target, cloudflare: data.cloudflare }),
  );
}

main().catch((err) => {
  console.error(err?.stack || String(err));
  process.exitCode = 1;
});
