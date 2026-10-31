import { describe, expect, it } from "vitest";
import {
  describeSeenBefore,
  lobbyEvent,
  seenBeforeIndex,
  type LobbyHistoryEntry,
} from "$lib/lobby_history";

describe("lobbyEvent", () => {
  it("counts a lobby as played when the game starts", () => {
    // League's real sequence: GameStart always comes between Champ Select and the game.
    expect(lobbyEvent("ChampSelect", "GameStart")).toBe("game-started");
    expect(lobbyEvent("GameStart", "InProgress")).toBe("game-started");
  });

  it("counts a lobby as dodged when it goes back to the lobby or queue", () => {
    expect(lobbyEvent("ChampSelect", "Lobby")).toBe("left");
    expect(lobbyEvent("ChampSelect", "Matchmaking")).toBe("left");
    expect(lobbyEvent("ChampSelect", "None")).toBe("left");
  });

  it("starts a lobby once per Champ Select", () => {
    expect(lobbyEvent("ReadyCheck", "ChampSelect")).toBe("new-lobby");
    expect(lobbyEvent("ChampSelect", "ChampSelect")).toBeNull();
  });
});

describe("seen before", () => {
  const player = { puuid: "p1", game_name: "Player", game_tag: "NA1" } as LobbyHistoryEntry["participants"][number];
  const lobby = (id: string, outcome: Partial<LobbyHistoryEntry>): LobbyHistoryEntry => ({
    id,
    revealedAt: "2026-10-30T12:00:00.000Z",
    participants: [player],
    ...outcome,
  });
  const now = Date.parse("2026-10-31T12:00:00.000Z");

  it("counts games and dodges, and doesn't call older lobbies dodges", () => {
    const seen = seenBeforeIndex(
      [
        lobby("played", { gameStartedAt: "2026-10-30T12:05:00.000Z" }),
        lobby("dodged", { dodgedAt: "2026-10-30T12:04:00.000Z" }),
        lobby("before-1.4.10", {}),
      ],
      null,
    ).get("p1")!;

    expect(seen).toMatchObject({ lobbies: 3, games: 1, dodges: 1 });
    expect(describeSeenBefore(seen, now)).toBe(
      "Seen 3× before (1 game together, 1 dodged) · last yesterday",
    );
  });

  it("leaves out the breakdown when every outcome is unknown", () => {
    const seen = seenBeforeIndex([lobby("a", {}), lobby("b", {})], null).get("p1")!;
    expect(describeSeenBefore(seen, now)).toBe("Seen 2× before · last yesterday");
  });
});
