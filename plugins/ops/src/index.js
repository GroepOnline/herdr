#!/usr/bin/env node
/**
 * com.chefgroep.ops scaffold.
 *
 * Writes a schema-compatible fleet_ops.json heartbeat fragment and reports the
 * host/socket runtime location in the CLI response so operators can see *what
 * runs where* without guessing. Full inventory (Tailscale, SSH probes,
 * UDO/Kater SSOTs) should land in GroepOnline/herdr-ops — keep this plugin
 * as the in-tree contract + free-CI exercise path.
 */
import { mkdir, rename, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

const SOURCE = "herdr-ops";
const TTL_SECONDS = 90;

function stateDir() {
  const dir = process.env.HERDR_PLUGIN_STATE_DIR;
  if (!dir) throw new Error("HERDR_PLUGIN_STATE_DIR is required");
  return dir;
}

function hostName() {
  for (const value of [
    process.env.HERDR_HOST_NAME,
    process.env.HOSTNAME,
    os.hostname(),
  ]) {
    if (typeof value === "string") {
      const trimmed = value.trim();
      if (trimmed) return trimmed;
    }
  }
  return "unknown";
}

// Persisted fragment. Core (`src/fleet/ops.rs::PluginFleetFragment`) only reads
// these keys, so keep the on-disk payload to fields it understands: adding a
// `location` object here would be silently dropped on deserialize. Runtime
// location is surfaced through the CLI response instead (see `location()`).
function fragment() {
  return {
    source: SOURCE,
    updated_at: new Date().toISOString(),
    ttl_seconds: TTL_SECONDS,
  };
}

// Host/socket runtime location for the CLI response only. Fleet online/total
// counts are the SSOT of fleet-health (Tailscale/SSH probes), not this scaffold.
function location() {
  return {
    host: hostName(),
    socket: process.env.HERDR_SOCKET_PATH || "default",
    note: "scaffold → herdr-ops",
  };
}

async function writeFleetOps(data) {
  const dir = stateDir();
  await mkdir(dir, { recursive: true });
  const target = path.join(dir, "fleet_ops.json");
  const tmp = path.join(dir, `fleet_ops.json.${process.pid}.tmp`);
  // Write to a temp file then rename so readers never observe a partial write.
  await writeFile(tmp, `${JSON.stringify(data, null, 2)}\n`, "utf8");
  await rename(tmp, target);
  return target;
}

async function main() {
  const action = process.argv[2] || "publish-context";
  switch (action) {
    case "publish-context":
    case "on-workspace-focused":
      break;
    default:
      console.error(`unknown action: ${action}`);
      process.exitCode = 1;
      return;
  }

  const data = fragment();
  const target = await writeFleetOps(data);
  console.log(
    JSON.stringify({
      ok: true,
      action,
      path: target,
      host: hostName(),
      recommended_repo: "GroepOnline/herdr-ops",
      location: location(),
    }),
  );
}

main().catch((err) => {
  console.error(err?.stack || String(err));
  process.exitCode = 1;
});
