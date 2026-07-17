import type { ChampSelect, Participant } from "$lib/champ_select";

const STORAGE_KEY = "reveal:lobby-history";
export const MAX_HISTORY_ITEMS = 50;

export interface LobbyHistoryEntry {
  id: string;
  revealedAt: string;
  gameStartedAt?: string;
  participants: Participant[];
}

function participantKey(participant: Participant): string {
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
    console.error("Failed to save lobby history", error);
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
    console.error("Failed to load lobby history", error);
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

export function clearLobbyHistory(): void {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch (error) {
    console.error("Failed to clear lobby history", error);
  }
}
