// sync-upstream — rebase herdr-private on upstream stable release
const { execSync } = require("child_process");

const target = process.argv[2] || "v0.7.3";
console.log(`Syncing with upstream ${target}...`);

execSync("git fetch upstream --tags", { stdio: "inherit" });
execSync(`git checkout -b sync/${target} upstream/${target} || git checkout sync/${target}`, { stdio: "inherit" });

// Cherry-pick downstream patches
const downstreamCommits = execSync("git rev-list --reverse HEAD..origin/master", { encoding: "utf8" }).trim().split("\n").filter(Boolean);
console.log(`Cherry-picking ${downstreamCommits.length} downstream commits...`);

for (const commit of downstreamCommits) {
  try {
    execSync(`git cherry-pick ${commit}`, { stdio: "inherit" });
  } catch {
    console.error(`Conflict on ${commit} — resolve manually`);
    process.exit(1);
  }
}

console.log("Sync complete. Run tests then merge to master.");