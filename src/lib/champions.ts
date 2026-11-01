import { invoke } from "@tauri-apps/api/tauri";
import { isTauriRuntime } from "$lib/runtime";

/** Every champion's id and name, for picking auto-ban and auto-pick champions. */
export interface ChampionName {
  id: number;
  name: string;
}

const SUMMARY_URL =
  "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-summary.json";

/** A download that hasn't answered by now has failed, so the error shows instead of empty names. */
const DOWNLOAD_TIMEOUT_MS = 10_000;

let loading: Promise<ChampionName[]> | null = null;

/**
 * Every champion, sorted by name and loaded once: from the League Client when it's running (no
 * download, and in the client's language), or else from Community Dragon.
 */
export function loadChampionNames(): Promise<ChampionName[]> {
  loading ??= fromLeague().then((champions) => champions ?? fromCommunityDragon());
  // Let a failed load be tried again.
  loading.catch(() => (loading = null));
  return loading;
}

async function fromLeague(): Promise<ChampionName[] | null> {
  if (!isTauriRuntime()) return null;
  try {
    const champions = await invoke<ChampionName[]>("get_champion_list");
    return champions.length ? champions : null;
  } catch {
    // League isn't running; Community Dragon has the same list.
    return null;
  }
}

async function fromCommunityDragon(): Promise<ChampionName[]> {
  const response = await fetch(SUMMARY_URL, { signal: AbortSignal.timeout(DOWNLOAD_TIMEOUT_MS) });
  if (!response.ok) throw new Error(`Champion list request failed: ${response.status}`);
  const champions = (await response.json()) as ChampionName[];
  return (
    champions
      // Game modes add copies with ids from 60000 up ("Annie" again as 60001).
      .filter((champion) => champion.id > 0 && champion.id < 10_000)
      .map(({ id, name }) => ({ id, name }))
      .sort((a, b) => a.name.localeCompare(b.name))
  );
}

function normalize(name: string): string {
  return name.toLowerCase().replace(/[^a-z0-9]/g, "");
}

const MAX_MATCHES = 12;

/** Champions whose names start with what's typed, then ones that contain it, ignoring case,
 * spaces and punctuation. */
export function searchChampions(
  champions: ChampionName[],
  typed: string,
  limit = MAX_MATCHES,
): ChampionName[] {
  const wanted = normalize(typed);
  if (!wanted) return [];
  const starts: ChampionName[] = [];
  const contains: ChampionName[] = [];
  for (const champion of champions) {
    const name = normalize(champion.name);
    if (name.startsWith(wanted)) starts.push(champion);
    else if (name.includes(wanted)) contains.push(champion);
  }
  return [...starts, ...contains].slice(0, limit);
}

/** The champion typed in, ignoring case, spaces and punctuation ("kaisa" finds Kai'Sa). */
export function findChampion(
  champions: ChampionName[],
  typed: string,
): ChampionName | undefined {
  const wanted = normalize(typed);
  if (!wanted) return undefined;
  return champions.find((champion) => normalize(champion.name) === wanted);
}
