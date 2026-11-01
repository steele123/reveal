/** Every champion's id and English name, for picking auto-ban and auto-pick champions. */
export interface ChampionName {
  id: number;
  name: string;
}

const SUMMARY_URL =
  "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-summary.json";

let loading: Promise<ChampionName[]> | null = null;

/** Community Dragon's champion list, sorted by name and fetched once. */
export function loadChampionNames(): Promise<ChampionName[]> {
  loading ??= fetch(SUMMARY_URL)
    .then((response) => {
      if (!response.ok) throw new Error(`Champion list request failed: ${response.status}`);
      return response.json() as Promise<ChampionName[]>;
    })
    .then((champions) =>
      champions
        // Game modes add copies with ids from 60000 up ("Annie" again as 60001).
        .filter((champion) => champion.id > 0 && champion.id < 10_000)
        .map(({ id, name }) => ({ id, name }))
        .sort((a, b) => a.name.localeCompare(b.name)),
    );
  // Let a failed download be tried again.
  loading.catch(() => (loading = null));
  return loading;
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
