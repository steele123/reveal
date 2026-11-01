import { describe, expect, it } from "vitest";
import type { Participant } from "$lib/champ_select";
import {
  csLead,
  describeMatchup,
  focusedEnemy,
  formatLead,
  draftLanes,
  leadClass,
  leagueOfGraphsCountersLink,
  lolalyticsCountersLink,
  matchupLink,
  uggCountersLink,
  uggMatchupLink,
  winRateEdge,
  withRoleOverrides,
  type ChampionRef,
  type EnemyMatchups,
  type MatchupBoard,
  type MatchupStats,
  type Role,
} from "$lib/matchups";

const leeSin: ChampionRef = { id: 64, name: "Lee Sin", slug: "leesin" };
const rekSai: ChampionRef = { id: 421, name: "Rek'Sai", slug: "reksai" };
const jarvan: ChampionRef = { id: 59, name: "Jarvan IV", slug: "jarvaniv" };

// Rek'Sai into Lee Sin, World Master+ on patch 26.21.
const stats: MatchupStats = {
  games: 644,
  winRate: 56.52,
  winRateDelta: 3.58,
  goldDiff15: 81.7,
  xpDiff15: -1.1,
  csDiff15: 0.24,
  jungleCsDiff15: -1.56,
  killDiff15: 0.14,
  lolalytics: null,
  ugg: { games: 644, winRate: 56.52, winRateDelta: 3.58 },
  agreement: 1,
  sources: 1,
};

describe("matchup links", () => {
  it("opens U.GG's Master+ counter and matchup pages", () => {
    expect(uggCountersLink(leeSin, "jungle")).toBe(
      "https://u.gg/lol/champions/leesin/counter?role=jungle&rank=master_plus",
    );
    expect(uggMatchupLink(rekSai, leeSin, "jungle")).toBe(
      "https://u.gg/lol/champions/reksai/build/jungle?opp=leesin&rank=master_plus",
    );
  });

  it("uses each site's own role names", () => {
    expect(uggCountersLink(leeSin, "middle")).toBe(
      "https://u.gg/lol/champions/leesin/counter?role=mid&rank=master_plus",
    );
    expect(uggMatchupLink(rekSai, leeSin, "utility")).toBe(
      "https://u.gg/lol/champions/reksai/build/support?opp=leesin&rank=master_plus",
    );
    expect(lolalyticsCountersLink(leeSin, "bottom")).toBe(
      "https://lolalytics.com/lol/leesin/counters/?lane=bottom&tier=master_plus",
    );
    expect(leagueOfGraphsCountersLink(jarvan, "jungle")).toBe(
      "https://www.leagueofgraphs.com/champions/counters/jarvaniv/jungle/master/sr-ranked",
    );
    expect(leagueOfGraphsCountersLink(leeSin, "bottom")).toBe(
      "https://www.leagueofgraphs.com/champions/counters/leesin/adc/master/sr-ranked",
    );
  });
});

describe("providers", () => {
  it("opens the matchup on the site its numbers came from", () => {
    expect(matchupLink("ugg", rekSai, leeSin, "jungle")).toBe(
      "https://u.gg/lol/champions/reksai/build/jungle?opp=leesin&rank=master_plus",
    );
    expect(matchupLink("lolalytics", rekSai, leeSin, "utility")).toBe(
      "https://lolalytics.com/lol/reksai/vs/leesin/build/?lane=support&vslane=support&tier=master_plus",
    );
  });

  it("leaves out the leads Lolalytics doesn't have", () => {
    const lolalytics: MatchupStats = {
      ...stats,
      goldDiff15: null,
      xpDiff15: null,
      csDiff15: null,
      jungleCsDiff15: null,
      killDiff15: null,
    };
    expect(formatLead(lolalytics.killDiff15, 1)).toBe("–");
    expect(csLead(lolalytics)).toBeNull();
    expect(leadClass(null)).toBe("text-muted-foreground");
    expect(describeMatchup(lolalytics, rekSai, leeSin)).toBe(
      "Rek'Sai vs Lee Sin: 56.5% win rate (+3.6 vs expected), 644 Master+ games\n" +
        "No U.GG leads at 15 min",
    );
  });
});

describe("both sites", () => {
  it("lists each site's numbers in the tooltip once both have loaded", () => {
    const both: MatchupStats = {
      ...stats,
      lolalytics: { games: 715, winRate: 53.29, winRateDelta: 2.99 },
      agreement: 2,
      sources: 2,
    };
    expect(describeMatchup(both, rekSai, leeSin).split("\n").slice(1, 3)).toEqual([
      "Lolalytics: 53.3% (+3.0 vs expected), 715 games",
      "U.GG: 56.5% (+3.6 vs expected), 644 games",
    ]);
    expect(describeMatchup({ ...both, ugg: null }, rekSai, leeSin)).toContain(
      "U.GG: not enough games",
    );
  });
});

describe("lanes", () => {
  const enemy = (id: number, role: Role | null): EnemyMatchups => ({
    champion: { id, name: `Champion ${id}`, slug: "" },
    role,
    roleGuessed: true,
    games: 0,
    winRate: null,
    loading: false,
    counters: [],
    ally: null,
  });
  const board: MatchupBoard = {
    provider: "ugg",
    patch: "26.21",
    error: null,
    selfRole: "jungle",
    allies: [{ champion: leeSin, role: "jungle", isSelf: true }],
    enemies: [enemy(421, "jungle"), enemy(103, "middle"), enemy(222, null)],
  };

  const player = (puuid: string, position?: string, isSelf = false) =>
    ({ puuid, game_name: puuid, assigned_position: position, is_self: isSelf }) as Participant;

  it("lines up each lane's players and picks, then whatever has no lane yet", () => {
    const rows = draftLanes(
      [player("me", "jungle", true), player("mid", "middle"), player("chat-only")],
      [player("their-top", "top")],
      board,
    );
    expect(
      rows.map((row) => [
        row.role,
        row.player?.puuid ?? null,
        row.pick?.champion.id ?? null,
        row.enemy?.champion.id ?? null,
        row.enemyPlayer?.puuid ?? null,
      ]),
    ).toEqual([
      ["top", null, null, null, "their-top"],
      ["jungle", "me", 64, 421, null],
      ["middle", "mid", null, 103, null],
      ["bottom", null, null, null, null],
      ["utility", null, null, null, null],
      [null, "chat-only", null, null, null],
      [null, null, null, 222, null],
    ]);
    expect(rows.filter((row) => row.isSelf).map((row) => row.role)).toEqual(["jungle"]);
  });

  it("moves a dragged pick into its lane and swaps out the one there", () => {
    const moved = withRoleOverrides(board.enemies, new Map([[103, "jungle"]]));
    expect(moved.map((enemy) => [enemy.champion.id, enemy.role])).toEqual([
      [421, "middle"],
      [103, "jungle"],
      [222, null],
    ]);
    expect(moved[1].roleGuessed).toBe(false);
    expect(withRoleOverrides(board.enemies, new Map())).toBe(board.enemies);
  });

  it("works before the enemy team picks", () => {
    const rows = draftLanes([player("me", "jungle", true)], [], null);
    expect(rows).toHaveLength(5);
    expect(rows[1].player?.puuid).toBe("me");
    expect(rows.every((row) => row.enemy === null)).toBe(true);
  });

  it("shows the counters you clicked, else your lane's, else the first enemy's", () => {
    const rows = draftLanes([], [], board);
    expect(focusedEnemy(rows, null)?.champion.id).toBe(421);
    expect(focusedEnemy(rows, 103)?.champion.id).toBe(103);
    expect(focusedEnemy(rows, 999)?.champion.id).toBe(421);
    expect(focusedEnemy(draftLanes([], [], { ...board, selfRole: "top" }), null)?.champion.id).toBe(
      421,
    );
    expect(focusedEnemy(draftLanes([], [], { ...board, enemies: [] }), null)).toBeNull();
  });
});

describe("matchup numbers", () => {
  it("signs leads", () => {
    expect(formatLead(451.9)).toBe("+452");
    expect(formatLead(-0.14, 1)).toBe("-0.1");
    expect(formatLead(-0.04, 1)).toBe("0.0");
    expect(formatLead(0)).toBe("0");
  });

  it("counts jungle camps in the farm lead", () => {
    expect(csLead(stats)).toBeCloseTo(-1.32);
  });

  it("judges a win rate by its delta when there is one", () => {
    expect(winRateEdge(stats)).toBe(3.58);
    expect(winRateEdge({ ...stats, winRate: 48.9, winRateDelta: null })).toBeCloseTo(-1.1);
  });

  it("colors leads outside the dead zone", () => {
    expect(leadClass(3.58, 1)).toBe("text-emerald-300");
    expect(leadClass(-0.5, 1)).toBe("text-muted-foreground");
    expect(leadClass(-2)).toBe("text-red-300");
  });

  it("puts every number in the tooltip", () => {
    expect(describeMatchup(stats, rekSai, leeSin)).toBe(
      "Rek'Sai vs Lee Sin: 56.5% win rate (+3.6 vs expected), 644 Master+ games\n" +
        "At 15 min: +0.14 kills, -1.3 CS (-1.6 jungle), +82 gold, -1 XP",
    );
    expect(describeMatchup({ ...stats, games: 120, winRateDelta: null }, rekSai, leeSin)).toBe(
      "Rek'Sai vs Lee Sin: 56.5% win rate, 120 Master+ games\n" +
        "At 15 min: +0.14 kills, -1.3 CS (-1.6 jungle), +82 gold, -1 XP\n" +
        "Small sample, so take it with a grain of salt",
    );
  });
});
