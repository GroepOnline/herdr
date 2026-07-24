#!/usr/bin/env node
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const SOURCE = "linear-context";
const TTL_SECONDS = 60;

function stateDir() {
  const dir = process.env.HERDR_PLUGIN_STATE_DIR;
  if (!dir) throw new Error("HERDR_PLUGIN_STATE_DIR is required");
  return dir;
}

function configDir() {
  return process.env.HERDR_PLUGIN_CONFIG_DIR || "";
}

async function readEnvKey(name) {
  const root = configDir();
  if (!root) return process.env[name] || "";
  try {
    const text = await readFile(path.join(root, ".env"), "utf8");
    for (const line of text.split(/\r?\n/)) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith("#")) continue;
      const eq = trimmed.indexOf("=");
      if (eq <= 0) continue;
      const key = trimmed.slice(0, eq).trim();
      if (key !== name) continue;
      let value = trimmed.slice(eq + 1).trim();
      if (
        (value.startsWith('"') && value.endsWith('"')) ||
        (value.startsWith("'") && value.endsWith("'"))
      ) {
        value = value.slice(1, -1);
      }
      return value;
    }
  } catch {
    // missing .env is fine for scaffold stub mode
  }
  return process.env[name] || "";
}

function emptyIssue() {
  return { id: "", title: "", status: "", assignee: "", cycle: "" };
}

async function writeFleetOps(data) {
  const dir = stateDir();
  await mkdir(dir, { recursive: true });
  const target = path.join(dir, "fleet_ops.json");
  await writeFile(target, `${JSON.stringify(data, null, 2)}\n`, "utf8");
  return target;
}

function liveFragment(issue) {
  return {
    source: SOURCE,
    updated_at: new Date().toISOString(),
    ttl_seconds: TTL_SECONDS,
    issue: { ...emptyIssue(), ...issue },
  };
}

function stubFragment(issue) {
  return {
    source: "linear",
    updated_at: new Date().toISOString(),
    ttl_seconds: TTL_SECONDS,
    stale: false,
    issue: { ...emptyIssue(), ...issue },
  };
}

async function main() {
  const action = process.argv[2] || "fetch-issue";
  const issueId = process.argv[3] || process.env.HERDR_LINEAR_ISSUE_ID || "";

  let issue = {};
  switch (action) {
    case "set-issue":
      issue = { id: issueId };
      break;
    case "fetch-issue":
    case "fetch-cycle":
    case "on-worktree-created":
    case "on-workspace-focused":
      issue = issueId ? { id: issueId } : {};
      break;
    default:
      console.error(`unknown action: ${action}`);
      process.exitCode = 1;
      return;
  }

  const apiKey = await readEnvKey("LINEAR_API_KEY");
  const data = apiKey ? liveFragment(issue) : stubFragment(issue);
  const target = await writeFleetOps(data);
  console.log(JSON.stringify({ ok: true, action, path: target, source: data.source }));
}

main().catch((err) => {
  console.error(err?.stack || String(err));
  process.exitCode = 1;
});
