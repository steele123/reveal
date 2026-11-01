import { describe, expect, it } from "vitest";
import type { Participant } from "$lib/champ_select";
import { participantKey, type SeenBefore } from "$lib/lobby_history";
import {
  generateNtfyTopic,
  isDiscordWebhook,
  isNtfyTopic,
  lobbyNotice,
  ntfyTopicLink,
} from "$lib/notifications";

function player(name: string, extra: Partial<Participant> = {}): Participant {
  return {
    cid: "",
    game_name: name,
    game_tag: "NA1",
    muted: false,
    pid: "",
    puuid: `puuid-${name}`,
    region: "na",
    ...extra,
  };
}

function seen(dodges: number): SeenBefore {
  return { lobbies: 1 + dodges, games: 1, dodges, lastSeen: "2026-10-30T12:00:00Z" };
}

describe("team revealed alert", () => {
  const team = [
    player("Me", { is_self: true }),
    player("Paper Lantern"),
    player("Kindred Spirit", { autofilled: true }),
    player("Blue Sentinel"),
    player("Last Whisper"),
    player("Enemy", { is_enemy: true }),
  ];

  it("counts teammates seen before, dodges and off-roles, without names", () => {
    const index = new Map([
      [participantKey(team[1]), seen(0)],
      [participantKey(team[2]), seen(1)],
      // You and the enemy team don't count.
      [participantKey(team[0]), seen(0)],
      [participantKey(team[5]), seen(0)],
    ]);
    const body = lobbyNotice(team, index);
    expect(body).toBe(
      "2 teammates you've seen before, 1 of them in a dodged lobby. 1 likely off-role or autofilled.",
    );
    expect(body).not.toContain("Kindred");
  });

  it("says so when nobody is familiar", () => {
    expect(lobbyNotice(team.slice(0, 2), new Map())).toBe("Nobody you've seen before.");
  });
});

describe("notification setup", () => {
  it("makes long random ntfy topics the backend accepts", () => {
    const topic = generateNtfyTopic();
    expect(topic).toMatch(/^reveal-[a-z2-9]{20}$/);
    expect(isNtfyTopic(topic)).toBe(true);
    expect(generateNtfyTopic()).not.toBe(topic);
    expect(ntfyTopicLink(` ${topic} `)).toBe(`https://ntfy.sh/${topic}`);
  });

  it("checks topics and webhooks like the backend does", () => {
    expect(isNtfyTopic("has space")).toBe(false);
    expect(isNtfyTopic("a".repeat(65))).toBe(false);
    expect(isDiscordWebhook("https://discord.com/api/webhooks/1/token")).toBe(true);
    expect(isDiscordWebhook("https://canary.discord.com/api/webhooks/1/token")).toBe(true);
    expect(isDiscordWebhook("https://discord.com.evil.example/api/webhooks/1/t")).toBe(false);
    expect(isDiscordWebhook("https://discord.com/channels/1/2")).toBe(false);
  });
});
