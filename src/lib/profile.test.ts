import { describe, expect, it } from "vitest";
import { formatRecord, recordWinRate } from "$lib/profile";

const rank = { tier: "EMERALD", division: "I", leaguePoints: 62, provisional: false };

describe("your ranked record", () => {
  it("shows wins, losses and win rate", () => {
    expect(formatRecord({ ...rank, wins: 118, losses: 97 })).toBe("118W 97L · 55%");
    expect(recordWinRate({ ...rank, wins: 118, losses: 97 })).toBe("55%");
  });

  it("stays empty before any ranked games", () => {
    expect(formatRecord({ ...rank, wins: 0, losses: 0 })).toBe("");
    expect(recordWinRate({ ...rank, wins: 0, losses: 0 })).toBe("");
  });
});
