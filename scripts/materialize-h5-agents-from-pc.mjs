#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const pcAgentsRoot = path.resolve(repoRoot, "apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-agents");
const h5AgentsRoot = path.resolve(repoRoot, "apps/sdkwork-agents-h5/packages/sdkwork-agents-h5-agents");

function copyDir(src, dest) {
  fs.mkdirSync(dest, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    if (entry.name === "node_modules") {
      continue;
    }
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      copyDir(srcPath, destPath);
      continue;
    }
    if (!/\.(ts|tsx|json)$/.test(entry.name)) {
      continue;
    }
    let content = fs.readFileSync(srcPath, "utf8");
    content = content
      .split("@sdkwork/agents-pc-commons").join("@sdkwork/agents-h5-commons")
      .split("@sdkwork/agents-pc-core").join("@sdkwork/agents-h5-core")
      .split("sdkwork-agents-pc").join("sdkwork-agents-h5")
      .split("VITE_SDKWORK_AGENTS_PC_").join("VITE_SDKWORK_AGENTS_H5_")
      .split('const CLIENT_SURFACE = "pc";').join('const CLIENT_SURFACE = "h5";');
    fs.writeFileSync(destPath, content, "utf8");
  }
}

function listRelativeFiles(root, prefix = "") {
  const found = [];
  if (!fs.existsSync(root)) return found;
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    if (entry.name === "node_modules") continue;
    const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) {
      found.push(...listRelativeFiles(path.join(root, entry.name), relative));
      continue;
    }
    if (/\.(ts|tsx|json)$/.test(entry.name)) found.push(relative);
  }
  return found;
}

/**
 * Pre-flight guard.
 *
 * The H5 agents package stopped being a pure derivative of the PC agents package:
 * it owns mobile-only surfaces (`MyAgentsView`, `mobileAgentTexts`, the marketplace
 * and character views) that have no PC counterpart, and PC owns desktop-only
 * modules that must not be mirrored into H5. The script below deletes the whole H5
 * `src/` tree before copying, so running it unchanged would silently destroy those
 * H5-only files and inject PC-only ones.
 *
 * This guard makes that loss impossible to trigger by accident. It is purely
 * defensive: pass --allow-overwrite-local to restore the previous behaviour once
 * the two surfaces are reconciled deliberately.
 */
const h5AgentsSrcRoot = path.join(h5AgentsRoot, "src");
const pcAgentsSrcRoot = path.join(pcAgentsRoot, "src");
const allowOverwriteLocal = process.argv.includes("--allow-overwrite-local");
// Compare the same base on both sides: the script deletes H5 `src/` and copies PC
// `src/` over it, so only the two `src/` subtrees are comparable.
const pcSrcFiles = new Set(listRelativeFiles(pcAgentsSrcRoot));
const h5SrcFiles = listRelativeFiles(h5AgentsSrcRoot);
const h5OnlyFiles = h5SrcFiles.filter((relative) => !pcSrcFiles.has(relative));
const pcOnlyFiles = [...pcSrcFiles].filter((relative) => !h5SrcFiles.includes(relative));

if ((h5OnlyFiles.length > 0 || pcOnlyFiles.length > 0) && !allowOverwriteLocal) {
  console.error("materialize-h5-agents-from-pc: refusing to overwrite diverged sources.");
  console.error("");
  console.error("The H5 and PC agents packages are no longer the same file set.");
  console.error("Running this script would DELETE the H5-only files and ADD the PC-only files:");
  if (h5OnlyFiles.length > 0) {
    console.error(`  H5-only files that would be deleted (${h5OnlyFiles.length}):`);
    for (const relative of h5OnlyFiles) console.error(`    - ${relative}`);
  }
  if (pcOnlyFiles.length > 0) {
    console.error(`  PC-only files that would be added to H5 (${pcOnlyFiles.length}):`);
    for (const relative of pcOnlyFiles) console.error(`    + ${relative}`);
  }
  console.error("");
  console.error("H5-only files are hand-maintained (no script generates them).");
  console.error("If the divergence is intended, re-run with --allow-overwrite-local.");
  process.exit(1);
}

if (fs.existsSync(h5AgentsSrcRoot)) {
  fs.rmSync(h5AgentsSrcRoot, { recursive: true, force: true });
}
fs.mkdirSync(h5AgentsRoot, { recursive: true });
copyDir(pcAgentsRoot, h5AgentsRoot);

const h5PackageJson = {
  name: "@sdkwork/agents-h5-agents",
  private: true,
  version: "0.1.0",
  type: "module",
  exports: {
    ".": {
      types: "./src/index.ts",
      import: "./src/index.ts",
      default: "./src/index.ts",
    },
  },
  dependencies: {
    "@sdkwork/agents-h5-commons": "workspace:*",
    "@sdkwork/agents-h5-core": "workspace:*",
    "@sdkwork/utils": "workspace:*",
    "@tiptap/extension-placeholder": "catalog:",
    "@tiptap/pm": "catalog:",
    "@tiptap/react": "catalog:",
    "@tiptap/starter-kit": "catalog:",
    "emoji-picker-react": "catalog:",
    "lucide-react": "catalog:",
    "motion": "catalog:",
    "react": "catalog:",
  },
};

fs.writeFileSync(path.join(h5AgentsRoot, "package.json"), `${JSON.stringify(h5PackageJson, null, 2)}\n`, "utf8");
console.log("Materialized sdkwork-agents-h5-agents from PC agents package.");
