#!/usr/bin/env node

const { spawnSync } = require("child_process");
const path = require("path");

const PLATFORMS = {
  "linux-x64": "schema_analysis-linux-x64",
  "linux-arm64": "schema_analysis-linux-arm64",
  "darwin-x64": "schema_analysis-darwin-x64",
  "darwin-arm64": "schema_analysis-darwin-arm64",
  "win32-x64": "schema_analysis-win32-x64.exe",
};

const key = `${process.platform}-${process.arch}`;
const bin = PLATFORMS[key];
if (!bin) {
  console.error(
    `schema_analysis: unsupported platform ${process.platform}-${process.arch}`
  );
  process.exit(1);
}

const binPath = path.join(__dirname, "..", "binaries", bin);
const result = spawnSync(binPath, process.argv.slice(2), { stdio: "inherit" });
process.exit(result.status ?? 1);
