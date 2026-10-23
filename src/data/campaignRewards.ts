/**
 * Galactic Campaign rewards by the item id the game sends with each campaign. The game API doesn't
 * name these items, so they are listed here by hand; add new campaigns' rewards as they are announced.
 */
export interface CampaignReward {
  /** What the in-game Control Center calls the reward type, e.g. "Stratagem". */
  kind: string;
  /** Item name, once known. */
  name?: string;
}

export const CAMPAIGN_REWARDS: Record<number, CampaignReward> = {
  2768520552: { kind: "Stratagem", name: "M-103 Supply FRV" }, // Census Thunder
  2232169940: { kind: "Primary weapon", name: "R-4 Hyena" }, // Celestial Fence
  1407130184: { kind: "Helmet", name: "KDM-500 Outrider" }, // Lightning Intercept
  3421347422: { kind: "Armor", name: "KDM-500 Outrider" }, // Permanent Enclosure
  2547900634: { kind: "Helmet", name: "IX-Voidwalker" }, // Void Piercer
  2961949068: { kind: "Stratagem", name: "Eagle Gas Strike" }, // Counterdissident Hammer
  3890208935: { kind: "Cape", name: "Message of Morale" }, // Blazing Electorate
  1618361436: { kind: "Stratagem", name: "TD-110 Maelstrom" }, // Armored Eagle
  2237630340: { kind: "Tactical action", name: "Eagle Gas Storm" }, // Epic Purge
  2659326411: { kind: "Stratagem" }, // Memoriam Eternal
};

/** Display name for a campaign's reward: the item if known, else its type, else a generic label. */
export function rewardLabel(mixId: number | null): string {
  const r = mixId == null ? undefined : CAMPAIGN_REWARDS[mixId];
  return r?.name ?? r?.kind ?? "Campaign reward";
}
