#!/usr/bin/env node

const fs = require("fs");
const https = require("https");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");

const pkg = require("../package.json");

if (process.env.PETERNAK_SKIP_DOWNLOAD === "1") {
  console.log("Skipping peternak-aiai binary download.");
  process.exit(0);
}

const target = resolveTarget();
const archiveName = `peternak-aiai-${target}.tar.gz`;
const baseUrl =
  process.env.PETERNAK_DOWNLOAD_BASE ||
  `https://github.com/timbubadibako/peternak/releases/download/v${pkg.version}`;
const url = `${baseUrl.replace(/\/$/, "")}/${archiveName}`;

const vendorDir = path.join(__dirname, "vendor");
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "peternak-aiai-"));
const archivePath = path.join(tmpDir, archiveName);

fs.mkdirSync(vendorDir, { recursive: true });

console.log(`Downloading ${url}`);
download(url, archivePath)
  .then(() => {
    const tar = spawnSync("tar", ["-xzf", archivePath, "-C", vendorDir], {
      stdio: "inherit",
    });
    if (tar.status !== 0) {
      throw new Error("failed to extract release archive with tar");
    }

    const exeName = process.platform === "win32" ? "peternak-aiai.exe" : "peternak-aiai";
    const binary = path.join(vendorDir, exeName);
    if (!fs.existsSync(binary)) {
      throw new Error(`archive did not contain ${exeName}`);
    }

    if (process.platform !== "win32") {
      fs.chmodSync(binary, 0o755);
    }

    console.log(`Installed ${binary}`);
  })
  .catch((error) => {
    console.error(`peternak-aiai install failed: ${error.message}`);
    console.error("Set PETERNAK_DOWNLOAD_BASE to your release URL if using a private build.");
    process.exit(1);
  });

function resolveTarget() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === "linux" && arch === "x64") return "x86_64-unknown-linux-gnu";
  if (platform === "darwin" && arch === "x64") return "x86_64-apple-darwin";
  if (platform === "darwin" && arch === "arm64") return "aarch64-apple-darwin";
  if (platform === "win32" && arch === "x64") return "x86_64-pc-windows-msvc";

  throw new Error(`unsupported platform ${platform}/${arch}`);
}

function download(url, destination) {
  return new Promise((resolve, reject) => {
    const request = https.get(url, (response) => {
      if ([301, 302, 303, 307, 308].includes(response.statusCode)) {
        response.resume();
        download(response.headers.location, destination).then(resolve, reject);
        return;
      }

      if (response.statusCode !== 200) {
        response.resume();
        reject(new Error(`download returned HTTP ${response.statusCode}`));
        return;
      }

      const file = fs.createWriteStream(destination);
      response.pipe(file);
      file.on("finish", () => file.close(resolve));
      file.on("error", reject);
    });

    request.on("error", reject);
  });
}
