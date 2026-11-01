import { invoke } from "@tauri-apps/api/tauri";
import {
  formatAssignedPosition,
  formatRank,
  offRoleReason,
  type Participant,
} from "$lib/champ_select";
import type { NotificationSettings } from "$lib/config";
import { participantKey, type SeenBefore } from "$lib/lobby_history";

/** Where alerts can go: Windows notifications, the ntfy phone app, a Discord channel. */
export type NotificationChannel = "desktop" | "phone" | "discord";

/** The alerts, by the keys the backend uses. */
export const NOTIFICATION_ALERTS: ReadonlyArray<{
  key: string;
  label: string;
  description: string;
}> = [
  {
    key: "autoQueue",
    label: "Queued again",
    description: "Reveal started the queue for you, after a game or a long wait",
  },
  {
    key: "matchFound",
    label: "Match found",
    description: "The ready check pops, and whether Reveal accepts it for you",
  },
  {
    key: "yourTurn",
    label: "Your turn",
    description: "Your ban or pick starts, with the seconds you have",
  },
  {
    key: "autoAction",
    label: "Auto ban and lock-in",
    description: "What Reveal banned or locked in for you, or why it couldn't",
  },
  {
    key: "dodge",
    label: "Dodge",
    description: "Champ Select ended without a game",
  },
  {
    key: "lobbyRevealed",
    label: "Team revealed",
    description: "Who's on your team, who you've seen before and who's likely off-role",
  },
];

const TOPIC_LETTERS = "abcdefghijkmnpqrstuvwxyz23456789";

/**
 * A new ntfy topic. Anyone who knows a topic can read what's sent to it, so it's long and
 * random, and leaves out letters that are easy to mix up when typed on a phone.
 */
export function generateNtfyTopic(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(20));
  return `reveal-${Array.from(bytes, (byte) => TOPIC_LETTERS[byte % TOPIC_LETTERS.length]).join("")}`;
}

/** Where to subscribe to the topic: the ntfy app, or ntfy.sh in a browser. */
export function ntfyTopicLink(topic: string): string {
  return `https://ntfy.sh/${encodeURIComponent(topic.trim())}`;
}

/** What the backend accepts, so the settings can say so before a test fails. */
export function isNtfyTopic(topic: string): boolean {
  return /^[A-Za-z0-9_-]{1,64}$/.test(topic.trim());
}

export function isDiscordWebhook(webhook: string): boolean {
  return /^https:\/\/(?:(?:ptb|canary)\.)?discord(?:app)?\.com\/api\/webhooks\/\S+$/.test(
    webhook.trim(),
  );
}

/**
 * The "team revealed" alert's summary: how many teammates you've met before (and dodged with),
 * and how many are likely off their usual role. Names go in `lobbyPlayers`, which only the phone
 * and Discord alerts list.
 */
export function lobbyNotice(participants: Participant[], seen: Map<string, SeenBefore>): string {
  const teammates = participants.filter((participant) => !participant.is_enemy && !participant.is_self);
  const met = teammates.filter((participant) => seen.has(participantKey(participant)));
  const dodged = met.filter((participant) => (seen.get(participantKey(participant))?.dodges ?? 0) > 0);
  const offRole = teammates.filter((participant) => offRoleReason(participant)).length;

  const plural = (count: number, word: string) => `${count} ${word}${count === 1 ? "" : "s"}`;
  const parts = [
    met.length === 0
      ? "Nobody you've seen before."
      : `${plural(met.length, "teammate")} you've seen before${dodged.length ? `, ${dodged.length} of them in a dodged lobby` : ""}.`,
    offRole ? `${offRole} likely off-role or autofilled.` : "",
  ];
  return parts.filter(Boolean).join(" ");
}

/** One player in an alert, as the backend lists them. */
export interface NoticePlayer {
  name: string;
  detail: string;
}

/**
 * Your team for the phone and Discord alerts, in pick order: Riot ID, then role, rank, how often
 * you've met them and whether they're likely off-role, e.g. "Jungle · Diamond II 45 LP · Seen 3×".
 */
export function lobbyPlayers(
  participants: Participant[],
  seen: Map<string, SeenBefore>,
): NoticePlayer[] {
  return participants
    .filter((participant) => !participant.is_enemy && participant.game_name)
    .map((participant) => {
      const rank = participant.rank && formatRank(participant.rank, participant.recent);
      const met = participant.is_self ? undefined : seen.get(participantKey(participant));
      const detail = [
        participant.assigned_position && formatAssignedPosition(participant.assigned_position),
        rank && [rank.tier, rank.details[0]].filter(Boolean).join(" "),
        met && `Seen ${met.lobbies}×${met.dodges ? ` (${met.dodges} dodged)` : ""}`,
        offRoleReason(participant),
      ];
      return {
        name: `${participant.game_name}#${participant.game_tag}${participant.is_self ? " (you)" : ""}`,
        detail: detail.filter(Boolean).join(" · "),
      };
    });
}

/** Sends a sample alert with the settings as they are on screen, saved or not. */
export async function sendTestNotification(
  channel: NotificationChannel,
  settings: NotificationSettings,
): Promise<void> {
  await invoke("send_test_notification", { channel, settings });
}

/** `link` opens the team's multi-search from the phone alert or the Discord message. */
export async function notifyLobbyRevealed(
  body: string,
  players: NoticePlayer[],
  link: string | null,
): Promise<void> {
  await invoke("notify_lobby_revealed", { body, players, link });
}
