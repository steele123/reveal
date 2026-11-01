import { describe, expect, it } from "vitest";
import { findChampion, searchChampions } from "$lib/champions";

const champions = [
  { id: 145, name: "Kai'Sa" },
  { id: 59, name: "Jarvan IV" },
  { id: 20, name: "Nunu & Willump" },
  { id: 38, name: "Kassadin" },
];

describe("findChampion", () => {
  it("ignores case, spaces and punctuation", () => {
    expect(findChampion(champions, "kaisa")?.id).toBe(145);
    expect(findChampion(champions, "JARVAN iv")?.id).toBe(59);
    expect(findChampion(champions, "Nunu & Willump")?.id).toBe(20);
  });

  it("finds nothing for partial or empty names", () => {
    expect(findChampion(champions, "jarvan")).toBeUndefined();
    expect(findChampion(champions, "  ")).toBeUndefined();
  });
});

describe("searchChampions", () => {
  it("lists names starting with what's typed before names containing it", () => {
    expect(searchChampions(champions, "ka").map((champion) => champion.name)).toEqual([
      "Kai'Sa",
      "Kassadin",
    ]);
    expect(searchChampions(champions, "sa").map((champion) => champion.name)).toEqual([
      "Kai'Sa",
      "Kassadin",
    ]);
    expect(searchChampions(champions, "will").map((champion) => champion.id)).toEqual([20]);
    expect(searchChampions(champions, " ")).toEqual([]);
  });
});
