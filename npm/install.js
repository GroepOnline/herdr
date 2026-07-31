#!/usr/bin/env node
"use strict";

const { createHash } = require("crypto");
const {
  chmodSync,
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
} = require("fs");
const { createWriteStream } = require("fs");
const https = require("https");
const { join } = require("path");

const packageJson = require("./package.json");
const VERSION = packageJson.version;
const REPO = repositorySlug(packageJson.repository);
const BIN_DIR = join(__dirname, "bin");
const BINARY_NAME = "herdr";
const BINARY_PATH = join(BIN_DIR, BINARY_NAME);
const MAX_REDIRECTS = 5;
const REQUEST_TIMEOUT_MS = 30_000;

const TARGETS = {
  "linux-x64": "herdr-linux-x86_64",
  "linux-arm64": "herdr-linux-aarch64",
  "darwin-x64": "herdr-macos-x86_64",
  "darwin-arm64": "herdr-macos-aarch64",
};

function repositorySlug(repository) {
  const raw = typeof repository === "string" ? repository : repository && repository.url;
  if (!raw) {
    throw new Error("package.json is missing repository.url");
  }

  // Split on a fixed host marker instead of matching the whole URL, so the
  // parse stays linear in the input length.
  const hostIndex = raw.search(/github\.com[/:]/);
  if (hostIndex === -1) {
    throw new Error("package.json repository must point to GitHub");
  }

  const path = raw
    .slice(hostIndex + "github.com/".length)
    .split("#")[0]
    .split("?")[0];
  const segments = path.split("/").filter((segment) => segment.length > 0);
  if (segments.length !== 2) {
    throw new Error("package.json repository must point to GitHub");
  }

  const owner = segments[0];
  const name = segments[1].endsWith(".git")
    ? segments[1].slice(0, -".git".length)
    : segments[1];
  if (!name) {
    throw new Error("package.json repository must point to GitHub");
  }
  return `${owner}/${name}`;
}

function releaseUrl(asset) {
  return `https://github.com/${REPO}/releases/download/v${VERSION}/${asset}`;
}

function request(url, redirects = 0) {
  return new Promise((resolve, reject) => {
    const req = https.get(url, { headers: { "User-Agent": `${packageJson.name}/${VERSION}` } }, resolve);
    req.setTimeout(REQUEST_TIMEOUT_MS, () => {
      req.destroy(new Error(`request timed out after ${REQUEST_TIMEOUT_MS}ms`));
    });
    req.on("error", reject);
  }).then((response) => {
    if ([301, 302, 303, 307, 308].includes(response.statusCode)) {
      response.resume();
      if (!response.headers.location) {
        throw new Error(`redirect from ${url} did not include a location`);
      }
      if (redirects >= MAX_REDIRECTS) {
        throw new Error(`too many redirects while downloading ${url}`);
      }
      return request(new URL(response.headers.location, url).toString(), redirects + 1);
    }
    if (response.statusCode !== 200) {
      response.resume();
      throw new Error(`download failed: HTTP ${response.statusCode} for ${url}`);
    }
    return response;
  });
}

async function downloadText(url) {
  const response = await request(url);
  const chunks = [];
  for await (const chunk of response) {
    chunks.push(chunk);
  }
  return Buffer.concat(chunks).toString("utf8");
}

async function downloadFile(url, destination) {
  const response = await request(url);
  await new Promise((resolve, reject) => {
    const file = createWriteStream(destination, { mode: 0o755 });
    response.on("error", reject);
    file.on("error", reject);
    file.on("finish", resolve);
    response.pipe(file);
  });
}

function expectedChecksum(sums, asset) {
  for (const line of sums.split(/\r?\n/)) {
    const match = line.trim().match(/^([a-fA-F0-9]{64})\s+\*?(.+)$/);
    if (match && match[2] === asset) {
      return match[1].toLowerCase();
    }
  }
  throw new Error(`SHA256SUMS does not contain ${asset}`);
}

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function binaryMatches(path, expected) {
  return existsSync(path) && sha256(path) === expected;
}

async function install() {
  const target = `${process.platform}-${process.arch}`;
  const asset = TARGETS[target];
  if (!asset) {
    throw new Error(
      `no prebuilt binary for ${target}; supported targets: ${Object.keys(TARGETS).join(", ")}`,
    );
  }

  mkdirSync(BIN_DIR, { recursive: true });
  const tempPath = join(BIN_DIR, `.herdr-download-${process.pid}-${Date.now()}`);

  try {
    const sums = await downloadText(releaseUrl("SHA256SUMS"));
    const expected = expectedChecksum(sums, asset);
    if (binaryMatches(BINARY_PATH, expected)) {
      console.log(`herdr ${VERSION} already installed and verified`);
      return;
    }

    if (existsSync(BINARY_PATH)) {
      console.log(`Replacing unverified herdr binary for ${target}...`);
    } else {
      console.log(`Downloading herdr ${VERSION} (${target})...`);
    }
    await downloadFile(releaseUrl(asset), tempPath);

    const actual = sha256(tempPath);
    if (actual !== expected) {
      throw new Error(`checksum mismatch for ${asset}: expected ${expected}, got ${actual}`);
    }

    chmodSync(tempPath, 0o755);
    renameSync(tempPath, BINARY_PATH);
    console.log(`herdr ${VERSION} installed to ${BINARY_PATH}`);
  } finally {
    rmSync(tempPath, { force: true });
  }
}

module.exports = {
  binaryMatches,
  expectedChecksum,
  repositorySlug,
  sha256,
};

if (require.main === module) {
  install().catch((error) => {
    console.error(`herdr install failed: ${error.message}`);
    process.exitCode = 1;
  });
}
