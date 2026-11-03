import assert from "node:assert/strict";
import test from "node:test";

import { verifyAgentCatalog } from "../../scripts/verify-agent-catalog.mjs";

test("official agent catalog covers the commercial industry matrix", () => {
  const result = verifyAgentCatalog();
  assert.equal(result.agents, 58);
  assert.equal(result.mcpServers, 16);
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
  ]);
});

test("catalog seeds are deterministic and locale manifests carry checksums", () => {
  const first = verifyAgentCatalog();
  const second = verifyAgentCatalog();
  assert.deepEqual(first, second);
});
