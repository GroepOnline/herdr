#!/usr/bin/env node
import { mkdir, readdir, readFile, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

const SOURCE = "session-park";
const TTL_SECONDS = 60;

function stateDir() {
  const dir = process.env.HERDR_PLUGIN_STATE_DIR;
  if (!dir) throw new Error("HERDR_PLUGIN_STATE_DIR is required");
  return dir;
}

function parkedDir() {
  return path.join(stateDir(), "parked");
}

async function writeFleetOps(data) {
  const dir = stateDir();
  await mkdir(dir, { recursive: true });
  const target = path.join(dir, "fleet_ops.json");
  await writeFile(target, `${JSON.stringify(data, null, 2)}\n`, "utf8");
  return target;
}

async function listParkedRecords() {
  const dir = parkedDir();
  await mkdir(dir, { recursive: true });
  const names = await readdir(dir);
  const records = [];
  for (const name of names) {
    if (!name.endsWith(".json")) continue;
    try {
      const raw = await readFile(path.join(dir, name), "utf8");
      records.push(JSON.parse(raw));
    } catch {
      // skip unreadable records
    }
  }
  return records;
}

function parkedSummary(records) {
  let oldestHours = null;
  const now = Date.now();
  for (const record of records) {
    const ts = Date.parse(record.parked_at || "");
    if (!Number.isFinite(ts)) continue;
    const hours = (now - ts) / (1000 * 60 * 60);
    if (oldestHours === null || hours > oldestHours) oldestHours = hours;
  }
  return {
    count: records.length,
    oldest_hours: oldestHours === null ? null : Math.round(oldestHours * 10) / 10,
  };
}

function fragment(parked) {
  return {
    source: SOURCE,
    updated_at: new Date().toISOString(),
    ttl_seconds: TTL_SECONDS,
    parked,
  };
}

async function writeParkRecord(id, extra = {}) {
  const dir = parkedDir();
  await mkdir(dir, { recursive: true });
  const record = {
    id,
    parked_at: new Date().toISOString(),
    workspace_id: process.env.HERDR_WORKSPACE_ID || null,
    pane_id: process.env.HERDR_PANE_ID || null,
    ...extra,
  };
  const target = path.join(dir, `${id}.json`);
  await writeFile(target, `${JSON.stringify(record, null, 2)}\n`, "utf8");
  return record;
}

async function refreshOps() {
  const records = await listParkedRecords();
  const data = fragment(parkedSummary(records));
  const target = await writeFleetOps(data);
  return { records, data, target };
}

async function main() {
  const action = process.argv[2] || "list-parked";
  const idArg = process.argv[3] || "";

  switch (action) {
    case "park": {
      const id =
        idArg ||
        process.env.HERDR_PANE_ID ||
        `pane-${Date.now()}`;
      await writeParkRecord(id, { scope: "pane" });
      break;
    }
    case "park-workspace": {
      const id =
        idArg ||
        process.env.HERDR_WORKSPACE_ID ||
        `workspace-${Date.now()}`;
      await writeParkRecord(id, { scope: "workspace" });
      break;
    }
    case "resume": {
      if (!idArg) {
        console.error("resume requires a parked record id");
        process.exitCode = 1;
        return;
      }
      const file = path.join(parkedDir(), `${idArg}.json`);
      try {
        await unlink(file);
      } catch (err) {
        if (err && err.code !== "ENOENT") throw err;
      }
      break;
    }
    case "expire": {
      const maxHours = Number(process.env.HERDR_PARK_EXPIRE_HOURS || "72");
      const records = await listParkedRecords();
      const cutoff = Date.now() - maxHours * 60 * 60 * 1000;
      for (const record of records) {
        const ts = Date.parse(record.parked_at || "");
        if (!Number.isFinite(ts) || ts > cutoff) continue;
        await unlink(path.join(parkedDir(), `${record.id}.json`)).catch(() => {});
      }
      break;
    }
    case "list-parked":
    case "on-pane-agent-status-changed":
    case "on-pane-exited":
      break;
    default:
      console.error(`unknown action: ${action}`);
      process.exitCode = 1;
      return;
  }

  const { records, data, target } = await refreshOps();
  console.log(
    JSON.stringify({
      ok: true,
      action,
      path: target,
      parked: data.parked,
      records: records.map((r) => r.id),
    }),
  );
}

main().catch((err) => {
  console.error(err?.stack || String(err));
  process.exitCode = 1;
});
