import type { ChampSelect, Participant } from "$lib/champ_select";
import { logFrontendError } from "$lib/logging";

const STORAGE_KEY = "reveal:lobby-history";
export const MAX_HISTORY_ITEMS = 200;

export interface LobbyHistoryEntry {
  id: string;
  revealedAt: string;
  gameStartedAt?: string;
  /** Set when the lobby went back to the lobby or queue instead of a game. Lobbies saved before
   * Reveal 1.4.10 have neither this nor gameStartedAt, so their outcome is unknown. */
  dodgedAt?: string;
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
    dodgedAt: existing?.dodgedAt,
    participants: mergeParticipants(
      existing?.participants ?? [],
      champSelect.participants,
    ),
  };

  return persist([entry, ...entries.filter((item) => item.id !== lobbyId)]);
}

/** What a League Client state change means for the lobby being recorded. */
export function lobbyEvent(
  previousState: string,
  nextState: string,
): "new-lobby" | "game-started" | "left" | null {
  if (nextState === "ChampSelect") {
    return previousState === "ChampSelect" ? null : "new-lobby";
  }
  // A game goes ChampSelect -> GameStart -> InProgress; a dodge goes back to Lobby or Matchmaking.
  if (nextState === "GameStart" || nextState === "InProgress") return "game-started";
  return "left";
}

export function markLobbyGameStarted(lobbyId: string): LobbyHistoryEntry[] {
  return markOutcome(lobbyId, "gameStartedAt");
}

export function markLobbyDodged(lobbyId: string): LobbyHistoryEntry[] {
  return markOutcome(lobbyId, "dodgedAt");
}

function markOutcome(lobbyId: string, outcome: "gameStartedAt" | "dodgedAt"): LobbyHistoryEntry[] {
  const entries = loadLobbyHistory();
  const at = new Date().toISOString();
  let changed = false;

  const nextEntries = entries.map((entry) => {
    if (entry.id !== lobbyId || entry.gameStartedAt || entry.dodgedAt) return entry;
    changed = true;
    return { ...entry, [outcome]: at };
  });

  return changed ? persist(nextEntries) : entries;
}

export interface SeenBefore {
  /** Earlier lobbies this player was in. */
  lobbies: number;
  /** Of those, how many went on to a game. */
  games: number;
  /** And how many were dodged. Older lobbies may be neither, since their outcome is unknown. */
  dodges: number;
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
      const seen = index.get(key) ?? {
        lobbies: 0,
        games: 0,
        dodges: 0,
        lastSeen: entry.revealedAt,
      };
      seen.lobbies += 1;
      if (entry.gameStartedAt) seen.games += 1;
      else if (entry.dodgedAt) seen.dodges += 1;
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

/** e.g. "Seen 3× before (2 games together, 1 dodged) · last 2 days ago". */
export function describeSeenBefore(seen: SeenBefore, now = Date.now()): string {
  const outcomes = [
    seen.games > 0 && `${seen.games} game${seen.games === 1 ? "" : "s"} together`,
    seen.dodges > 0 && `${seen.dodges} dodged`,
  ].filter(Boolean);
  const detail = outcomes.length ? ` (${outcomes.join(", ")})` : "";
  return `Seen ${seen.lobbies}× before${detail} · last ${formatTimeAgo(seen.lastSeen, now)}`;
}

export function clearLobbyHistory(): void {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch (error) {
    logFrontendError("Failed to clear lobby history", error);
  }
}
