import type { ChampSelect, RankSummary } from "$lib/champ_select";
import type { LobbyHistoryEntry } from "$lib/lobby_history";

interface PreviewState {
  connected: boolean;
  state: string;
  champSelect: ChampSelect | null;
  activePage: "reveal" | "history";
  history: LobbyHistoryEntry[];
}

const PREVIEW_RANKS: RankSummary[] = [
  { tier: "DIAMOND", division: "II", leaguePoints: 45, wins: 70, losses: 51, provisional: false },
  { tier: "MASTER", leaguePoints: 155, wins: 376, losses: 328, provisional: false },
  { tier: "GOLD", division: "IV", leaguePoints: 0, wins: 22, losses: 25, provisional: false },
  { tier: "UNRANKED", leaguePoints: 0, wins: 2, losses: 1, provisional: true },
  { tier: "UNRANKED", leaguePoints: 0, wins: 0, losses: 0, provisional: false },
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
      history: [],
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
