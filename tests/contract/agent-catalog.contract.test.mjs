import assert from "node:assert/strict";
import test from "node:test";

import { verifyAgentCatalog } from "../../scripts/verify-agent-catalog.mjs";

test("official agent catalog covers the professional industry matrix", () => {
  const result = verifyAgentCatalog();
  assert.equal(result.agents, 64);
  assert.equal(result.localExecutionAgents, 7);
  assert.equal(result.mcpServers, 18);
  assert.deepEqual(result.agentKinds, ["conversation", "local-execution"]);
  assert.deepEqual(result.runtimeFamilies, ["harness", "rig"]);
  assert.deepEqual(result.categories, [
    "data-ai",
    "ecommerce",
    "education-higher",
    "education-k12",
    "education-training",
    "engineering",
    "enterprise",
    "finance",
    "government",
    "healthcare",
    "hospitality",
    "hr",
    "legal",
    "local-execution",
    "manufacturing",
    "marketing",
    "media",
    "operations",
    "product",
    "realestate",
    "restaurant",
    "sales",
    "supply-chain",
    "travel",
    "writing",
  ]);
});

test("catalog seeds are deterministic and locale manifests carry checksums", () => {
  const first = verifyAgentCatalog();
  const second = verifyAgentCatalog();
  assert.deepEqual(first, second);
});
