import type {
  ChampSelect,
  ChampionMain,
  RankSummary,
  RecentForm,
} from "$lib/champ_select";
import type { LobbyHistoryEntry } from "$lib/lobby_history";
import type { LiveGame, LivePlayer } from "$lib/live_game";

interface PreviewState {
  connected: boolean;
  state: string;
  champSelect: ChampSelect | null;
  activePage: "reveal" | "history";
  history: LobbyHistoryEntry[];
  liveGame?: LiveGame;
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

const ICONS: Record<number, string> = {
  3020: "3020_class_t2_sorcerersshoes.png",
  6655: "6655_mage_t4_ludenstempest.png",
  3340: "3340_class_t1_wardingtotem.png",
  1056: "1056_mage_t1_doransring.png",
  3006: "3006_class_t2_berserkersgreaves.png",
  6672: "6672_marksman_t4_behemothslayer.png",
  1055: "1055_marksman_t1_doransblade.png",
  3047: "3047_class_t2_ninjatabi.png",
  3068: "3068_tank_t4_sunfireaegis.png",
};

function livePlayer(
  base: Partial<LivePlayer> & Pick<LivePlayer, "gameName" | "tagLine" | "team" | "championName" | "position">,
): LivePlayer {
  return {
    isSelf: false, isBot: false, championId: null, level: 9, kills: 2, deaths: 2, assists: 3,
    creepScore: 90, isDead: false, respawnTimer: 0, items: [], puuid: null, rank: null, recent: null,
    ...base,
  };
}

const PREVIEW_LIVE_GAME: LiveGame = {
  gameTime: 812,
  gameMode: "CLASSIC",
  players: [
    livePlayer({ gameName: "Paper Lantern", tagLine: "NA1", team: "ORDER", championName: "Vayne", championId: 67, position: "TOP", level: 10, kills: 3, deaths: 1, assists: 2, creepScore: 121, rank: PREVIEW_RANKS[0], recent: PREVIEW_RECENT[0], items: [{ id: 1055, name: "Doran's Blade", slot: 0, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[1055]}` }, { id: 3006, name: "Berserker's Greaves", slot: 1, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[3006]}` }, { id: 3340, name: "Stealth Ward", slot: 6, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[3340]}` }] }),
    livePlayer({ gameName: "Kindred Spirit", tagLine: "WOLF", team: "ORDER", championName: "Qiyana", championId: 246, position: "JUNGLE", level: 11, kills: 6, deaths: 2, assists: 4, creepScore: 98, rank: PREVIEW_RANKS[1], recent: PREVIEW_RECENT[1] }),
    livePlayer({ gameName: "spooky", tagLine: "2729", team: "ORDER", championName: "Taliyah", championId: 163, position: "MIDDLE", isSelf: true, level: 11, kills: 4, deaths: 1, assists: 7, creepScore: 142, rank: PREVIEW_RANKS[2], recent: PREVIEW_RECENT[2], items: [{ id: 6655, name: "Luden's Companion", slot: 0, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[6655]}` }, { id: 3020, name: "Sorcerer's Shoes", slot: 1, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[3020]}` }, { id: 1056, name: "Doran's Ring", slot: 2, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[1056]}` }, { id: 3340, name: "Stealth Ward", slot: 6, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[3340]}` }] }),
    livePlayer({ gameName: "wanan yuen", tagLine: "chaye", team: "ORDER", championName: "Twisted Fate", championId: 4, position: "BOTTOM", level: 9, kills: 1, deaths: 3, assists: 5, creepScore: 110 }),
    livePlayer({ gameName: "yuen", tagLine: "vvv", team: "ORDER", championName: "Lux", championId: 99, position: "UTILITY", level: 8, kills: 0, deaths: 2, assists: 11, creepScore: 18, rank: PREVIEW_RANKS[3], recent: PREVIEW_RECENT[3] }),
    livePlayer({ gameName: "Shopkeeper", tagLine: "123", team: "CHAOS", championName: "Wukong", championId: 62, position: "TOP", level: 10, kills: 2, deaths: 4, assists: 1, creepScore: 104, isDead: true, respawnTimer: 13, rank: PREVIEW_RANKS[2], recent: PREVIEW_RECENT[2], items: [{ id: 3047, name: "Plated Steelcaps", slot: 1, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[3047]}` }, { id: 3068, name: "Sunfire Aegis", slot: 0, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[3068]}` }] }),
    livePlayer({ gameName: "Odysseus", tagLine: "KR131", team: "CHAOS", championName: "Lee Sin", championId: 64, position: "JUNGLE", puuid: "enemy-2", level: 11, kills: 5, deaths: 3, assists: 3, creepScore: 95, rank: PREVIEW_RANKS[1], recent: PREVIEW_RECENT[0] }),
    livePlayer({ gameName: "SeTab", tagLine: "123", team: "CHAOS", championName: "Ahri", championId: 103, position: "MIDDLE", level: 10, kills: 2, deaths: 4, assists: 2, creepScore: 130, rank: PREVIEW_RANKS[0], recent: PREVIEW_RECENT[1] }),
    livePlayer({ gameName: "Clozer", tagLine: "0727", team: "CHAOS", championName: "Jinx", championId: 222, position: "BOTTOM", level: 9, kills: 1, deaths: 2, assists: 1, creepScore: 118, items: [{ id: 6672, name: "Kraken Slayer", slot: 0, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[6672]}` }, { id: 3006, name: "Berserker's Greaves", slot: 1, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[3006]}` }] }),
    livePlayer({ gameName: "Simsons", tagLine: "spfd", team: "CHAOS", championName: "Thresh", championId: 412, position: "UTILITY", isBot: false, level: 8, kills: 0, deaths: 2, assists: 5, creepScore: 12 }),
  ],
};

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
      history: [
        {
          id: "preview-seen-enemy",
          revealedAt: new Date(Date.now() - 3 * 86_400_000).toISOString(),
          gameStartedAt: new Date(Date.now() - 3 * 86_400_000).toISOString(),
          participants: [{ ...PREVIEW_PARTICIPANTS[0], puuid: "enemy-2", game_name: "Odysseus", game_tag: "KR131" }],
        },
      ],
      liveGame: PREVIEW_LIVE_GAME,
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
