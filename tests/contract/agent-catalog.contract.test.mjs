import assert from "node:assert/strict";
import test from "node:test";

import { verifyAgentCatalog } from "../../scripts/verify-agent-catalog.mjs";

test("official agent catalog covers the commercial industry matrix", () => {
  const result = verifyAgentCatalog();
  assert.equal(result.agents, 64);
  assert.equal(result.mcpServers, 18);
  assert.deepEqual(result.categories, [
    "ecommerce",
    "education",
    "engineering",
    "enterprise",
    "government",
    "healthcare",
    "hospitality",
    "legal",
    "realestate",
    "restaurant",
    "travel",
    "writing",
  ]);
});

test("catalog seeds are deterministic and locale manifests carry checksums", () => {
  const first = verifyAgentCatalog();
  const second = verifyAgentCatalog();
  assert.deepEqual(first, second);
});
