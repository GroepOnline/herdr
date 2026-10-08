#!/usr/bin/env node
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

const SOURCE = "issue-provision";
const TTL_SECONDS = 60;

function stateDir() {
  const dir = process.env.HERDR_PLUGIN_STATE_DIR;
  if (!dir) throw new Error("HERDR_PLUGIN_STATE_DIR is required");
  return dir;
}

function emptyIssue() {
  return { id: "", title: "", status: "", assignee: "", cycle: "" };
}

function fragment(issue = {}) {
  return {
    source: SOURCE,
    updated_at: new Date().toISOString(),
    ttl_seconds: TTL_SECONDS,
    issue: { ...emptyIssue(), ...issue },
  };
}

async function writeFleetOps(data) {
  const dir = stateDir();
  await mkdir(dir, { recursive: true });
  const target = path.join(dir, "fleet_ops.json");
  await writeFile(target, `${JSON.stringify(data, null, 2)}\n`, "utf8");
  return target;
}

function issueIdFromClickedUrl() {
  const url = process.env.HERDR_PLUGIN_CLICKED_URL || "";
  const match = url.match(/issue\/([A-Z]+-\d+)/i);
  return match ? match[1].toUpperCase() : "";
}

async function main() {
  const action = process.argv[2] || "provision";
  const issueId =
    process.argv[3] ||
    process.env.HERDR_LINEAR_ISSUE_ID ||
    issueIdFromClickedUrl() ||
    "";

  switch (action) {
    case "provision":
    case "teardown":
    case "list-provisioned":
      break;
    default:
      console.error(`unknown action: ${action}`);
      process.exitCode = 1;
      return;
  }

  // One-shot scaffold: record the intended issue id without creating worktrees yet.
  const data = fragment(issueId ? { id: issueId, status: action } : { status: action });
  const target = await writeFleetOps(data);
  console.log(
    JSON.stringify({
      ok: true,
      action,
      path: target,
      issue: data.issue,
      clicked_url: process.env.HERDR_PLUGIN_CLICKED_URL || null,
    }),
  );
}

main().catch((err) => {
  console.error(err?.stack || String(err));
  process.exitCode = 1;
});
