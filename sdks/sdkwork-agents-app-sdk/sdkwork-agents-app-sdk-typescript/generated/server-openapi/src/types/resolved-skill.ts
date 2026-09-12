/** One enabled skill attached to an agent. */
export interface ResolvedSkill {
  slotId: string;
  targetRef: string;
  title?: string | null;
  instructions?: string | null;
}
