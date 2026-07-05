#!/usr/bin/env node

const fs = require("fs");
const path = require("path");
const { spawn } = require("child_process");

const exeName = process.platform === "win32" ? "peternak-aiai.exe" : "peternak-aiai";
const binary = path.join(__dirname, "..", "vendor", exeName);

if (!fs.existsSync(binary)) {
  console.error("peternak-aiai binary is not installed.");
  console.error("Run `npm rebuild -g peternak-aiai` or reinstall the package.");
  process.exit(1);
}

const child = spawn(binary, process.argv.slice(2), {
  stdio: "inherit",
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 0);
});
