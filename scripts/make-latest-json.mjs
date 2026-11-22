// Builds the updater's latest.json from the signed release assets.
//
//   node scripts/make-latest-json.mjs --version 0.2.0 --tag v0.2.0 --repo To-Recomp/launcher --dir release-assets
//
// Every `<name>.sig` next to its asset contributes one platform; a platform whose asset isn't
// signed is left out, so an unsigned build produces no updater entry at all.

import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

/** Release asset name suffix -> platform key of the Tauri updater. */
export const PLATFORMS = {
  "_linux-x86_64.AppImage": "linux-x86_64",
  "_windows-x86_64-setup.exe": "windows-x86_64",
};

/**
 * @param {{ version: string, tag: string, repo: string, signatures: Record<string, string>, pubDate?: string }} input
 *   `signatures` maps an asset file name to the content of its `.sig` file.
 */
export function buildLatest({ version, tag, repo, signatures, pubDate = new Date().toISOString() }) {
  const platforms = {};
  for (const [asset, signature] of Object.entries(signatures)) {
    const suffix = Object.keys(PLATFORMS).find((s) => asset.endsWith(s));
    if (!suffix) continue;
    platforms[PLATFORMS[suffix]] = {
      signature: signature.trim(),
      url: `https://github.com/${repo}/releases/download/${tag}/${encodeURIComponent(asset)}`,
    };
  }
  if (Object.keys(platforms).length === 0) return null;
  return {
    version,
    notes: `https://github.com/${repo}/releases/tag/${tag}`,
    pub_date: pubDate,
    platforms,
  };
}

function readSignatures(dir) {
  const signatures = {};
  for (const file of readdirSync(dir)) {
    if (file.endsWith(".sig")) signatures[file.slice(0, -4)] = readFileSync(join(dir, file), "utf8");
  }
  return signatures;
}

function main(argv) {
  const args = Object.fromEntries(
    argv.flatMap((arg, i) => (arg.startsWith("--") ? [[arg.slice(2), argv[i + 1]]] : [])),
  );
  for (const required of ["version", "tag", "repo", "dir"]) {
    if (!args[required]) throw new Error(`missing --${required}`);
  }
  const latest = buildLatest({ ...args, signatures: readSignatures(args.dir) });
  if (!latest) {
    console.error("No signed updater assets found: not writing latest.json.");
    process.exit(3);
  }
  process.stdout.write(JSON.stringify(latest, null, 2) + "\n");
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main(process.argv.slice(2));
