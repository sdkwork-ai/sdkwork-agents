import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

/**
 * Cross-client theme conformance contract.
 *
 * Authority: `specs/client-theme.spec.json` (tech-blue flat theme) and
 * `APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md` section 11 (cross-architecture
 * alignment). Every client root must carry the primary token in its theme layer
 * and must not fall back to the previous teal accent.
 */

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const spec = JSON.parse(
  fs.readFileSync(path.join(root, "specs", "client-theme.spec.json"), "utf8"),
);

const SKIP_DIRS = new Set([
  "node_modules",
  "build",
  "dist",
  ".dart_tool",
  ".gradle",
  ".sdkwork",
  "Pods",
  ".idea",
]);

function collectSourceFiles(relativeRoot, extensions) {
  const absoluteRoot = path.join(root, relativeRoot);
  const pending = [absoluteRoot];
  const files = [];
  while (pending.length > 0) {
    const current = pending.pop();
    let entries;
    try {
      entries = fs.readdirSync(current, { withFileTypes: true });
    } catch {
      continue;
    }
    for (const entry of entries) {
      if (SKIP_DIRS.has(entry.name)) {
        continue;
      }
      const child = path.join(current, entry.name);
      if (entry.isSymbolicLink()) {
        continue;
      }
      if (entry.isDirectory()) {
        pending.push(child);
      } else if (extensions.some((extension) => entry.name.endsWith(extension))) {
        files.push(child);
      }
    }
  }
  return files;
}

function readConcat(files) {
  return files.map((file) => fs.readFileSync(file, "utf8")).join("\n");
}

const PRIMARY_HEX = spec.color.primary.toUpperCase();

test("theme spec declares a flat, borderless, tech-blue palette", () => {
  assert.equal(spec.theme, "tech-blue-flat");
  assert.equal(spec.base, "light");
  assert.equal(spec.shape.borderWidth, 0, "decorative border width must be 0");
  assert.equal(spec.elevation.shadow, "none", "shadows must be disabled");
  assert.equal(spec.color.primary.toUpperCase(), PRIMARY_HEX);
  assert.equal(spec.color.border, "transparent");
  for (const key of ["background", "surface", "textPrimary", "textSecondary", "rail"]) {
    assert.match(spec.color[key], /^#[0-9A-Fa-f]{6}$/u, `${key} must be a hex color`);
  }
});

test("theme spec declares a complete dark scheme and icon colour contract", () => {
  assert.ok(spec.dark, "a dark token set is required");
  for (const key of ["background", "surface", "textPrimary", "textSecondary", "primary"]) {
    assert.match(spec.dark[key], /^#[0-9A-Fa-f]{6}$/u, `dark.${key} must be a hex color`);
  }
  assert.deepEqual(spec.themeMode.supported, ["light", "dark", "system"]);
  assert.equal(spec.themeMode.default, "system");
  for (const key of ["default", "strong", "active", "onPrimary", "onRail", "disabled"]) {
    assert.match(spec.iconColors[key], /^#[0-9A-Fa-f]{6}$/u, `iconColors.${key} must be a hex color`);
    assert.notEqual(
      spec.iconColors[key].toLowerCase(),
      "transparent",
      `iconColors.${key} must never be transparent`,
    );
  }
});

test("PC theme layer carries the primary token", () => {
  const source = readConcat(
    collectSourceFiles("apps/sdkwork-agents-pc/src", [".css", ".ts", ".tsx"]),
  );
  assert.ok(/165dff/iu.test(source), "PC theme must reference #165DFF");
});

test("H5 theme layer carries the primary token", () => {
  const source = readConcat(
    collectSourceFiles("apps/sdkwork-agents-h5/src", [".css", ".ts", ".tsx"]),
  );
  assert.ok(/165dff/iu.test(source), "H5 theme must reference #165DFF");
});

test("mini program theme layer carries the primary token", () => {
  const source = readConcat(
    collectSourceFiles("apps/sdkwork-agents-mini-program/src", [".wxss", ".json", ".js"]),
  );
  assert.ok(/165dff/iu.test(source), "mini program theme must reference #165DFF");
  assert.ok(!/0f766e/iu.test(source), "mini program must not keep the old teal accent");
});

test("Flutter theme layer carries the primary token and drops the teal seed", () => {
  const source = readConcat(
    collectSourceFiles("apps/sdkwork-agents-flutter-mobile/lib", [".dart"]),
  );
  const packages = readConcat(
    collectSourceFiles("apps/sdkwork-agents-flutter-mobile/packages", [".dart"]),
  );
  const combined = `${source}\n${packages}`;
  assert.ok(/0xFF165DFF/iu.test(combined), "Flutter theme must declare 0xFF165DFF");
  assert.ok(!/0xFF0F766E/iu.test(combined), "Flutter must not keep the teal seed");
});

test("HarmonyOS theme layer carries the primary token", () => {
  const source = readConcat(
    collectSourceFiles("apps/sdkwork-agents-harmony-mobile", [".ets", ".json"]),
  );
  assert.ok(/165dff/iu.test(source), "Harmony theme must reference #165DFF");
});

test("H5 exposes light, dark, and system theme modes", () => {
  const css = fs.readFileSync(
    path.join(root, "apps/sdkwork-agents-h5/src/index.css"),
    "utf8",
  );
  assert.match(css, /:root\[data-theme="dark"\]/u, "H5 needs an explicit dark token block");
  assert.match(css, /:root\[data-theme="system"\]/u, "H5 system mode needs a token block");
  assert.match(css, /prefers-color-scheme:\s*dark/u, "H5 system mode must follow the platform");
  assert.match(css, /--color-background:\s*#0a1224/iu, "H5 dark background must use the spec token");

  const theme = fs.readFileSync(
    path.join(root, "apps/sdkwork-agents-h5/src/bootstrap/theme.ts"),
    "utf8",
  );
  for (const mode of ["system", "light", "dark"]) {
    assert.match(theme, new RegExp(`"${mode}"`, "u"), `H5 theme mode must include ${mode}`);
  }
  assert.match(theme, /data-theme/u, "H5 theme must set the document data-theme attribute");
});
