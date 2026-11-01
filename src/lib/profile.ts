import { invoke } from "@tauri-apps/api/tauri";
import type { RankSummary } from "$lib/champ_select";

/** The player signed in to the League Client. */
export interface OwnProfile {
  gameName: string;
  gameTag: string;
  puuid: string;
  profileIconId: number;
  level: number;
  solo: RankSummary;
  flex: RankSummary;
}

export async function getOwnProfile(): Promise<OwnProfile> {
  return invoke<OwnProfile>("get_own_profile");
}

/**
 * "120W 98L · 55%". League reports your own losses (unlike other players'), so your record is
 * complete. Empty before any ranked games.
 */
export function formatRecord(rank: RankSummary): string {
  const winRate = recordWinRate(rank);
  return winRate ? `${rank.wins}W ${rank.losses}L · ${winRate}` : "";
}

/** "55%" this split, or empty before any ranked games. */
export function recordWinRate(rank: RankSummary): string {
  const games = rank.wins + rank.losses;
  return games === 0 ? "" : `${Math.round((rank.wins / games) * 100)}%`;
}
