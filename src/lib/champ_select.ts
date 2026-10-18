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
  tierClass: string;
  /** Full line for the tooltip, including games played. */
  summary: string;
  /** Rank crest bundled in public/ranks (League's mini crests, via Community Dragon). */
  icon: string;
  details: string[];
}

/** Below this many games (or during placements) a win rate is noise, so show the record instead. */
export const MIN_GAMES_FOR_WIN_RATE = 10;

/** e.g. { tier: "Diamond II", details: ["45 LP", "58%"], summary: "... · 121 games" } */
export function formatRank(rank: RankSummary): FormattedRank {
  const games = rank.wins + rank.losses;
  // Early in a split most players have a handful of games, where "100%" just means 2-0.
  const winRate =
    games === 0
      ? ""
      : games < MIN_GAMES_FOR_WIN_RATE || rank.provisional
        ? `${rank.wins}W ${rank.losses}L`
        : `${Math.round((rank.wins / games) * 100)}%`;
  const record = games > 0 ? `${rank.wins}W ${rank.losses}L (${games} games)` : "";

  if (rank.tier === "UNRANKED") {
    const tier = rank.provisional && games > 0 ? "Placements" : "Unranked";
    return {
      tier,
      tierClass: "text-muted-foreground",
      icon: "/ranks/unranked.svg",
      details: [winRate].filter(Boolean),
      summary: [tier, record].filter(Boolean).join(" · "),
    };
  }

  const tierName = rank.tier.charAt(0) + rank.tier.slice(1).toLowerCase();
  const tier = rank.division ? `${tierName} ${rank.division}` : tierName;
  const lp = `${rank.leaguePoints} LP`;
  return {
    tier,
    tierClass: TIER_COLORS[rank.tier] ?? "text-foreground",
    icon: TIER_COLORS[rank.tier]
      ? `/ranks/${rank.tier.toLowerCase()}.svg`
      : "/ranks/unranked.svg",
    details: [lp, winRate].filter(Boolean),
    summary: [tier, lp, record].filter(Boolean).join(" · "),
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
