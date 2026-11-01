import { invoke } from "@tauri-apps/api/tauri";
import type { Participant } from "$lib/champ_select";
import { isTauriRuntime } from "$lib/runtime";

/**
 * Master+ counters and lane matchups for the enemy picks, as the backend sends them. Both
 * Lolalytics and U.GG are read (World, Master and above, Ranked Solo/Duo) and weighed together;
 * links go back to the main site, with the others alongside to compare.
 */

/** League's position names, as Champ Select reports them. */
export type Role = "top" | "jungle" | "middle" | "bottom" | "utility";

/**
 * The site whose win rate is shown first; both are always read. League of Graphs blocks apps,
 * so it's only a link.
 */
export type CountersProvider = "ugg" | "lolalytics";

export const COUNTERS_PROVIDERS: ReadonlyArray<{ label: string; value: CountersProvider }> = [
  { label: "Lolalytics", value: "lolalytics" },
  { label: "U.GG", value: "ugg" },
];

export interface ChampionRef {
  id: number;
  name: string;
  /** The name stats sites use in URLs, e.g. "leesin"; empty when the client didn't say. */
  slug: string;
}

/** One site's numbers for a matchup. */
export interface SourceStats {
  games: number;
  winRate: number;
  winRateDelta: number | null;
}

/**
 * One side of a matchup, per game: the main site's win rate (or the other's, when the main one
 * lacks it), U.GG's leads at 15 minutes (negative when behind), and each site's own numbers.
 */
export interface MatchupStats {
  games: number;
  /** 0-100. */
  winRate: number;
  /**
   * Win rate minus what both champions' overall win rates in the role predict, in points
   * (like Lolalytics' Δ2). Null when either champion has no games in the role.
   */
  winRateDelta: number | null;
  goldDiff15: number | null;
  xpDiff15: number | null;
  csDiff15: number | null;
  jungleCsDiff15: number | null;
  killDiff15: number | null;
  lolalytics: SourceStats | null;
  ugg: SourceStats | null;
  /** How many sites see the matchup going better than expected, out of `sources`. */
  agreement: number;
  /** How many sites' stats for the enemy pick have loaded. */
  sources: number;
}

export interface Counter {
  champion: ChampionRef;
  /** From the counter pick's side. */
  stats: MatchupStats;
}

export interface AllyMatchup {
  champion: ChampionRef;
  isSelf: boolean;
  /** From your teammate's side; null when U.GG has no Master+ games of the matchup. */
  stats: MatchupStats | null;
}

export interface EnemyMatchups {
  champion: ChampionRef;
  role: Role | null;
  /** Riot hid the role, so it's the likeliest one given what every enemy pick usually plays. */
  roleGuessed: boolean;
  /** The enemy champion's own games and win rate in the role. */
  games: number;
  winRate: number | null;
  /** Its matchups are still downloading. */
  loading: boolean;
  /** Best first. */
  counters: Counter[];
  /** Your team's pick in the same role. */
  ally: AllyMatchup | null;
}

export interface AllyPick {
  champion: ChampionRef;
  role: Role;
  isSelf: boolean;
}

export interface MatchupBoard {
  provider: CountersProvider;
  /** e.g. "26.21". */
  patch: string | null;
  error: string | null;
  selfRole: Role | null;
  /** Your team's picks, hovers included. */
  allies: AllyPick[];
  /** In lane order once roles are known. */
  enemies: EnemyMatchups[];
}

export const LANES: Role[] = ["top", "jungle", "middle", "bottom", "utility"];

/**
 * One lane of the draft: your teammate (name and pick) against the enemy pick, plus the enemy
 * player when Riot shows their name. Players and picks without a known lane get rows of their
 * own after the five lanes.
 */
export interface DraftLane {
  role: Role | null;
  player: Participant | null;
  pick: AllyPick | null;
  enemy: EnemyMatchups | null;
  enemyPlayer: Participant | null;
  isSelf: boolean;
}

/**
 * The enemy picks with the lanes the user dragged them to. A pick already in that lane swaps
 * into the dragged pick's old lane until the backend sends its own guess.
 */
export function withRoleOverrides(
  enemies: EnemyMatchups[],
  overrides: ReadonlyMap<number, Role>,
): EnemyMatchups[] {
  let placed = enemies;
  for (const [championId, role] of overrides) {
    const moved = placed.find((enemy) => enemy.champion.id === championId);
    if (!moved || moved.role === role) continue;
    const from = moved.role;
    placed = placed.map((enemy) => {
      if (enemy === moved) return { ...enemy, role, roleGuessed: false };
      if (enemy.role === role) return { ...enemy, role: from };
      return enemy;
    });
  }
  return placed;
}

let previewRoleHandler: ((championId: number, role: Role) => void) | null = null;

/** In the browser preview, where there's no backend, what to do when a pick is dragged. */
export function setPreviewRoleHandler(handler: (championId: number, role: Role) => void) {
  previewRoleHandler = handler;
}

/** Tells the backend an enemy pick plays `role`, so its counters are worked out for that lane. */
export async function setEnemyRole(championId: number, role: Role): Promise<void> {
  if (!isTauriRuntime()) {
    previewRoleHandler?.(championId, role);
    return;
  }
  await invoke("set_enemy_role", { championId, role });
}

export function draftLanes(
  allies: Participant[],
  enemyPlayers: Participant[],
  board: MatchupBoard | null,
  overrides: ReadonlyMap<number, Role> = new Map(),
): DraftLane[] {
  const inRole = (player: Participant, role: Role) => player.assigned_position === role;
  const enemies = withRoleOverrides(board?.enemies ?? [], overrides);
  const lanes = LANES.map((role) => {
    const player = allies.find((ally) => inRole(ally, role)) ?? null;
    return {
      role,
      player,
      pick: board?.allies.find((ally) => ally.role === role) ?? null,
      enemy: enemies.find((enemy) => enemy.role === role) ?? null,
      enemyPlayer: enemyPlayers.find((enemy) => inRole(enemy, role)) ?? null,
      isSelf: player ? Boolean(player.is_self) : board?.selfRole === role,
    };
  });
  const isLane = (position: string | undefined) => LANES.some((role) => role === position);
  const unplacedPlayers = allies
    .filter((ally) => !isLane(ally.assigned_position))
    .map((player) => ({
      role: null,
      player,
      pick: null,
      enemy: null,
      enemyPlayer: null,
      isSelf: Boolean(player.is_self),
    }));
  const unplacedEnemies = enemies
    .filter((enemy) => enemy.role === null)
    .map((enemy) => ({
      role: null,
      player: null,
      pick: null,
      enemy,
      enemyPlayer: null,
      isSelf: false,
    }));
  return [...lanes, ...unplacedPlayers, ...unplacedEnemies];
}

/** The enemy whose counters to show: the one picked, else the one in your lane, else the first. */
export function focusedEnemy(
  rows: DraftLane[],
  pickedId: number | null,
): EnemyMatchups | null {
  const enemies = rows.filter((row) => row.enemy);
  return (
    enemies.find((row) => row.enemy?.champion.id === pickedId)?.enemy ??
    enemies.find((row) => row.isSelf)?.enemy ??
    enemies[0]?.enemy ??
    null
  );
}

/** Matchups with fewer games are still shown, but marked as a small sample. */
export const MIN_RELIABLE_GAMES = 200;

const UGG_ROLES: Record<Role, string> = {
  top: "top",
  jungle: "jungle",
  middle: "mid",
  bottom: "adc",
  utility: "support",
};

const LOLALYTICS_LANES: Record<Role, string> = {
  top: "top",
  jungle: "jungle",
  middle: "middle",
  bottom: "bottom",
  utility: "support",
};

const LEAGUE_OF_GRAPHS_ROLES: Record<Role, string> = {
  top: "top",
  jungle: "jungle",
  middle: "middle",
  bottom: "adc",
  utility: "support",
};

/** U.GG's World Master+ counter page for `enemy`, where Reveal's counter numbers come from. */
export function uggCountersLink(enemy: ChampionRef, role: Role): string {
  return `https://u.gg/lol/champions/${enemy.slug}/counter?role=${UGG_ROLES[role]}&rank=master_plus`;
}

/** U.GG's Master+ build page for `champion` into `opponent`, with that matchup's numbers. */
export function uggMatchupLink(champion: ChampionRef, opponent: ChampionRef, role: Role): string {
  return `https://u.gg/lol/champions/${champion.slug}/build/${UGG_ROLES[role]}?opp=${opponent.slug}&rank=master_plus`;
}

/** Lolalytics' Master+ page for `champion` into `opponent`, with that matchup's numbers. */
export function lolalyticsMatchupLink(
  champion: ChampionRef,
  opponent: ChampionRef,
  role: Role,
): string {
  const lane = LOLALYTICS_LANES[role];
  return `https://lolalytics.com/lol/${champion.slug}/vs/${opponent.slug}/build/?lane=${lane}&vslane=${lane}&tier=master_plus`;
}

/** The matchup's page on the site the numbers came from. */
export function matchupLink(
  provider: CountersProvider,
  champion: ChampionRef,
  opponent: ChampionRef,
  role: Role,
): string {
  return provider === "lolalytics"
    ? lolalyticsMatchupLink(champion, opponent, role)
    : uggMatchupLink(champion, opponent, role);
}

export function lolalyticsCountersLink(enemy: ChampionRef, role: Role): string {
  return `https://lolalytics.com/lol/${enemy.slug}/counters/?lane=${LOLALYTICS_LANES[role]}&tier=master_plus`;
}

export function leagueOfGraphsCountersLink(enemy: ChampionRef, role: Role): string {
  return `https://www.leagueofgraphs.com/champions/counters/${enemy.slug}/${LEAGUE_OF_GRAPHS_ROLES[role]}/master/sr-ranked`;
}

/** "+452", "-0.4": a lead with its sign, rounded to `digits` decimals; "–" when unknown. */
export function formatLead(value: number | null, digits = 0): string {
  if (value === null) return "–";
  const rounded = Number(value.toFixed(digits));
  if (rounded === 0) return (0).toFixed(digits);
  return `${rounded > 0 ? "+" : "-"}${Math.abs(rounded).toFixed(digits)}`;
}

/** "56.5%". */
export function formatWinRate(winRate: number): string {
  return `${winRate.toFixed(1)}%`;
}

/** Farm lead at 15 minutes: lane minions plus jungle camps. */
export function csLead(stats: MatchupStats): number | null {
  if (stats.csDiff15 === null || stats.jungleCsDiff15 === null) return null;
  return stats.csDiff15 + stats.jungleCsDiff15;
}

/**
 * How much better than expected the matchup goes, in win rate points: the Δ when both
 * champions have games in the role, or else the distance from 50%.
 */
export function winRateEdge(stats: MatchupStats): number {
  return stats.winRateDelta ?? stats.winRate - 50;
}

/** Green when ahead, red when behind; leads within `deadZone` of zero stay neutral. */
export function leadClass(value: number | null, deadZone = 0): string {
  if (value === null || Math.abs(value) <= deadZone) return "text-muted-foreground";
  return value > 0 ? "text-emerald-300" : "text-red-300";
}

/** Every number of a matchup, for its tooltip. */
export function describeMatchup(
  stats: MatchupStats,
  champion: ChampionRef,
  opponent: ChampionRef,
): string {
  const delta =
    stats.winRateDelta === null ? "" : ` (${formatLead(stats.winRateDelta, 1)} vs expected)`;
  const site = (label: string, side: SourceStats | null) =>
    side
      ? `${label}: ${formatWinRate(side.winRate)}${side.winRateDelta === null ? "" : ` (${formatLead(side.winRateDelta, 1)} vs expected)`}, ${side.games.toLocaleString("en-US")} games`
      : `${label}: not enough games`;
  return [
    `${champion.name} vs ${opponent.name}: ${formatWinRate(stats.winRate)} win rate${delta}, ${stats.games.toLocaleString("en-US")} Master+ games`,
    stats.sources > 1 ? site("Lolalytics", stats.lolalytics) : "",
    stats.sources > 1 ? site("U.GG", stats.ugg) : "",
    stats.goldDiff15 === null
      ? "No U.GG leads at 15 min"
      : `At 15 min: ${formatLead(stats.killDiff15, 2)} kills, ${formatLead(csLead(stats), 1)} CS (${formatLead(stats.jungleCsDiff15, 1)} jungle), ${formatLead(stats.goldDiff15)} gold, ${formatLead(stats.xpDiff15)} XP`,
    stats.games < MIN_RELIABLE_GAMES ? "Small sample, so take it with a grain of salt" : "",
  ]
    .filter(Boolean)
    .join("\n");
}
