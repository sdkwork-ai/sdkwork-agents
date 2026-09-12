import type {
  AgentCompositionSlotRecord,
  SdkworkAgentsAppClient,
} from "@sdkwork/agents-pc-core/sdk/agentsAppSdkClient";
import { syncAllOffsetPages } from "@sdkwork/agents-pc-core/sdk/pagination";

import type { AgentConfig } from "./AgentService";

type CompositionSlotKind =
  | "memory"
  | "knowledge"
  | "skill"
  | "prompt"
  | "drive"
  | "tool"
  | "mcp";
type CompositionTargetModule =
  | "memory"
  | "knowledgebase"
  | "skills"
  | "prompts"
  | "drive"
  | "tools"
  | "mcp";

interface DesiredCompositionSlot {
  slotId: string;
  slotKind: CompositionSlotKind;
  targetModule: CompositionTargetModule;
  targetRef: string;
}

function slotIdForRef(targetRef: string): string {
  const normalized = targetRef.replace(/[^a-zA-Z0-9._-]+/g, "-").replace(/^-+|-+$/g, "");
  return `slot.${normalized || "resource"}`;
}

function buildDesiredCompositionSlots(config: AgentConfig): DesiredCompositionSlot[] {
  const slots: DesiredCompositionSlot[] = [];

  for (const knowledgeId of config.knowledgeBaseIds ?? []) {
    const targetRef = knowledgeId.startsWith("kb.") ? knowledgeId : `kb.space.${knowledgeId}`;
    slots.push({
      slotId: slotIdForRef(targetRef),
      slotKind: "knowledge",
      targetModule: "knowledgebase",
      targetRef,
    });
  }

  for (const skillId of config.skillIds ?? []) {
    const targetRef = skillId.startsWith("skill.") ? skillId : `skill.${skillId}`;
    slots.push({
      slotId: slotIdForRef(targetRef),
      slotKind: "skill",
      targetModule: "skills",
      targetRef,
    });
  }

  for (const toolId of config.toolIds ?? []) {
    const targetRef = toolId.includes(".") ? toolId : `tool.${toolId}`;
    slots.push({
      slotId: slotIdForRef(targetRef),
      slotKind: "tool",
      targetModule: "tools",
      targetRef,
    });
  }

  for (const serverKey of config.mcpServerKeys ?? []) {
    const targetRef = serverKey.includes(".") ? serverKey : serverKey;
    slots.push({
      slotId: slotIdForRef(`mcp.${targetRef}`),
      slotKind: "mcp",
      targetModule: "mcp",
      targetRef,
    });
  }

  return slots;
}

export async function syncAgentCompositionSlots(
  client: SdkworkAgentsAppClient,
  agentId: string,
  config: AgentConfig,
): Promise<void> {
  const desired = buildDesiredCompositionSlots(config);
  const desiredIds = new Set(desired.map((slot) => slot.slotId));

  // Batch diff sync on agent save — `syncAllOffsetPages` is allowed per `PAGINATION_SPEC.md` §7.
  const existingItems = await syncAllOffsetPages<AgentCompositionSlotRecord>(
    (params) => client.ai.agents.compositionSlots.list(agentId, params),
    {},
  );

  for (const item of existingItems) {
    if (!desiredIds.has(item.slotId)) {
      await client.ai.agents.compositionSlots.delete(agentId, item.slotId);
    }
  }

  const existingById = new Map(existingItems.map((item) => [item.slotId, item]));
  for (const [index, slot] of desired.entries()) {
    const current = existingById.get(slot.slotId);
    if (current) {
      continue;
    }

    await client.ai.agents.compositionSlots.create(agentId, {
      slotId: slot.slotId,
      slotKind: slot.slotKind,
      targetModule: slot.targetModule,
      targetRef: slot.targetRef,
      priority: index + 1,
      enabled: true,
      policyJson: "{}",
      requestedAt: new Date().toISOString(),
    });
  }
}
