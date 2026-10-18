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
  /** Rank crest bundled in public/ranks (League's mini crests, via Community Dragon). */
  icon: string;
  details: string[];
}

/** e.g. { tier: "Diamond II", details: ["45 LP", "58%", "120g"] } */
export function formatRank(rank: RankSummary): FormattedRank {
  const games = rank.wins + rank.losses;
  const winRate = games > 0 ? `${Math.round((rank.wins / games) * 100)}%` : "";
  const gamesLabel = games > 0 ? `${games}g` : "";

  if (rank.tier === "UNRANKED") {
    return {
      tier: rank.provisional && games > 0 ? "Placements" : "Unranked",
      tierClass: "text-muted-foreground",
      icon: "/ranks/unranked.svg",
      details: [winRate, gamesLabel].filter(Boolean),
    };
  }

  const tierName = rank.tier.charAt(0) + rank.tier.slice(1).toLowerCase();
  return {
    tier: rank.division ? `${tierName} ${rank.division}` : tierName,
    tierClass: TIER_COLORS[rank.tier] ?? "text-foreground",
    icon: TIER_COLORS[rank.tier]
      ? `/ranks/${rank.tier.toLowerCase()}.svg`
      : "/ranks/unranked.svg",
    details: [`${rank.leaguePoints} LP`, winRate, gamesLabel].filter(Boolean),
  };
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
