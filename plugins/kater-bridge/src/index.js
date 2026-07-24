#!/usr/bin/env node
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

const SOURCE = "kater-bridge";
const TTL_SECONDS = 60;

function stateDir() {
  const dir = process.env.HERDR_PLUGIN_STATE_DIR;
  if (!dir) throw new Error("HERDR_PLUGIN_STATE_DIR is required");
  return dir;
}

function fragment(fleet = {}) {
  return {
    source: SOURCE,
    updated_at: new Date().toISOString(),
    ttl_seconds: TTL_SECONDS,
    fleet: {
      online: fleet.online ?? null,
      total: fleet.total ?? null,
      summary: fleet.summary ?? "",
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
  const action = process.argv[2] || "fleet-status";

  switch (action) {
    case "fleet-status":
    case "pipeline-health":
    case "on-workspace-focused": {
      const data = fragment({
        online: null,
        total: null,
        summary: action === "pipeline-health" ? "pipeline: unknown" : "",
      });
      const target = await writeFleetOps(data);
      console.log(JSON.stringify({ ok: true, action, path: target, fleet: data.fleet }));
      return;
    }
    case "fleet-dashboard": {
      // Pane entrypoint: refresh fragment then print a tiny dashboard line.
      const data = fragment({ summary: "Utrecht fleet dashboard (scaffold)" });
      await writeFleetOps(data);
      console.log("Utrecht Fleet (scaffold)");
      console.log(`updated_at=${data.updated_at}`);
      console.log("Connect Kater MCP at http://127.0.0.1:9091 for live inventory.");
      // Keep pane alive briefly so herdr can attach; exit cleanly for headless runs.
      return;
    }
    default:
      console.error(`unknown action: ${action}`);
      process.exitCode = 1;
  }
}

main().catch((err) => {
  console.error(err?.stack || String(err));
  process.exitCode = 1;
});
