#!/usr/bin/env node
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

const SOURCE = "github-status";
const TTL_SECONDS = 30;

function stateDir() {
  const dir = process.env.HERDR_PLUGIN_STATE_DIR;
  if (!dir) throw new Error("HERDR_PLUGIN_STATE_DIR is required");
  return dir;
}

function fragment(pr = {}) {
  return {
    source: SOURCE,
    updated_at: new Date().toISOString(),
    ttl_seconds: TTL_SECONDS,
    pr: {
      number: pr.number ?? null,
      checks: pr.checks ?? "",
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
  const action = process.argv[2] || "check-pr";
  const numberRaw = process.argv[3] || process.env.HERDR_GITHUB_PR_NUMBER || "";
  const number = numberRaw ? Number(numberRaw) : null;

  let pr = { number: Number.isFinite(number) ? number : null, checks: "" };
  switch (action) {
    case "check-pr":
    case "list-open-prs":
    case "watch-checks":
    case "on-worktree-opened":
      break;
    default:
      console.error(`unknown action: ${action}`);
      process.exitCode = 1;
      return;
  }

  // Scaffold: write PR/CI fragment without calling GitHub yet.
  const data = fragment(pr);
  const target = await writeFleetOps(data);
  console.log(JSON.stringify({ ok: true, action, path: target, pr: data.pr }));
}

main().catch((err) => {
  console.error(err?.stack || String(err));
  process.exitCode = 1;
});
