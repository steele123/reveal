import { describe, expect, it } from "vitest";
import type { Participant } from "$lib/champ_select";
import type { MultiProvider } from "$lib/config";
import { createMultiSearchLink, createProfileLink, webRegion } from "$lib/link_actions";
import { profileUrl, type LiveGame, type LivePlayer } from "$lib/live_game";
import linkCases from "$lib/link_cases.json";

const PROVIDERS: MultiProvider[] = ["opgg", "fow", "ugg", "porofessor", "laddereye"];

function lobby(region: string): Participant[] {
  return linkCases.players.map(
    (player) =>
      ({ game_name: player.gameName, game_tag: player.tagLine, region }) as Participant,
  );
}

describe.each(linkCases.cases)("links for region $region", (expected) => {
  it.each(PROVIDERS)("opens the %s profile", (provider) => {
    expect(createProfileLink(lobby(expected.region)[0], provider)).toBe(expected.profile[provider]);
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

  it("uses the backend's web region and the chosen site", () => {
    expect(profileUrl(game("EUW"), player, "opgg")).toBe(
      "https://op.gg/lol/summoners/euw/Player%20One-NA1",
    );
    expect(profileUrl(game("EUW"), player, "ugg")).toBe(
      "https://u.gg/lol/profile/euw1/Player%20One-NA1/overview",
    );
  });

  it("opens Porofessor's live page in game", () => {
    expect(profileUrl(game("EUW"), player, "porofessor")).toBe(
      "https://porofessor.gg/live/euw/Player%20One-NA1",
    );
  });

  it("has no link for hidden players or an unknown region", () => {
    expect(profileUrl(game("EUW"), { ...player, nameHidden: true }, "opgg")).toBeNull();
    expect(profileUrl(game(""), player, "opgg")).toBeNull();
  });
});
