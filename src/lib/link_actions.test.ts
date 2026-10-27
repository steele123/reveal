import { describe, expect, it } from "vitest";
import type { Participant } from "$lib/champ_select";
import type { MultiProvider } from "$lib/config";
import { createMultiSearchLink, createOpggProfileLink, webRegion } from "$lib/link_actions";
import { opggProfileUrl, type LiveGame, type LivePlayer } from "$lib/live_game";
import linkCases from "$lib/link_cases.json";

const PROVIDERS: MultiProvider[] = ["opgg", "fow", "ugg", "porofessor", "laddereye"];

function lobby(region: string): Participant[] {
  return linkCases.players.map(
    (player) =>
      ({ game_name: player.gameName, game_tag: player.tagLine, region }) as Participant,
  );
}

describe.each(linkCases.cases)("links for region $region", (expected) => {
  it("opens the OP.GG profile", () => {
    expect(createOpggProfileLink(lobby(expected.region)[0])).toBe(expected.profile);
  });

  it.each(PROVIDERS)("opens %s multi-search", (provider) => {
    expect(createMultiSearchLink(lobby(expected.region), provider)).toBe(expected[provider]);
  });
});

describe("webRegion", () => {
  it("defaults to NA when the region is missing", () => {
    expect(webRegion("")).toBe("NA");
  });
});

describe("live game profile links", () => {
  const player = { gameName: "Player One", tagLine: "NA1" } as LivePlayer;
  const game = (region: string) => ({ region }) as LiveGame;

  it("uses the backend's web region", () => {
    expect(opggProfileUrl(game("EUW"), player)).toBe(
      "https://op.gg/lol/summoners/euw/Player%20One-NA1",
    );
  });

  it("has no link for hidden players or an unknown region", () => {
    expect(opggProfileUrl(game("EUW"), { ...player, nameHidden: true })).toBeNull();
    expect(opggProfileUrl(game(""), player)).toBeNull();
  });
});
