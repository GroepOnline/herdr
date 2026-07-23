#!/usr/bin/env node
const { createWriteStream, existsSync, mkdirSync, chmodSync } = require("fs");
const { join } = require("path");
const https = require("https");

const VERSION = "0.7.6";
const REPO = "OnlineChefGroep/herdr";
const BIN_DIR = join(__dirname, "bin");
const BINARY_NAME = "herdr";
const BINARY_PATH = join(BIN_DIR, BINARY_NAME);

if (process.platform !== "linux" || process.arch !== "x64") {
  console.error("No prebuilt binary for " + process.platform + "-" + process.arch);
  console.error("This distribution ships linux-x86_64 only.");
  console.error("Build from source: git clone https://github.com/" + REPO);
  process.exit(1);
}

if (existsSync(BINARY_PATH)) {
  console.log("herdr " + VERSION + " already installed");
  process.exit(0);
}

const asset = "herdr-linux-x86_64";
const platform = "linux-x64";

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
      chmodSync(BINARY_PATH, 0o755);
      console.log("herdr " + VERSION + " installed to " + BINARY_PATH);
    });
  }).on("error", (err) => {
    console.error("Download failed: " + err.message);
    process.exit(1);
  });
}

doDownload(url);
