// herdr doctor — diagnostics for Chef downstream distribution
const { execSync } = require("child_process");

const checks: { name: string; run: () => string }[] = [
  { name: "bun", run: () => `bun ${Bun.version}` },
  { name: "rustc", run: () => execSync("rustc --version", { encoding: "utf8" }).trim() },
  { name: "cargo", run: () => execSync("cargo --version", { encoding: "utf8" }).trim() },
  { name: "zig", run: () => execSync("zig version", { encoding: "utf8" }).trim() },
  { name: "git", run: () => execSync("git --version", { encoding: "utf8" }).trim() },
  { name: "upstream-behind", run: () => execSync("git rev-list --count HEAD..upstream/master", { encoding: "utf8" }).trim() + " commits" },
  { name: "downstream-ahead", run: () => execSync("git rev-list --count $(git merge-base HEAD upstream/master)..HEAD", { encoding: "utf8" }).trim() + " commits" },
  { name: "rust-toolchain", run: () => Bun.file("rust-toolchain.toml").exists() ? "present" : "MISSING" },
  { name: "DOWNSTREAM.md", run: () => Bun.file("DOWNSTREAM.md").exists() ? "present" : "MISSING" },
];

console.log("herdr doctor — OnlineChefGroep downstream\n");
for (const check of checks) {
  try {
    console.log(`  ${check.name.padEnd(20)} ${check.run()}`);
  } catch (e) {
    console.log(`  ${check.name.padEnd(20)} ERROR: ${e}`);
  }
}