import type { ChampSelect, Participant } from "$lib/champ_select";
import { logFrontendError } from "$lib/logging";

const STORAGE_KEY = "reveal:lobby-history";
export const MAX_HISTORY_ITEMS = 200;

export interface LobbyHistoryEntry {
  id: string;
  revealedAt: string;
  gameStartedAt?: string;
  participants: Participant[];
}

export function participantKey(participant: Participant): string {
  return (
    participant.puuid ||
    `${participant.game_name}#${participant.game_tag}:${participant.region}`
  ).toLowerCase();
}

function mergeParticipants(
  current: Participant[],
  incoming: Participant[],
): Participant[] {
  const participants = new Map<string, Participant>();

  for (const participant of [...current, ...incoming]) {
    participants.set(participantKey(participant), participant);
  }

  return Array.from(participants.values()).slice(0, 5);
}

function isHistoryEntry(value: unknown): value is LobbyHistoryEntry {
  if (!value || typeof value !== "object") return false;

  const entry = value as Partial<LobbyHistoryEntry>;
  return (
    typeof entry.id === "string" &&
    typeof entry.revealedAt === "string" &&
    Array.isArray(entry.participants)
  );
}

function persist(entries: LobbyHistoryEntry[]): LobbyHistoryEntry[] {
  const nextEntries = entries.slice(0, MAX_HISTORY_ITEMS);

  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(nextEntries));
  } catch (error) {
    logFrontendError("Failed to save lobby history", error);
  }

  return nextEntries;
}

export function createLobbyHistoryId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }

  return `${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

export function loadLobbyHistory(): LobbyHistoryEntry[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (!stored) return [];

    const parsed: unknown = JSON.parse(stored);
    if (!Array.isArray(parsed)) return [];

    return parsed.filter(isHistoryEntry).slice(0, MAX_HISTORY_ITEMS);
  } catch (error) {
    logFrontendError("Failed to load lobby history", error);
    return [];
  }
}

export function recordLobbyReveal(
  lobbyId: string,
  champSelect: ChampSelect,
): LobbyHistoryEntry[] {
  const entries = loadLobbyHistory();
  const existing = entries.find((entry) => entry.id === lobbyId);
  const entry: LobbyHistoryEntry = {
    id: lobbyId,
    revealedAt: existing?.revealedAt ?? new Date().toISOString(),
    gameStartedAt: existing?.gameStartedAt,
    participants: mergeParticipants(
      existing?.participants ?? [],
      champSelect.participants,
    ),
  };

  return persist([entry, ...entries.filter((item) => item.id !== lobbyId)]);
}

export function markLobbyGameStarted(lobbyId: string): LobbyHistoryEntry[] {
  const entries = loadLobbyHistory();
  const startedAt = new Date().toISOString();
  let changed = false;

  const nextEntries = entries.map((entry) => {
    if (entry.id !== lobbyId || entry.gameStartedAt) return entry;
    changed = true;
    return { ...entry, gameStartedAt: startedAt };
  });

  return changed ? persist(nextEntries) : entries;
}

export interface SeenBefore {
  /** Earlier lobbies this player was in. */
  lobbies: number;
  /** Of those, how many went on to a game (not dodged). */
  games: number;
  lastSeen: string;
}

/** How often each player appeared in earlier lobbies, keyed by participantKey. */
export function seenBeforeIndex(
  history: LobbyHistoryEntry[],
  currentLobbyId: string | null,
): Map<string, SeenBefore> {
  const index = new Map<string, SeenBefore>();
  for (const entry of history) {
    if (entry.id === currentLobbyId) continue;
    for (const participant of entry.participants) {
      const key = participantKey(participant);
      const seen = index.get(key) ?? { lobbies: 0, games: 0, lastSeen: entry.revealedAt };
      seen.lobbies += 1;
      if (entry.gameStartedAt) seen.games += 1;
      if (entry.revealedAt > seen.lastSeen) seen.lastSeen = entry.revealedAt;
      index.set(key, seen);
    }
  }
  return index;
}

const relativeTime = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });

/** e.g. "2 days ago", "yesterday", "3 hours ago". */
export function formatTimeAgo(iso: string, now = Date.now()): string {
  const seconds = (new Date(iso).getTime() - now) / 1000;
  const steps: [Intl.RelativeTimeFormatUnit, number][] = [
    ["year", 31_536_000],
    ["month", 2_592_000],
    ["week", 604_800],
    ["day", 86_400],
    ["hour", 3_600],
    ["minute", 60],
  ];
  for (const [unit, size] of steps) {
    if (Math.abs(seconds) >= size) return relativeTime.format(Math.round(seconds / size), unit);
  }
  return "just now";
}

/** e.g. "Seen 3× before (2 games together) · last 2 days ago". */
export function describeSeenBefore(seen: SeenBefore): string {
  const games =
    seen.games === seen.lobbies
      ? `${seen.games} game${seen.games === 1 ? "" : "s"} together`
      : `${seen.games} game${seen.games === 1 ? "" : "s"} together, ${seen.lobbies - seen.games} dodged`;
  return `Seen ${seen.lobbies}× before (${games}) · last ${formatTimeAgo(seen.lastSeen)}`;
}

export function clearLobbyHistory(): void {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch (error) {
    logFrontendError("Failed to clear lobby history", error);
  }
}
