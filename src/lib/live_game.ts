import type { RankSummary, RecentForm } from "$lib/champ_select";

/** Scoreboard from League's Live Client Data API, sent by the backend every few seconds. */
export interface LiveGame {
  gameTime: number;
  gameMode: string;
  players: LivePlayer[];
}

export interface LivePlayer {
  gameName: string;
  tagLine: string;
  /** "ORDER" (blue side) or "CHAOS" (red side). */
  team: string;
  isSelf: boolean;
  isBot: boolean;
  championName: string;
  championId: number | null;
  position: string;
  level: number;
  kills: number;
  deaths: number;
  assists: number;
  creepScore: number;
  isDead: boolean;
  respawnTimer: number;
  items: LiveItem[];
  puuid: string | null;
  rank: RankSummary | null;
  recent: RecentForm | null;
}

export interface LiveItem {
  id: number;
  name: string;
  slot: number;
  icon: string | null;
}

const POSITION_ORDER = ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"];

/** Splits players into your team and the enemy team, each ordered top to support. */
export function splitTeams(game: LiveGame): { ally: LivePlayer[]; enemy: LivePlayer[] } {
  const myTeam = game.players.find((player) => player.isSelf)?.team ?? "ORDER";
  const byPosition = (a: LivePlayer, b: LivePlayer) =>
    rankOf(a.position) - rankOf(b.position);
  return {
    ally: game.players.filter((player) => player.team === myTeam).sort(byPosition),
    enemy: game.players.filter((player) => player.team !== myTeam).sort(byPosition),
  };
}

function rankOf(position: string): number {
  const index = POSITION_ORDER.indexOf(position);
  return index === -1 ? POSITION_ORDER.length : index;
}

/** 812 -> "13:32". */
export function formatGameTime(seconds: number): string {
  const whole = Math.max(0, Math.floor(seconds));
  return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, "0")}`;
}

export function teamKills(players: LivePlayer[]): number {
  return players.reduce((total, player) => total + player.kills, 0);
}
