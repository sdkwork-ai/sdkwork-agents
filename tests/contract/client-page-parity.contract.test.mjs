import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

/**
 * Cross-client page parity contract.
 *
 * Authority: `APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md` section 7 (route
 * identity) and section 11 (cross-architecture alignment), plus
 * `specs/client-page-parity.spec.json`.
 *
 * The PC workbench is the reference page system. Every mobile root must declare
 * the same `app.agents.*` page identities, even where the physical path,
 * container, or presentation differs.
 */

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const contractPath = path.join(root, "specs", "client-page-parity.spec.json");
const contract = JSON.parse(fs.readFileSync(contractPath, "utf8"));

const CLIENT_KEYS = ["pc", "h5", "miniProgram", "flutter", "harmony"];

const ALLOWED_PRESENTATION = {
  pc: new Set(["tab", "page", "dialog", "overlay", "drawer"]),
  h5: new Set(["tab", "stack", "modal", "sheet"]),
  miniProgram: new Set(["page", "subpackagePage"]),
  flutter: new Set(["tab", "route", "bottomSheet", "dialog"]),
  harmony: new Set(["tab", "page", "dialog", "sheet"]),
};

function readRegistry(clientKey) {
  const client = contract.clients[clientKey];
  const contents = client.registry.map((relativePath) => {
    const absolute = path.join(root, relativePath);
    assert.ok(fs.existsSync(absolute), `${relativePath} must exist for ${clientKey}`);
    return fs.readFileSync(absolute, "utf8");
  });
  return contents.join("\n");
}

function extractPageIds(source) {
  const matches = source.matchAll(/app\.agents\.[a-z0-9]+(?:\.[a-z0-9]+)+/gu);
  return new Set([...matches].map((match) => match[0]));
}

test("every page id is a four-segment app.agents route identity", () => {
  const seen = new Set();
  for (const page of contract.pages) {
    const segments = page.id.split(".");
    assert.equal(segments.length, 4, `${page.id} must have surface.domain.capability.screen`);
    assert.equal(segments[0], "app", `${page.id} must use the app surface`);
    assert.equal(segments[1], "agents", `${page.id} must use the agents domain`);
    assert.ok(!seen.has(page.id), `${page.id} is declared twice`);
    seen.add(page.id);
  }
});

test("every page declares a known presentation per client", () => {
  for (const page of contract.pages) {
    assert.ok(typeof page.titleKey === "string" && page.titleKey.length > 0, `${page.id} needs a titleKey`);
    for (const clientKey of CLIENT_KEYS) {
      const entry = page[clientKey];
      if (entry === null || entry === undefined) {
        continue;
      }
      const allowed = ALLOWED_PRESENTATION[clientKey];
      assert.ok(
        allowed.has(entry.presentation),
        `${page.id} declares unsupported ${clientKey} presentation "${entry.presentation}"`,
      );
    }
  }
});

test("the PC reference declares every page it owns", () => {
  const pcIds = extractPageIds(readRegistry("pc"));
  const missing = contract.pages
    .filter((page) => page.pc !== null && page.pc !== undefined)
    .map((page) => page.id)
    .filter((id) => !pcIds.has(id));
  assert.deepEqual(missing, [], `PC registry is missing page ids: ${missing.join(", ")}`);
});

test("every client declares the M1 page ids", () => {
  const required = contract.pages
    .filter((page) => page.requiredFrom === "M1")
    .map((page) => page.id);

  const failures = [];
  for (const clientKey of ["h5", "miniProgram", "flutter", "harmony"]) {
    const ids = extractPageIds(readRegistry(clientKey));
    const missing = required.filter((id) => !ids.has(id));
    if (missing.length > 0) {
      failures.push(`${clientKey}: ${missing.join(", ")}`);
    }
  }
  assert.deepEqual(failures, [], `Client roots missing M1 page ids:\n${failures.join("\n")}`);
});

test("clients that declare M2 deliver every M2 page id", () => {
  const required = contract.pages
    .filter((page) => page.requiredFrom === "M2")
    .map((page) => page.id);

  const failures = [];
  for (const clientKey of CLIENT_KEYS) {
    if (contract.clients[clientKey].deliveredThrough !== "M2") {
      continue;
    }
    const ids = extractPageIds(readRegistry(clientKey));
    const missing = required.filter((id) => !ids.has(id));
    if (missing.length > 0) {
      failures.push(`${clientKey}: ${missing.join(", ")}`);
    }
  }
  assert.deepEqual(failures, [], `Client roots missing M2 page ids:\n${failures.join("\n")}`);
});

test("M1 page ids are also present in every registry as source literals", () => {
  // Guards against an id being assembled at runtime from fragments: the parity
  // sweep reads source text, so a computed id would silently hide a gap.
  for (const clientKey of CLIENT_KEYS) {
    const source = readRegistry(clientKey);
    assert.match(source, /app\.agents\.[a-z0-9]+\.[a-z0-9]+/u, `${clientKey} registry must declare literal ids`);
  }
});
