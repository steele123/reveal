import type {
  ChampSelect,
  ChampionMain,
  RankSummary,
  RecentForm,
} from "$lib/champ_select";
import type { LobbyHistoryEntry } from "$lib/lobby_history";

interface PreviewState {
  connected: boolean;
  state: string;
  champSelect: ChampSelect | null;
  activePage: "reveal" | "history";
  history: LobbyHistoryEntry[];
}

const PREVIEW_RANKS: RankSummary[] = [
  { tier: "DIAMOND", division: "II", leaguePoints: 45, wins: 70, losses: 0, provisional: false },
  { tier: "MASTER", leaguePoints: 155, wins: 376, losses: 0, provisional: false },
  { tier: "GOLD", division: "I", leaguePoints: 24, wins: 12, losses: 0, provisional: false },
  { tier: "UNRANKED", leaguePoints: 0, wins: 2, losses: 0, provisional: true },
  { tier: "UNRANKED", leaguePoints: 0, wins: 0, losses: 0, provisional: false },
];

const PREVIEW_RECENT: RecentForm[] = [
  { wins: 11, losses: 8 },
  { wins: 10, losses: 10 },
  { wins: 3, losses: 3 },
  { wins: 2, losses: 1 },
  { wins: 0, losses: 0 },
];

const PREVIEW_MAINS: ChampionMain[][] = [
  [
    { championId: 157, name: "Yasuo", level: 40, points: 406821 },
    { championId: 92, name: "Riven", level: 39, points: 394816 },
    { championId: 238, name: "Zed", level: 38, points: 384110 },
  ],
  [
    { championId: 64, name: "Lee Sin", level: 25, points: 210500 },
    { championId: 121, name: "Kha'Zix", level: 18, points: 150200 },
    { championId: 254, name: "Vi", level: 12, points: 90100 },
  ],
  [
    { championId: 103, name: "Ahri", level: 30, points: 250000 },
    { championId: 950, name: "Naafiri", level: 6, points: 40000 },
  ],
  [
    { championId: 222, name: "Jinx", level: 9, points: 60000 },
    { championId: 51, name: "Caitlyn", level: 7, points: 45000 },
    { championId: 145, name: "Kai'Sa", level: 5, points: 30000 },
  ],
  [],
];

const PREVIEW_PARTICIPANTS = [
  ["Paper Lantern", "NA1"],
  ["Kindred Spirit", "WOLF"],
  ["Blue Sentinel", "MID"],
  ["Last Whisper", "ADC"],
  ["River Walker", "SUP"],
].map(([game_name, game_tag], index) => ({
  cid: `champ-select-${index}`,
  game_name,
  game_tag,
  muted: false,
  name: game_name,
  pid: `preview-${index}`,
  puuid: `preview-${index}`,
  region: "NA",
  assigned_position: ["top", "jungle", "middle", "bottom", "utility"][index],
  cell_id: index,
  pick_turn: index,
  rank: PREVIEW_RANKS[index],
  mains: PREVIEW_MAINS[index],
  recent: PREVIEW_RECENT[index],
  position_guessed: index === 1 || index === 3,
  is_self: index === 2,
  autofilled: index === 3,
  role_fit: [
    { games: 20, inRole: 14, mainPosition: "top", mainGames: 14 },
    { games: 18, inRole: 2, mainPosition: "middle", mainGames: 12 },
    { games: 19, inRole: 16, mainPosition: "middle", mainGames: 16 },
    { games: 17, inRole: 1, mainPosition: "jungle", mainGames: 13 },
    undefined,
  ][index],
  profile_icon_id: [7085, 4553, 29, 5367, undefined][index],
}));

export function getPreviewState(): PreviewState {
  const preview = new URLSearchParams(window.location.search).get("preview");

  if (preview === "history") {
    return {
      connected: true,
      state: "Lobby",
      champSelect: null,
      activePage: "history",
      history: [
        {
          id: "preview-history-current",
          revealedAt: new Date().toISOString(),
          gameStartedAt: new Date().toISOString(),
          participants: PREVIEW_PARTICIPANTS,
        },
        {
          id: "preview-history-earlier",
          revealedAt: new Date(Date.now() - 3_600_000).toISOString(),
          participants: PREVIEW_PARTICIPANTS.slice(0, 4).map(
            (participant, index) => ({
              ...participant,
              game_name: `${participant.game_name} ${index + 1}`,
              puuid: `earlier-${index}`,
            }),
          ),
        },
      ],
    };
  }

  if (preview === "champ-select") {
    return {
      connected: true,
      state: "ChampSelect",
      champSelect: { participants: PREVIEW_PARTICIPANTS },
      activePage: "reveal",
      history: [
        {
          id: "preview-seen-1",
          revealedAt: new Date(Date.now() - 2 * 86_400_000).toISOString(),
          gameStartedAt: new Date(Date.now() - 2 * 86_400_000 + 120_000).toISOString(),
          participants: [PREVIEW_PARTICIPANTS[1], PREVIEW_PARTICIPANTS[3]],
        },
        {
          id: "preview-seen-2",
          revealedAt: new Date(Date.now() - 6 * 86_400_000).toISOString(),
          participants: [PREVIEW_PARTICIPANTS[1]],
        },
        {
          id: "preview-seen-3",
          revealedAt: new Date(Date.now() - 9 * 86_400_000).toISOString(),
          gameStartedAt: new Date(Date.now() - 9 * 86_400_000 + 120_000).toISOString(),
          participants: [PREVIEW_PARTICIPANTS[1], PREVIEW_PARTICIPANTS[2]],
        },
      ],
    };
  }

  if (preview === "disconnected") {
    return {
      connected: false,
      state: "Unknown",
      champSelect: null,
      activePage: "reveal",
      history: [],
    };
  }

  if (preview === "in-game") {
    return {
      connected: true,
      state: "InProgress",
      champSelect: null,
      activePage: "reveal",
      history: [],
    };
  }

  return {
    connected: true,
    state: "Lobby",
    champSelect: null,
    activePage: "reveal",
    history: [],
  };
}
