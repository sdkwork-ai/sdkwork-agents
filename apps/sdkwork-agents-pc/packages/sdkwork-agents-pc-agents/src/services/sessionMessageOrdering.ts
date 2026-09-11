import type { AgentSessionItemRecord } from "@sdkwork/agents-pc-core/sdk/agentsAppSdkClient";

function sessionItemKindOrder(kind: AgentSessionItemRecord["kind"]): number {
  switch (kind) {
    case "user_input":
      return 10;
    case "system_instruction":
    case "status_notice":
      return 15;
    case "assistant_output":
    case "reasoning":
      return 20;
    case "tool_call":
      return 30;
    case "tool_result":
      return 40;
    case "error_notice":
      return 50;
    default:
      return 60;
  }
}

function compareInt64String(left: string, right: string): number {
  if (left === right) {
    return 0;
  }
  const leftValue = BigInt(left);
  const rightValue = BigInt(right);
  if (leftValue < rightValue) {
    return -1;
  }
  if (leftValue > rightValue) {
    return 1;
  }
  return 0;
}

export function compareSessionItems(
  left: AgentSessionItemRecord,
  right: AgentSessionItemRecord,
): number {
  const sequenceOrder = compareInt64String(left.sequence, right.sequence);
  if (sequenceOrder !== 0) {
    return sequenceOrder;
  }
  const createdAtOrder = left.createdAt.localeCompare(right.createdAt);
  if (createdAtOrder !== 0) {
    return createdAtOrder;
  }
  return sessionItemKindOrder(left.kind) - sessionItemKindOrder(right.kind);
}

export function sortSessionItems(
  items: AgentSessionItemRecord[],
): AgentSessionItemRecord[] {
  return [...items].sort(compareSessionItems);
}
