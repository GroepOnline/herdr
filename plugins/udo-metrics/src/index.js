#!/usr/bin/env node
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

const SOURCE = "udo-metrics";
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
  const action = process.argv[2] || "refresh-metrics";
  const node = process.argv[3] || "";

  switch (action) {
    case "refresh-metrics":
    case "fetch-node":
    case "on-workspace-focused":
      break;
    default:
      console.error(`unknown action: ${action}`);
      process.exitCode = 1;
      return;
  }

  const summary =
    action === "fetch-node" && node
      ? `udo metrics: ${node} (scaffold)`
      : "udo metrics: unknown (scaffold)";
  const data = fragment({ summary });
  const target = await writeFleetOps(data);
  console.log(JSON.stringify({ ok: true, action, path: target, fleet: data.fleet }));
}

main().catch((err) => {
  console.error(err?.stack || String(err));
  process.exitCode = 1;
});
