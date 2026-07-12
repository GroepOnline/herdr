// baseline — record downstream state snapshot
const { execSync } = require("child_process");

const head = execSync("git rev-parse --short HEAD", { encoding: "utf8" }).trim();
const base = execSync("git merge-base HEAD upstream/master", { encoding: "utf8" }).trim();
const behind = execSync("git rev-list --count HEAD..upstream/master", { encoding: "utf8" }).trim();
const ahead = execSync("git rev-list --count $(git merge-base HEAD upstream/master)..HEAD", { encoding: "utf8" }).trim();

console.log(JSON.stringify({
  date: new Date().toISOString(),
  head,
  upstream_base: base.substring(0, 7),
  version: execSync("grep '^version' Cargo.toml | head -1", { encoding: "utf8" }).trim(),
  downstream_commits: parseInt(ahead),
  behind_upstream: parseInt(behind),
  tags: execSync("git tag --points-at HEAD", { encoding: "utf8" }).trim().split("\n").filter(Boolean),
}, null, 2));