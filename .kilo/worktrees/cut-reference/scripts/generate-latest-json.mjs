import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";

const [assetDir, outFile] = process.argv.slice(2);

if (!assetDir || !outFile) {
  console.error("Usage: node scripts/generate-latest-json.mjs <asset-dir> <out-file>");
  process.exit(1);
}

const version = process.env.VERSION;
const repository = process.env.GITHUB_REPOSITORY || "johuniq/whisprtypr";
const stripTrailingLineBreaks = (value) => value?.replace(/[\r\n]+$/, "");
const expandEscapedNewlines = (value) => value.replace(/\\n/g, "\n").replace(/\\r/g, "\r");
const normalizeSigningKey = (rawValue) => {
  if (!rawValue) return rawValue;

  const trimmed = rawValue.trim();
  const withRealNewlines = expandEscapedNewlines(trimmed);
  if (withRealNewlines.includes("BEGIN PRIVATE KEY")) {
    return stripTrailingLineBreaks(withRealNewlines);
  }

  try {
    const decoded = Buffer.from(trimmed, "base64").toString("utf8");
    const normalizedDecoded = expandEscapedNewlines(decoded).trim();
    if (normalizedDecoded.includes("BEGIN PRIVATE KEY")) {
      return stripTrailingLineBreaks(normalizedDecoded);
    }
  } catch {
    // keep original format if not valid base64
  }

  return stripTrailingLineBreaks(withRealNewlines);
};

const signingKey = normalizeSigningKey(process.env.TAURI_SIGNING_PRIVATE_KEY);
const signingPassword = stripTrailingLineBreaks(process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD) || "";
const pubDate = process.env.PUB_DATE || new Date().toISOString().replace(/\.\d{3}Z$/, "Z");
const requireAllPlatforms = process.env.REQUIRE_ALL_PLATFORMS === "true";

if (!version) {
  console.error("VERSION is required.");
  process.exit(1);
}

const files = readdirSync(assetDir).filter((name) => !name.endsWith(".sig"));
let existingManifest = null;
let signingKeyPath = null;
let signingKeyDir = null;

if (existsSync(outFile)) {
  try {
    existingManifest = JSON.parse(readFileSync(outFile, "utf8"));
  } catch (error) {
    console.warn(`Ignoring existing ${outFile}: ${error.message}`);
  }
}

process.on("exit", () => {
  if (signingKeyDir) {
    rmSync(signingKeyDir, { recursive: true, force: true });
  }
});

function findAsset(patterns) {
  return files.find((name) => patterns.every((pattern) => pattern.test(name)));
}

function getSigningKeyPath() {
  if (!signingKey) {
    console.error(
      "TAURI_SIGNING_PRIVATE_KEY is required to create missing updater signatures."
    );
    process.exit(1);
  }

  if (!signingKeyPath) {
    signingKeyDir = mkdtempSync(join(tmpdir(), "whisprtypr-tauri-key-"));
    signingKeyPath = join(signingKeyDir, "tauri.key");
    writeFileSync(signingKeyPath, signingKey, { mode: 0o600 });
  }

  return signingKeyPath;
}

function existingSignatureFor(fileName, platformKey) {
  const platforms = existingManifest?.platforms;
  if (!platforms || typeof platforms !== "object") return null;

  const candidates = [
    platforms[platformKey],
    ...Object.values(platforms),
  ].filter(Boolean);

  const entry = candidates.find((candidate) => {
    if (!candidate.signature || !candidate.url) return false;
    return basename(new URL(candidate.url).pathname) === fileName;
  });

  return entry?.signature || null;
}

function signAsset(fileName, platformKey) {
  if (!fileName) return null;

  const filePath = join(assetDir, fileName);
  const signaturePath = `${filePath}.sig`;
  const existingSignature = existingSignatureFor(fileName, platformKey);

  if (!existsSync(signaturePath) && !existingSignature) {
    const signerEnv = { ...process.env };
    delete signerEnv.TAURI_SIGNING_PRIVATE_KEY;
    signerEnv.TAURI_SIGNING_PRIVATE_KEY_PASSWORD = signingPassword;

    try {
      execFileSync("cargo", [
        "tauri",
        "signer",
        "sign",
        "--private-key-path",
        getSigningKeyPath(),
        "--password",
        signingPassword,
        filePath,
      ], {
        env: signerEnv,
        stdio: "inherit",
      });
    } catch (error) {
      console.error(
        [
          `Failed to sign ${fileName}.`,
          "Verify that TAURI_SIGNING_PRIVATE_KEY is the exact contents of the Tauri private key file",
          "and TAURI_SIGNING_PRIVATE_KEY_PASSWORD matches the password used when that key was generated.",
        ].join(" ")
      );
      throw error;
    }
  }

  return {
    signature: existingSignature || readFileSync(signaturePath, "utf8").trim(),
    url: `https://github.com/${repository}/releases/download/v${version}/${basename(fileName)}`,
  };
}

const windowsExe = findAsset([/^WhisprTypr_.*_x64-setup\.exe$/]);
const windowsMsi = findAsset([/^WhisprTypr_.*_x64.*\.msi$/]);
const darwinAarch64 = findAsset([/^WhisprTypr_.*_aarch64\.app\.tar\.gz$/]);
const darwinX64 = findAsset([/^WhisprTypr_.*_x64\.app\.tar\.gz$/]) || findAsset([/^WhisprTypr_.*_x86_64\.app\.tar\.gz$/]);

if (requireAllPlatforms) {
  const missing = [
    ["Windows NSIS setup", windowsExe],
    ["Windows MSI", windowsMsi],
    ["macOS Apple Silicon app archive", darwinAarch64],
    ["macOS Intel app archive", darwinX64],
  ]
    .filter(([, fileName]) => !fileName)
    .map(([label]) => label);

  if (missing.length > 0) {
    console.error(`Missing release assets: ${missing.join(", ")}`);
    console.error(`Available assets: ${files.join(", ")}`);
    process.exit(1);
  }
}

const platforms = {};

const windowsUpdater = signAsset(windowsExe, "windows-x86_64");
if (windowsUpdater) platforms["windows-x86_64"] = windowsUpdater;

const windowsMsiEntry = signAsset(windowsMsi, "windows-x86_64-msi");
if (windowsMsiEntry) platforms["windows-x86_64-msi"] = windowsMsiEntry;

const darwinAarch64Entry = signAsset(darwinAarch64, "darwin-aarch64");
if (darwinAarch64Entry) platforms["darwin-aarch64"] = darwinAarch64Entry;

const darwinX64Entry = signAsset(darwinX64, "darwin-x86_64");
if (darwinX64Entry) platforms["darwin-x86_64"] = darwinX64Entry;

const manifest = {
  version,
  notes: `See the release notes for v${version}`,
  pub_date: pubDate,
  platforms,
};

writeFileSync(outFile, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`Generated ${outFile}`);
console.log(JSON.stringify(manifest, null, 2));
