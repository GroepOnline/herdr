#!/usr/bin/env node
const { createWriteStream, existsSync, mkdirSync, chmodSync } = require("fs");
const { join } = require("path");
const https = require("https");
const os = require("os");

const VERSION = "0.7.3";
const REPO = "OnlineChefGroep/herdr";
const BIN_DIR = join(__dirname, "bin");
const BINARY_NAME = os.platform() === "win32" ? "herdr.exe" : "herdr";
const BINARY_PATH = join(BIN_DIR, BINARY_NAME);

if (existsSync(BINARY_PATH)) {
  console.log("herdr " + VERSION + " already installed");
  process.exit(0);
}

const platformMap = {
  "linux-x64": "herdr-linux-x86_64",
  "linux-arm64": "herdr-linux-aarch64",
  "darwin-x64": "herdr-macos-x86_64",
  "darwin-arm64": "herdr-macos-aarch64",
};

const arch = os.arch() === "arm64" ? "arm64" : "x64";
const platform = os.platform() + "-" + arch;
const asset = platformMap[platform];

if (!asset) {
  console.error("No prebuilt binary for " + platform);
  console.error("Build from source: git clone https://github.com/" + REPO);
  process.exit(1);
}

const url = "https://github.com/" + REPO + "/releases/download/v" + VERSION + "/" + asset;
console.log("Downloading herdr " + VERSION + " (" + platform + ")...");

mkdirSync(BIN_DIR, { recursive: true });
const file = createWriteStream(BINARY_PATH);

function doDownload(dlUrl) {
  https.get(dlUrl, (res) => {
    if (res.statusCode === 302 || res.statusCode === 301) {
      doDownload(res.headers.location);
      return;
    }
    if (res.statusCode !== 200) {
      console.error("Download failed: HTTP " + res.statusCode);
      process.exit(1);
    }
    res.pipe(file);
    file.on("finish", () => {
      file.close();
      if (os.platform() !== "win32") chmodSync(BINARY_PATH, 0o755);
      console.log("herdr " + VERSION + " installed to " + BINARY_PATH);
    });
  }).on("error", (err) => {
    console.error("Download failed: " + err.message);
    process.exit(1);
  });
}

doDownload(url);
