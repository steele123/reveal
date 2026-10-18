export interface ChampSelect {
  participants: Participant[];
}

export interface Participant {
  cid: string;
  game_name: string;
  game_tag: string;
  muted: boolean;
  name?: string;
  pid: string;
  puuid: string;
  region: string;
  assigned_position?: string;
  cell_id?: number;
  pick_turn?: number;
  rank?: RankSummary;
  mains?: ChampionMain[];
  profile_icon_id?: number;
  recent?: RecentForm;
}

/** Solo/Duo results from the player's recent match history. */
export interface RecentForm {
  wins: number;
  losses: number;
}

/** Profile icon from Community Dragon's mirror of League's game data. */
export function profileIconUrl(iconId: number): string {
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/profile-icons/${iconId}.jpg`;
}

/** One of a player's most-played champions, by mastery points. */
export interface ChampionMain {
  championId: number;
  name: string;
  level: number;
  points: number;
}

/** Square champion icon from Community Dragon's mirror of League's game data. */
export function championIconUrl(championId: number): string {
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/${championId}.png`;
}

export function describeMain(main: ChampionMain): string {
  return `${main.name} · Mastery ${main.level} · ${main.points.toLocaleString()} pts`;
}

export interface RankSummary {
  /** "IRON" through "CHALLENGER", or "UNRANKED". */
  tier: string;
  /** "I" through "IV"; absent for Master and above. */
  division?: string;
  leaguePoints: number;
  wins: number;
  losses: number;
  provisional: boolean;
}

const TIER_COLORS: Record<string, string> = {
  IRON: "text-stone-400",
  BRONZE: "text-orange-400",
  SILVER: "text-slate-300",
  GOLD: "text-yellow-400",
  PLATINUM: "text-teal-300",
  EMERALD: "text-emerald-400",
  DIAMOND: "text-sky-300",
  MASTER: "text-fuchsia-400",
  GRANDMASTER: "text-red-400",
  CHALLENGER: "text-amber-300",
};

export interface FormattedRank {
  tier: string;
  /** Tier for the compact row, e.g. "GM" for Grandmaster. */
  shortTier: string;
  tierClass: string;
  /** Full line for the tooltip, including games played. */
  summary: string;
  /** Rank crest bundled in public/ranks (League's mini crests, via Community Dragon). */
  icon: string;
  details: string[];
}

/** Below this many recent games a win rate is noise, so show the record instead. */
export const MIN_GAMES_FOR_WIN_RATE = 10;

/**
 * Recent win rate for the compact row, e.g. "58%", or "3W 3L" for a handful of games.
 * The row has no room for the sample size; the tooltip and popup show it.
 */
export function formatRecentForm(recent: RecentForm | undefined): string {
  if (!recent) return "";
  const games = recent.wins + recent.losses;
  if (games === 0) return "";
  if (games < MIN_GAMES_FOR_WIN_RATE) return `${recent.wins}W ${recent.losses}L`;
  return `${Math.round((recent.wins / games) * 100)}%`;
}

/**
 * e.g. { tier: "Diamond II", details: ["45 LP", "58% last 19"] }.
 * League reports other players' losses as 0, so the season record is wins only and the
 * win rate comes from recent Solo/Duo match history instead.
 */
export function formatRank(
  rank: RankSummary,
  recent?: RecentForm,
): FormattedRank {
  const recentLabel = formatRecentForm(recent);
  const recentSummary =
    recent && recent.wins + recent.losses > 0
      ? `win rate from last ${recent.wins + recent.losses} Solo/Duo games: ${recent.wins}W ${recent.losses}L`
      : "";
  const seasonWins = rank.wins > 0 ? `${rank.wins} wins this season` : "";

  if (rank.tier === "UNRANKED") {
    const tier = rank.provisional && rank.wins > 0 ? "Placements" : "Unranked";
    return {
      tier,
      shortTier: tier,
      tierClass: "text-muted-foreground",
      icon: "/ranks/unranked.svg",
      details: [recentLabel].filter(Boolean),
      summary: [tier, seasonWins, recentSummary].filter(Boolean).join(" · "),
    };
  }

  const tierName = rank.tier.charAt(0) + rank.tier.slice(1).toLowerCase();
  const tier = rank.division ? `${tierName} ${rank.division}` : tierName;
  // "Grandmaster · 600 LP · 58%" is the one rank line too wide for a two-column row.
  const shortTier = rank.tier === "GRANDMASTER" ? "GM" : tier;
  const lp = `${rank.leaguePoints} LP`;
  return {
    tier,
    shortTier,
    tierClass: TIER_COLORS[rank.tier] ?? "text-foreground",
    icon: TIER_COLORS[rank.tier]
      ? `/ranks/${rank.tier.toLowerCase()}.svg`
      : "/ranks/unranked.svg",
    details: [lp, recentLabel].filter(Boolean),
    summary: [tier, lp, seasonWins, recentSummary].filter(Boolean).join(" · "),
  };
}

const POSITION_BADGES: Record<string, string> = {
  top: "TOP",
  jungle: "JGL",
  middle: "MID",
  bottom: "ADC",
  utility: "SUP",
};

/** Short role label for the row badge, e.g. "JGL". */
export function positionBadge(position: string): string {
  return POSITION_BADGES[position.toLowerCase()] ?? position.slice(0, 3).toUpperCase();
}

const POSITION_LABELS: Record<string, string> = {
  top: "Top",
  jungle: "Jungle",
  middle: "Mid",
  bottom: "ADC",
  utility: "Support",
};

export function formatAssignedPosition(position: string): string {
  return (
    POSITION_LABELS[position.toLowerCase()] ||
    `${position.slice(0, 1).toUpperCase()}${position.slice(1)}`
  );
}
