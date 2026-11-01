import type {
  ChampSelect,
  ChampionMain,
  RankSummary,
  RecentForm,
} from "$lib/champ_select";
import type { LobbyHistoryEntry } from "$lib/lobby_history";
import type { LiveGame, LivePlayer } from "$lib/live_game";
import type {
  ChampionRef,
  CountersProvider,
  EnemyMatchups,
  MatchupBoard,
  MatchupStats,
  Role,
} from "$lib/matchups";

interface PreviewState {
  connected: boolean;
  state: string;
  champSelect: ChampSelect | null;
  activePage: "reveal" | "history" | "auto";
  history: LobbyHistoryEntry[];
  liveGame?: LiveGame;
  matchups?: MatchupBoard;
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
    isSelf: false, isBot: false, nameHidden: false, championId: null, level: 9, kills: 2, deaths: 2, assists: 3,
    creepScore: 90, isDead: false, respawnTimer: 0, items: [], puuid: null, rank: null, recent: null,
    ...base,
  };
}

const PREVIEW_LIVE_GAME: LiveGame = {
  gameTime: 812,
  gameMode: "CLASSIC",
  region: "NA",
  queueName: "Ranked Solo/Duo",
  players: [
    livePlayer({ gameName: "Paper Lantern", tagLine: "NA1", team: "ORDER", championName: "Vayne", championId: 67, position: "TOP", level: 10, kills: 3, deaths: 1, assists: 2, creepScore: 121, rank: PREVIEW_RANKS[0], recent: PREVIEW_RECENT[0], items: [{ id: 1055, name: "Doran's Blade", slot: 0, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[1055]}` }, { id: 3006, name: "Berserker's Greaves", slot: 1, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[3006]}` }, { id: 3340, name: "Stealth Ward", slot: 6, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[3340]}` }] }),
    livePlayer({ gameName: "Kindred Spirit", tagLine: "WOLF", team: "ORDER", championName: "Qiyana", championId: 246, position: "JUNGLE", level: 11, kills: 6, deaths: 2, assists: 4, creepScore: 98, rank: PREVIEW_RANKS[1], recent: PREVIEW_RECENT[1] }),
    livePlayer({ gameName: "spooky", tagLine: "2729", team: "ORDER", championName: "Taliyah", championId: 163, position: "MIDDLE", isSelf: true, level: 11, kills: 4, deaths: 1, assists: 7, creepScore: 142, rank: PREVIEW_RANKS[2], recent: PREVIEW_RECENT[2], items: [{ id: 6655, name: "Luden's Companion", slot: 0, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[6655]}` }, { id: 3020, name: "Sorcerer's Shoes", slot: 1, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[3020]}` }, { id: 1056, name: "Doran's Ring", slot: 2, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[1056]}` }, { id: 3340, name: "Stealth Ward", slot: 6, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[3340]}` }] }),
    livePlayer({ gameName: "wanan yuen", tagLine: "chaye", team: "ORDER", championName: "Twisted Fate", championId: 4, position: "BOTTOM", level: 9, kills: 1, deaths: 3, assists: 5, creepScore: 110 }),
    livePlayer({ gameName: "yuen", tagLine: "vvv", team: "ORDER", championName: "Lux", championId: 99, position: "UTILITY", level: 8, kills: 0, deaths: 2, assists: 11, creepScore: 18, rank: PREVIEW_RANKS[3], recent: PREVIEW_RECENT[3] }),
    livePlayer({ gameName: "Shopkeeper", tagLine: "123", team: "CHAOS", championName: "Wukong", championId: 62, position: "TOP", level: 10, kills: 2, deaths: 4, assists: 1, creepScore: 104, isDead: true, respawnTimer: 13, rank: PREVIEW_RANKS[2], recent: PREVIEW_RECENT[2], items: [{ id: 3047, name: "Plated Steelcaps", slot: 1, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[3047]}` }, { id: 3068, name: "Sunfire Aegis", slot: 0, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[3068]}` }] }),
    livePlayer({ gameName: "Odysseus", tagLine: "KR131", team: "CHAOS", championName: "Lee Sin", championId: 64, position: "JUNGLE", puuid: "enemy-2", level: 11, kills: 5, deaths: 3, assists: 3, creepScore: 95, rank: PREVIEW_RANKS[1], recent: PREVIEW_RECENT[0] }),
    livePlayer({ gameName: "", tagLine: "", nameHidden: true, team: "CHAOS", championName: "Ahri", championId: 103, position: "MIDDLE", level: 10, kills: 2, deaths: 4, assists: 2, creepScore: 130, rank: PREVIEW_RANKS[0], recent: PREVIEW_RECENT[1] }),
    livePlayer({ gameName: "Clozer", tagLine: "0727", team: "CHAOS", championName: "Jinx", championId: 222, position: "BOTTOM", level: 9, kills: 1, deaths: 2, assists: 1, creepScore: 118, items: [{ id: 6672, name: "Kraken Slayer", slot: 0, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[6672]}` }, { id: 3006, name: "Berserker's Greaves", slot: 1, icon: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${ICONS[3006]}` }] }),
    livePlayer({ gameName: "Simsons", tagLine: "spfd", team: "CHAOS", championName: "Thresh", championId: 412, position: "UTILITY", isBot: false, level: 8, kills: 0, deaths: 2, assists: 5, creepScore: 12 }),
  ],
};

const PREVIEW_ENEMIES = [
  ["Shopkeeper", "123", "top"],
  ["Odysseus", "KR131", "jungle"],
  ["SeTab", "123", "middle"],
  ["Clozer", "0727", "bottom"],
  ["Simsons", "spfd", "utility"],
].map(([game_name, game_tag, position], index) => ({
  ...PREVIEW_PARTICIPANTS[index],
  game_name,
  game_tag,
  name: game_name,
  puuid: `enemy-${index}`,
  pid: `enemy-${index}`,
  assigned_position: position,
  position_guessed: false,
  autofilled: false,
  is_self: false,
  is_enemy: true,
}));

const champion = (id: number, name: string, slug: string): ChampionRef => ({ id, name, slug });

// Lolalytics' and U.GG's World Master+ numbers from patch 26.21 for the live game preview's
// teams, weighed together like the backend does, with the enemy roles guessed as in ranked.
const PREVIEW_MATCHUPS: MatchupBoard = {
  provider: "lolalytics",
  patch: "26.21",
  error: null,
  selfRole: "middle",
  allies: [
    { champion: champion(266, "Aatrox", "aatrox"), role: "top", isSelf: false },
    { champion: champion(246, "Qiyana", "qiyana"), role: "jungle", isSelf: false },
    { champion: champion(163, "Taliyah", "taliyah"), role: "middle", isSelf: true },
    { champion: champion(51, "Caitlyn", "caitlyn"), role: "bottom", isSelf: false },
    { champion: champion(99, "Lux", "lux"), role: "utility", isSelf: false },
  ],
  enemies: [
    {
      "champion": {
        "id": 126,
        "name": "Jayce",
        "slug": "jayce"
      },
      "role": "top",
      "roleGuessed": true,
      "games": 26067,
      "winRate": 48.2,
      "loading": false,
      "counters": [
        {
          "champion": {
            "id": 54,
            "name": "Malphite",
            "slug": "malphite"
          },
          "stats": {
            "games": 2050,
            "winRate": 54.93,
            "winRateDelta": 3.71,
            "goldDiff15": -145.0,
            "xpDiff15": 397.0,
            "csDiff15": -10.4,
            "jungleCsDiff15": -0.1,
            "killDiff15": 0.67,
            "lolalytics": {
              "games": 2050,
              "winRate": 54.93,
              "winRateDelta": 3.71
            },
            "ugg": {
              "games": 1814,
              "winRate": 56.67,
              "winRateDelta": 4.27
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 14,
            "name": "Sion",
            "slug": "sion"
          },
          "stats": {
            "games": 623,
            "winRate": 55.86,
            "winRateDelta": 5.51,
            "goldDiff15": -140.0,
            "xpDiff15": 241.0,
            "csDiff15": -3.8,
            "jungleCsDiff15": 0.0,
            "killDiff15": -0.22,
            "lolalytics": {
              "games": 623,
              "winRate": 55.86,
              "winRateDelta": 5.51
            },
            "ugg": {
              "games": 582,
              "winRate": 57.04,
              "winRateDelta": 5.25
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 98,
            "name": "Shen",
            "slug": "shen"
          },
          "stats": {
            "games": 444,
            "winRate": 55.18,
            "winRateDelta": 3.39,
            "goldDiff15": -918.0,
            "xpDiff15": -389.0,
            "csDiff15": -30.1,
            "jungleCsDiff15": -0.3,
            "killDiff15": 0.05,
            "lolalytics": {
              "games": 444,
              "winRate": 55.18,
              "winRateDelta": 3.39
            },
            "ugg": {
              "games": 412,
              "winRate": 59.22,
              "winRateDelta": 5.73
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 39,
            "name": "Irelia",
            "slug": "irelia"
          },
          "stats": {
            "games": 1283,
            "winRate": 53.86,
            "winRateDelta": 2.94,
            "goldDiff15": 803.0,
            "xpDiff15": 646.0,
            "csDiff15": 18.8,
            "jungleCsDiff15": 0.4,
            "killDiff15": 0.62,
            "lolalytics": {
              "games": 1283,
              "winRate": 53.86,
              "winRateDelta": 2.94
            },
            "ugg": {
              "games": 1181,
              "winRate": 55.72,
              "winRateDelta": 3.09
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 516,
            "name": "Ornn",
            "slug": "ornn"
          },
          "stats": {
            "games": 584,
            "winRate": 53.6,
            "winRateDelta": 2.11,
            "goldDiff15": -511.0,
            "xpDiff15": 24.0,
            "csDiff15": -15.6,
            "jungleCsDiff15": -0.2,
            "killDiff15": 0.1,
            "lolalytics": {
              "games": 584,
              "winRate": 53.6,
              "winRateDelta": 2.11
            },
            "ugg": {
              "games": 555,
              "winRate": 55.5,
              "winRateDelta": 2.97
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 41,
            "name": "Gangplank",
            "slug": "gangplank"
          },
          "stats": {
            "games": 708,
            "winRate": 53.25,
            "winRateDelta": 1.43,
            "goldDiff15": -159.0,
            "xpDiff15": -125.0,
            "csDiff15": -9.5,
            "jungleCsDiff15": -0.1,
            "killDiff15": -0.24,
            "lolalytics": {
              "games": 708,
              "winRate": 53.25,
              "winRateDelta": 1.43
            },
            "ugg": {
              "games": 664,
              "winRate": 55.42,
              "winRateDelta": 2.04
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 2,
            "name": "Olaf",
            "slug": "olaf"
          },
          "stats": {
            "games": 713,
            "winRate": 53.3,
            "winRateDelta": 3.06,
            "goldDiff15": 147.0,
            "xpDiff15": 358.0,
            "csDiff15": -0.8,
            "jungleCsDiff15": -0.0,
            "killDiff15": 0.67,
            "lolalytics": {
              "games": 713,
              "winRate": 53.3,
              "winRateDelta": 3.06
            },
            "ugg": {
              "games": 636,
              "winRate": 54.56,
              "winRateDelta": 2.22
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 6,
            "name": "Urgot",
            "slug": "urgot"
          },
          "stats": {
            "games": 376,
            "winRate": 53.19,
            "winRateDelta": 1.49,
            "goldDiff15": -136.0,
            "xpDiff15": 160.0,
            "csDiff15": -8.9,
            "jungleCsDiff15": -0.1,
            "killDiff15": 0.84,
            "lolalytics": {
              "games": 376,
              "winRate": 53.19,
              "winRateDelta": 1.49
            },
            "ugg": {
              "games": 358,
              "winRate": 55.31,
              "winRateDelta": 1.58
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 83,
            "name": "Yorick",
            "slug": "yorick"
          },
          "stats": {
            "games": 563,
            "winRate": 53.64,
            "winRateDelta": 4.63,
            "goldDiff15": 261.0,
            "xpDiff15": 452.0,
            "csDiff15": 0.9,
            "jungleCsDiff15": 0.2,
            "killDiff15": 0.47,
            "lolalytics": {
              "games": 563,
              "winRate": 53.64,
              "winRateDelta": 4.63
            },
            "ugg": {
              "games": 515,
              "winRate": 53.59,
              "winRateDelta": 3.26
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 8,
            "name": "Vladimir",
            "slug": "vladimir"
          },
          "stats": {
            "games": 242,
            "winRate": 53.31,
            "winRateDelta": 3.53,
            "goldDiff15": 46.0,
            "xpDiff15": 525.0,
            "csDiff15": 0.4,
            "jungleCsDiff15": -0.2,
            "killDiff15": 0.75,
            "lolalytics": {
              "games": 242,
              "winRate": 53.31,
              "winRateDelta": 3.53
            },
            "ugg": {
              "games": 212,
              "winRate": 55.66,
              "winRateDelta": 3.55
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 79,
            "name": "Gragas",
            "slug": "gragas"
          },
          "stats": {
            "games": 555,
            "winRate": 52.61,
            "winRateDelta": 2.01,
            "goldDiff15": -267.0,
            "xpDiff15": 156.0,
            "csDiff15": -13.7,
            "jungleCsDiff15": -0.1,
            "killDiff15": 0.31,
            "lolalytics": {
              "games": 555,
              "winRate": 52.61,
              "winRateDelta": 2.01
            },
            "ugg": {
              "games": 525,
              "winRate": 53.9,
              "winRateDelta": 1.4
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 36,
            "name": "Dr. Mundo",
            "slug": "drmundo"
          },
          "stats": {
            "games": 434,
            "winRate": 53.0,
            "winRateDelta": 3.8,
            "goldDiff15": -499.0,
            "xpDiff15": -56.0,
            "csDiff15": -12.0,
            "jungleCsDiff15": -0.2,
            "killDiff15": -0.15,
            "lolalytics": {
              "games": 434,
              "winRate": 53.0,
              "winRateDelta": 3.8
            },
            "ugg": {
              "games": 393,
              "winRate": 53.69,
              "winRateDelta": 2.48
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 75,
            "name": "Nasus",
            "slug": "nasus"
          },
          "stats": {
            "games": 373,
            "winRate": 51.47,
            "winRateDelta": 0.15,
            "goldDiff15": -900.0,
            "xpDiff15": -486.0,
            "csDiff15": -22.6,
            "jungleCsDiff15": -0.1,
            "killDiff15": -0.5,
            "lolalytics": {
              "games": 373,
              "winRate": 51.47,
              "winRateDelta": 0.15
            },
            "ugg": {
              "games": 353,
              "winRate": 54.96,
              "winRateDelta": 1.96
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 92,
            "name": "Riven",
            "slug": "riven"
          },
          "stats": {
            "games": 250,
            "winRate": 51.2,
            "winRateDelta": -0.47,
            "goldDiff15": -342.0,
            "xpDiff15": -368.0,
            "csDiff15": -16.3,
            "jungleCsDiff15": 0.1,
            "killDiff15": -0.04,
            "lolalytics": {
              "games": 250,
              "winRate": 51.2,
              "winRateDelta": -0.47
            },
            "ugg": {
              "games": 225,
              "winRate": 55.11,
              "winRateDelta": 2.14
            },
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 80,
            "name": "Pantheon",
            "slug": "pantheon"
          },
          "stats": {
            "games": 816,
            "winRate": 51.47,
            "winRateDelta": 1.58,
            "goldDiff15": -109.0,
            "xpDiff15": -162.0,
            "csDiff15": -10.3,
            "jungleCsDiff15": -0.1,
            "killDiff15": 0.32,
            "lolalytics": {
              "games": 816,
              "winRate": 51.47,
              "winRateDelta": 1.58
            },
            "ugg": {
              "games": 729,
              "winRate": 53.22,
              "winRateDelta": 1.2
            },
            "agreement": 2,
            "sources": 2
          }
        }
      ],
      "ally": {
        "champion": {
          "id": 266,
          "name": "Aatrox",
          "slug": "aatrox"
        },
        "isSelf": false,
        "stats": {
          "games": 1185,
          "winRate": 47.09,
          "winRateDelta": -2.57,
          "goldDiff15": -74.0,
          "xpDiff15": 269.0,
          "csDiff15": -6.9,
          "jungleCsDiff15": 0.1,
          "killDiff15": 0.51,
          "lolalytics": {
            "games": 1185,
            "winRate": 47.09,
            "winRateDelta": -2.57
          },
          "ugg": {
            "games": 1067,
            "winRate": 49.3,
            "winRateDelta": -1.79
          },
          "agreement": 0,
          "sources": 2
        }
      }
    },
    {
      "champion": {
        "id": 120,
        "name": "Hecarim",
        "slug": "hecarim"
      },
      "role": "jungle",
      "roleGuessed": true,
      "games": 25012,
      "winRate": 50.87,
      "loading": false,
      "counters": [
        {
          "champion": {
            "id": 31,
            "name": "Cho'Gath",
            "slug": "chogath"
          },
          "stats": {
            "games": 688,
            "winRate": 52.47,
            "winRateDelta": 3.74,
            "goldDiff15": -625.0,
            "xpDiff15": -564.0,
            "csDiff15": -1.4,
            "jungleCsDiff15": -9.4,
            "killDiff15": -0.05,
            "lolalytics": {
              "games": 688,
              "winRate": 52.47,
              "winRateDelta": 3.74
            },
            "ugg": {
              "games": 620,
              "winRate": 55.32,
              "winRateDelta": 4.97
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 76,
            "name": "Nidalee",
            "slug": "nidalee"
          },
          "stats": {
            "games": 696,
            "winRate": 51.87,
            "winRateDelta": 3.11,
            "goldDiff15": -155.0,
            "xpDiff15": -452.0,
            "csDiff15": -2.5,
            "jungleCsDiff15": -8.3,
            "killDiff15": -0.05,
            "lolalytics": {
              "games": 696,
              "winRate": 51.87,
              "winRateDelta": 3.11
            },
            "ugg": {
              "games": 636,
              "winRate": 52.67,
              "winRateDelta": 1.08
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 154,
            "name": "Zac",
            "slug": "zac"
          },
          "stats": {
            "games": 368,
            "winRate": 51.09,
            "winRateDelta": 2.43,
            "goldDiff15": -681.0,
            "xpDiff15": -797.0,
            "csDiff15": -0.6,
            "jungleCsDiff15": -19.1,
            "killDiff15": -0.19,
            "lolalytics": {
              "games": 368,
              "winRate": 51.09,
              "winRateDelta": 2.43
            },
            "ugg": {
              "games": 327,
              "winRate": 55.05,
              "winRateDelta": 4.18
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 122,
            "name": "Darius",
            "slug": "darius"
          },
          "stats": {
            "games": 206,
            "winRate": 53.4,
            "winRateDelta": 3.99,
            "goldDiff15": -85.0,
            "xpDiff15": -170.0,
            "csDiff15": -0.2,
            "jungleCsDiff15": -4.3,
            "killDiff15": 0.24,
            "lolalytics": {
              "games": 206,
              "winRate": 53.4,
              "winRateDelta": 3.99
            },
            "ugg": {
              "games": 200,
              "winRate": 55.0,
              "winRateDelta": 3.59
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 163,
            "name": "Taliyah",
            "slug": "taliyah"
          },
          "stats": {
            "games": 406,
            "winRate": 51.23,
            "winRateDelta": 2.39,
            "goldDiff15": -67.0,
            "xpDiff15": -246.0,
            "csDiff15": 1.3,
            "jungleCsDiff15": -5.2,
            "killDiff15": -0.56,
            "lolalytics": {
              "games": 406,
              "winRate": 51.23,
              "winRateDelta": 2.39
            },
            "ugg": {
              "games": 361,
              "winRate": 53.74,
              "winRateDelta": 2.29
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 35,
            "name": "Shaco",
            "slug": "shaco"
          },
          "stats": {
            "games": 509,
            "winRate": 51.08,
            "winRateDelta": 2.1,
            "goldDiff15": -473.0,
            "xpDiff15": -560.0,
            "csDiff15": -3.0,
            "jungleCsDiff15": -9.8,
            "killDiff15": -0.14,
            "lolalytics": {
              "games": 509,
              "winRate": 51.08,
              "winRateDelta": 2.1
            },
            "ugg": {
              "games": 479,
              "winRate": 52.82,
              "winRateDelta": 1.28
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 200,
            "name": "Bel'Veth",
            "slug": "belveth"
          },
          "stats": {
            "games": 385,
            "winRate": 50.13,
            "winRateDelta": 1.84,
            "goldDiff15": -572.0,
            "xpDiff15": -702.0,
            "csDiff15": -1.5,
            "jungleCsDiff15": -9.4,
            "killDiff15": -0.4,
            "lolalytics": {
              "games": 385,
              "winRate": 50.13,
              "winRateDelta": 1.84
            },
            "ugg": {
              "games": 376,
              "winRate": 53.99,
              "winRateDelta": 3.32
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 517,
            "name": "Sylas",
            "slug": "sylas"
          },
          "stats": {
            "games": 1228,
            "winRate": 49.67,
            "winRateDelta": 2.79,
            "goldDiff15": -346.0,
            "xpDiff15": -627.0,
            "csDiff15": -1.5,
            "jungleCsDiff15": -14.3,
            "killDiff15": -0.09,
            "lolalytics": {
              "games": 1228,
              "winRate": 49.67,
              "winRateDelta": 2.79
            },
            "ugg": {
              "games": 1169,
              "winRate": 51.67,
              "winRateDelta": 2.38
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 203,
            "name": "Kindred",
            "slug": "kindred"
          },
          "stats": {
            "games": 543,
            "winRate": 50.83,
            "winRateDelta": 3.21,
            "goldDiff15": -196.0,
            "xpDiff15": -551.0,
            "csDiff15": -0.4,
            "jungleCsDiff15": -11.1,
            "killDiff15": -0.11,
            "lolalytics": {
              "games": 543,
              "winRate": 50.83,
              "winRateDelta": 3.21
            },
            "ugg": {
              "games": 490,
              "winRate": 51.02,
              "winRateDelta": 0.49
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 60,
            "name": "Elise",
            "slug": "elise"
          },
          "stats": {
            "games": 547,
            "winRate": 49.54,
            "winRateDelta": 0.95,
            "goldDiff15": -310.0,
            "xpDiff15": -740.0,
            "csDiff15": -2.7,
            "jungleCsDiff15": -16.5,
            "killDiff15": -0.05,
            "lolalytics": {
              "games": 547,
              "winRate": 49.54,
              "winRateDelta": 0.95
            },
            "ugg": {
              "games": 477,
              "winRate": 50.94,
              "winRateDelta": -0.36
            },
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 876,
            "name": "Lillia",
            "slug": "lillia"
          },
          "stats": {
            "games": 867,
            "winRate": 49.37,
            "winRateDelta": 1.33,
            "goldDiff15": -412.0,
            "xpDiff15": -358.0,
            "csDiff15": 0.3,
            "jungleCsDiff15": -5.7,
            "killDiff15": -0.33,
            "lolalytics": {
              "games": 867,
              "winRate": 49.37,
              "winRateDelta": 1.33
            },
            "ugg": {
              "games": 759,
              "winRate": 50.59,
              "winRateDelta": 0.47
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 20,
            "name": "Nunu & Willump",
            "slug": "nunu"
          },
          "stats": {
            "games": 381,
            "winRate": 49.61,
            "winRateDelta": -0.03,
            "goldDiff15": -733.0,
            "xpDiff15": -813.0,
            "csDiff15": -1.3,
            "jungleCsDiff15": -16.9,
            "killDiff15": -0.43,
            "lolalytics": {
              "games": 381,
              "winRate": 49.61,
              "winRateDelta": -0.03
            },
            "ugg": {
              "games": 327,
              "winRate": 51.07,
              "winRateDelta": -0.49
            },
            "agreement": 0,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 28,
            "name": "Evelynn",
            "slug": "evelynn"
          },
          "stats": {
            "games": 345,
            "winRate": 48.99,
            "winRateDelta": 0.98,
            "goldDiff15": -408.0,
            "xpDiff15": -619.0,
            "csDiff15": -3.4,
            "jungleCsDiff15": -15.8,
            "killDiff15": 0.07,
            "lolalytics": {
              "games": 345,
              "winRate": 48.99,
              "winRateDelta": 0.98
            },
            "ugg": {
              "games": 316,
              "winRate": 51.27,
              "winRateDelta": 1.17
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 77,
            "name": "Udyr",
            "slug": "udyr"
          },
          "stats": {
            "games": 253,
            "winRate": 48.22,
            "winRateDelta": 0.87,
            "goldDiff15": -252.0,
            "xpDiff15": -229.0,
            "csDiff15": -1.1,
            "jungleCsDiff15": -2.7,
            "killDiff15": -0.06,
            "lolalytics": {
              "games": 253,
              "winRate": 48.22,
              "winRateDelta": 0.87
            },
            "ugg": {
              "games": 249,
              "winRate": 51.81,
              "winRateDelta": 2.36
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 5,
            "name": "Xin Zhao",
            "slug": "xinzhao"
          },
          "stats": {
            "games": 483,
            "winRate": 48.45,
            "winRateDelta": 1.14,
            "goldDiff15": -593.0,
            "xpDiff15": -784.0,
            "csDiff15": -1.3,
            "jungleCsDiff15": -14.1,
            "killDiff15": -0.2,
            "lolalytics": {
              "games": 483,
              "winRate": 48.45,
              "winRateDelta": 1.14
            },
            "ugg": {
              "games": 448,
              "winRate": 50.0,
              "winRateDelta": 0.17
            },
            "agreement": 2,
            "sources": 2
          }
        }
      ],
      "ally": {
        "champion": {
          "id": 246,
          "name": "Qiyana",
          "slug": "qiyana"
        },
        "isSelf": false,
        "stats": {
          "games": 1410,
          "winRate": 41.13,
          "winRateDelta": -3.21,
          "goldDiff15": -23.0,
          "xpDiff15": -558.0,
          "csDiff15": -1.5,
          "jungleCsDiff15": -10.4,
          "killDiff15": -0.33,
          "lolalytics": {
            "games": 1410,
            "winRate": 41.13,
            "winRateDelta": -3.21
          },
          "ugg": {
            "games": 1256,
            "winRate": 43.87,
            "winRateDelta": -3.44
          },
          "agreement": 0,
          "sources": 2
        }
      }
    },
    {
      "champion": {
        "id": 103,
        "name": "Ahri",
        "slug": "ahri"
      },
      "role": "middle",
      "roleGuessed": true,
      "games": 30716,
      "winRate": 51.38,
      "loading": false,
      "counters": [
        {
          "champion": {
            "id": 91,
            "name": "Talon",
            "slug": "talon"
          },
          "stats": {
            "games": 361,
            "winRate": 50.97,
            "winRateDelta": 3.54,
            "goldDiff15": 428.0,
            "xpDiff15": -14.0,
            "csDiff15": -3.7,
            "jungleCsDiff15": 0.3,
            "killDiff15": 0.44,
            "lolalytics": {
              "games": 361,
              "winRate": 50.97,
              "winRateDelta": 3.54
            },
            "ugg": {
              "games": 311,
              "winRate": 54.34,
              "winRateDelta": 3.85
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 134,
            "name": "Syndra",
            "slug": "syndra"
          },
          "stats": {
            "games": 2018,
            "winRate": 49.16,
            "winRateDelta": 3.28,
            "goldDiff15": 131.0,
            "xpDiff15": 46.0,
            "csDiff15": 8.3,
            "jungleCsDiff15": 0.0,
            "killDiff15": -0.22,
            "lolalytics": {
              "games": 2018,
              "winRate": 49.16,
              "winRateDelta": 3.28
            },
            "ugg": {
              "games": 1765,
              "winRate": 51.27,
              "winRateDelta": 2.53
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 166,
            "name": "Akshan",
            "slug": "akshan"
          },
          "stats": {
            "games": 570,
            "winRate": 48.95,
            "winRateDelta": 2.43,
            "goldDiff15": 537.0,
            "xpDiff15": 38.0,
            "csDiff15": 0.5,
            "jungleCsDiff15": 0.0,
            "killDiff15": 0.13,
            "lolalytics": {
              "games": 570,
              "winRate": 48.95,
              "winRateDelta": 2.43
            },
            "ugg": {
              "games": 501,
              "winRate": 52.3,
              "winRateDelta": 3.08
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 893,
            "name": "Aurora",
            "slug": "aurora"
          },
          "stats": {
            "games": 682,
            "winRate": 48.97,
            "winRateDelta": 2.06,
            "goldDiff15": 188.0,
            "xpDiff15": 195.0,
            "csDiff15": 1.9,
            "jungleCsDiff15": -0.0,
            "killDiff15": 0.35,
            "lolalytics": {
              "games": 682,
              "winRate": 48.97,
              "winRateDelta": 2.06
            },
            "ugg": {
              "games": 630,
              "winRate": 51.9,
              "winRateDelta": 1.63
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 127,
            "name": "Lissandra",
            "slug": "lissandra"
          },
          "stats": {
            "games": 968,
            "winRate": 48.35,
            "winRateDelta": 1.1,
            "goldDiff15": 143.0,
            "xpDiff15": 145.0,
            "csDiff15": 4.9,
            "jungleCsDiff15": 0.1,
            "killDiff15": 0.02,
            "lolalytics": {
              "games": 968,
              "winRate": 48.35,
              "winRateDelta": 1.1
            },
            "ugg": {
              "games": 885,
              "winRate": 52.09,
              "winRateDelta": 2.06
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 245,
            "name": "Ekko",
            "slug": "ekko"
          },
          "stats": {
            "games": 898,
            "winRate": 49.0,
            "winRateDelta": 1.72,
            "goldDiff15": 126.0,
            "xpDiff15": 58.0,
            "csDiff15": -1.6,
            "jungleCsDiff15": 0.2,
            "killDiff15": 0.01,
            "lolalytics": {
              "games": 898,
              "winRate": 49.0,
              "winRateDelta": 1.72
            },
            "ugg": {
              "games": 807,
              "winRate": 51.05,
              "winRateDelta": 1.24
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 1,
            "name": "Annie",
            "slug": "annie"
          },
          "stats": {
            "games": 381,
            "winRate": 48.03,
            "winRateDelta": 1.36,
            "goldDiff15": -91.0,
            "xpDiff15": -141.0,
            "csDiff15": -1.7,
            "jungleCsDiff15": -0.0,
            "killDiff15": 0.01,
            "lolalytics": {
              "games": 381,
              "winRate": 48.03,
              "winRateDelta": 1.36
            },
            "ugg": {
              "games": 336,
              "winRate": 52.68,
              "winRateDelta": 2.79
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 4,
            "name": "Twisted Fate",
            "slug": "twistedfate"
          },
          "stats": {
            "games": 1018,
            "winRate": 49.41,
            "winRateDelta": 2.26,
            "goldDiff15": 310.0,
            "xpDiff15": -165.0,
            "csDiff15": -5.2,
            "jungleCsDiff15": -0.0,
            "killDiff15": -0.45,
            "lolalytics": {
              "games": 1018,
              "winRate": 49.41,
              "winRateDelta": 2.26
            },
            "ugg": {
              "games": 885,
              "winRate": 49.94,
              "winRateDelta": 0.09
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 238,
            "name": "Zed",
            "slug": "zed"
          },
          "stats": {
            "games": 1353,
            "winRate": 48.56,
            "winRateDelta": 2.57,
            "goldDiff15": 428.0,
            "xpDiff15": 111.0,
            "csDiff15": -3.2,
            "jungleCsDiff15": 0.1,
            "killDiff15": 0.27,
            "lolalytics": {
              "games": 1353,
              "winRate": 48.56,
              "winRateDelta": 2.57
            },
            "ugg": {
              "games": 1201,
              "winRate": 50.46,
              "winRateDelta": 2.26
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 55,
            "name": "Katarina",
            "slug": "katarina"
          },
          "stats": {
            "games": 1074,
            "winRate": 48.04,
            "winRateDelta": 0.37,
            "goldDiff15": 177.0,
            "xpDiff15": -92.0,
            "csDiff15": -11.4,
            "jungleCsDiff15": -0.0,
            "killDiff15": 0.23,
            "lolalytics": {
              "games": 1074,
              "winRate": 48.04,
              "winRateDelta": 0.37
            },
            "ugg": {
              "games": 962,
              "winRate": 51.04,
              "winRateDelta": 1.37
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 142,
            "name": "Zoe",
            "slug": "zoe"
          },
          "stats": {
            "games": 685,
            "winRate": 46.57,
            "winRateDelta": -0.47,
            "goldDiff15": 429.0,
            "xpDiff15": 247.0,
            "csDiff15": 4.6,
            "jungleCsDiff15": 0.2,
            "killDiff15": 0.62,
            "lolalytics": {
              "games": 685,
              "winRate": 46.57,
              "winRateDelta": -0.47
            },
            "ugg": {
              "games": 601,
              "winRate": 52.08,
              "winRateDelta": 1.47
            },
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 39,
            "name": "Irelia",
            "slug": "irelia"
          },
          "stats": {
            "games": 409,
            "winRate": 46.45,
            "winRateDelta": 1.25,
            "goldDiff15": 980.0,
            "xpDiff15": 415.0,
            "csDiff15": 21.1,
            "jungleCsDiff15": 0.3,
            "killDiff15": 0.49,
            "lolalytics": {
              "games": 409,
              "winRate": 46.45,
              "winRateDelta": 1.25
            },
            "ugg": {
              "games": 380,
              "winRate": 52.11,
              "winRateDelta": 4.77
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 161,
            "name": "Vel'Koz",
            "slug": "velkoz"
          },
          "stats": {
            "games": 223,
            "winRate": 50.22,
            "winRateDelta": 1.86,
            "goldDiff15": null,
            "xpDiff15": null,
            "csDiff15": null,
            "jungleCsDiff15": null,
            "killDiff15": null,
            "lolalytics": {
              "games": 223,
              "winRate": 50.22,
              "winRateDelta": 1.86
            },
            "ugg": null,
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 805,
            "name": "Locke",
            "slug": "locke"
          },
          "stats": {
            "games": 1285,
            "winRate": 47.08,
            "winRateDelta": 0.8,
            "goldDiff15": 472.0,
            "xpDiff15": -36.0,
            "csDiff15": -6.9,
            "jungleCsDiff15": 0.1,
            "killDiff15": 0.44,
            "lolalytics": {
              "games": 1285,
              "winRate": 47.08,
              "winRateDelta": 0.8
            },
            "ugg": {
              "games": 1145,
              "winRate": 50.22,
              "winRateDelta": 1.04
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 7,
            "name": "LeBlanc",
            "slug": "leblanc"
          },
          "stats": {
            "games": 1058,
            "winRate": 47.35,
            "winRateDelta": 0.9,
            "goldDiff15": 402.0,
            "xpDiff15": 235.0,
            "csDiff15": 2.3,
            "jungleCsDiff15": -0.0,
            "killDiff15": 0.82,
            "lolalytics": {
              "games": 1058,
              "winRate": 47.35,
              "winRateDelta": 0.9
            },
            "ugg": {
              "games": 942,
              "winRate": 49.89,
              "winRateDelta": 0.71
            },
            "agreement": 2,
            "sources": 2
          }
        }
      ],
      "ally": {
        "champion": {
          "id": 163,
          "name": "Taliyah",
          "slug": "taliyah"
        },
        "isSelf": true,
        "stats": {
          "games": 587,
          "winRate": 45.66,
          "winRateDelta": -1.56,
          "goldDiff15": -51.0,
          "xpDiff15": -102.0,
          "csDiff15": 0.9,
          "jungleCsDiff15": 0.4,
          "killDiff15": -0.36,
          "lolalytics": {
            "games": 587,
            "winRate": 45.66,
            "winRateDelta": -1.56
          },
          "ugg": {
            "games": 544,
            "winRate": 47.24,
            "winRateDelta": -3.13
          },
          "agreement": 0,
          "sources": 2
        }
      }
    },
    {
      "champion": {
        "id": 222,
        "name": "Jinx",
        "slug": "jinx"
      },
      "role": "bottom",
      "roleGuessed": true,
      "games": 37226,
      "winRate": 50.75,
      "loading": false,
      "counters": [
        {
          "champion": {
            "id": 99,
            "name": "Lux",
            "slug": "lux"
          },
          "stats": {
            "games": 220,
            "winRate": 59.55,
            "winRateDelta": 11.77,
            "goldDiff15": 335.0,
            "xpDiff15": 520.0,
            "csDiff15": 3.1,
            "jungleCsDiff15": -0.1,
            "killDiff15": 0.42,
            "lolalytics": {
              "games": 220,
              "winRate": 59.55,
              "winRateDelta": 11.77
            },
            "ugg": {
              "games": 216,
              "winRate": 59.26,
              "winRateDelta": 7.61
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 29,
            "name": "Twitch",
            "slug": "twitch"
          },
          "stats": {
            "games": 1236,
            "winRate": 51.29,
            "winRateDelta": 3.19,
            "goldDiff15": 283.0,
            "xpDiff15": 264.0,
            "csDiff15": -4.3,
            "jungleCsDiff15": -0.1,
            "killDiff15": 1.08,
            "lolalytics": {
              "games": 1236,
              "winRate": 51.29,
              "winRateDelta": 3.19
            },
            "ugg": {
              "games": 1139,
              "winRate": 55.22,
              "winRateDelta": 5.03
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 161,
            "name": "Vel'Koz",
            "slug": "velkoz"
          },
          "stats": {
            "games": 202,
            "winRate": 58.42,
            "winRateDelta": 8.7,
            "goldDiff15": null,
            "xpDiff15": null,
            "csDiff15": null,
            "jungleCsDiff15": null,
            "killDiff15": null,
            "lolalytics": {
              "games": 202,
              "winRate": 58.42,
              "winRateDelta": 8.7
            },
            "ugg": null,
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 134,
            "name": "Syndra",
            "slug": "syndra"
          },
          "stats": {
            "games": 263,
            "winRate": 51.71,
            "winRateDelta": 4.21,
            "goldDiff15": 301.0,
            "xpDiff15": 515.0,
            "csDiff15": 7.6,
            "jungleCsDiff15": -0.0,
            "killDiff15": 0.25,
            "lolalytics": {
              "games": 263,
              "winRate": 51.71,
              "winRateDelta": 4.21
            },
            "ugg": {
              "games": 242,
              "winRate": 54.55,
              "winRateDelta": 3.42
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 55,
            "name": "Katarina",
            "slug": "katarina"
          },
          "stats": {
            "games": 232,
            "winRate": 55.6,
            "winRateDelta": 7.52,
            "goldDiff15": null,
            "xpDiff15": null,
            "csDiff15": null,
            "jungleCsDiff15": null,
            "killDiff15": null,
            "lolalytics": {
              "games": 232,
              "winRate": 55.6,
              "winRateDelta": 7.52
            },
            "ugg": null,
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 115,
            "name": "Ziggs",
            "slug": "ziggs"
          },
          "stats": {
            "games": 309,
            "winRate": 50.49,
            "winRateDelta": 2.75,
            "goldDiff15": 192.0,
            "xpDiff15": 428.0,
            "csDiff15": 4.8,
            "jungleCsDiff15": -0.1,
            "killDiff15": 0.22,
            "lolalytics": {
              "games": 309,
              "winRate": 50.49,
              "winRateDelta": 2.75
            },
            "ugg": {
              "games": 269,
              "winRate": 52.04,
              "winRateDelta": 1.69
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 910,
            "name": "Hwei",
            "slug": "hwei"
          },
          "stats": {
            "games": 203,
            "winRate": 52.71,
            "winRateDelta": 2.84,
            "goldDiff15": null,
            "xpDiff15": null,
            "csDiff15": null,
            "jungleCsDiff15": null,
            "killDiff15": null,
            "lolalytics": {
              "games": 203,
              "winRate": 52.71,
              "winRateDelta": 2.84
            },
            "ugg": null,
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 112,
            "name": "Viktor",
            "slug": "viktor"
          },
          "stats": {
            "games": 388,
            "winRate": 49.74,
            "winRateDelta": 2.7,
            "goldDiff15": 25.0,
            "xpDiff15": 507.0,
            "csDiff15": 5.3,
            "jungleCsDiff15": -0.1,
            "killDiff15": 0.29,
            "lolalytics": {
              "games": 388,
              "winRate": 49.74,
              "winRateDelta": 2.7
            },
            "ugg": {
              "games": 369,
              "winRate": 49.59,
              "winRateDelta": -0.54
            },
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 119,
            "name": "Draven",
            "slug": "draven"
          },
          "stats": {
            "games": 992,
            "winRate": 47.88,
            "winRateDelta": 0.69,
            "goldDiff15": 1265.0,
            "xpDiff15": 384.0,
            "csDiff15": 13.6,
            "jungleCsDiff15": 0.2,
            "killDiff15": 0.68,
            "lolalytics": {
              "games": 992,
              "winRate": 47.88,
              "winRateDelta": 0.69
            },
            "ugg": {
              "games": 876,
              "winRate": 51.26,
              "winRateDelta": 1.19
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 498,
            "name": "Xayah",
            "slug": "xayah"
          },
          "stats": {
            "games": 1778,
            "winRate": 47.92,
            "winRateDelta": -0.34,
            "goldDiff15": 333.0,
            "xpDiff15": 324.0,
            "csDiff15": 3.5,
            "jungleCsDiff15": 0.1,
            "killDiff15": 0.52,
            "lolalytics": {
              "games": 1778,
              "winRate": 47.92,
              "winRateDelta": -0.34
            },
            "ugg": {
              "games": 1563,
              "winRate": 50.67,
              "winRateDelta": -0.45
            },
            "agreement": 0,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 30,
            "name": "Karthus",
            "slug": "karthus"
          },
          "stats": {
            "games": 220,
            "winRate": 47.73,
            "winRateDelta": -3.17,
            "goldDiff15": 328.0,
            "xpDiff15": 504.0,
            "csDiff15": 7.0,
            "jungleCsDiff15": 0.0,
            "killDiff15": 0.0,
            "lolalytics": {
              "games": 220,
              "winRate": 47.73,
              "winRateDelta": -3.17
            },
            "ugg": {
              "games": 203,
              "winRate": 49.75,
              "winRateDelta": -5.35
            },
            "agreement": 0,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 202,
            "name": "Jhin",
            "slug": "jhin"
          },
          "stats": {
            "games": 4294,
            "winRate": 47.28,
            "winRateDelta": 1.39,
            "goldDiff15": 319.0,
            "xpDiff15": 366.0,
            "csDiff15": 4.4,
            "jungleCsDiff15": -0.1,
            "killDiff15": 0.57,
            "lolalytics": {
              "games": 4294,
              "winRate": 47.28,
              "winRateDelta": 1.39
            },
            "ugg": {
              "games": 3773,
              "winRate": 50.65,
              "winRateDelta": 1.81
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 221,
            "name": "Zeri",
            "slug": "zeri"
          },
          "stats": {
            "games": 1494,
            "winRate": 47.39,
            "winRateDelta": 0.49,
            "goldDiff15": 425.0,
            "xpDiff15": 40.0,
            "csDiff15": 5.3,
            "jungleCsDiff15": 0.1,
            "killDiff15": 0.33,
            "lolalytics": {
              "games": 1494,
              "winRate": 47.39,
              "winRateDelta": 0.49
            },
            "ugg": {
              "games": 1341,
              "winRate": 49.59,
              "winRateDelta": -0.81
            },
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 50,
            "name": "Swain",
            "slug": "swain"
          },
          "stats": {
            "games": 233,
            "winRate": 46.35,
            "winRateDelta": -1.22,
            "goldDiff15": 104.0,
            "xpDiff15": 339.0,
            "csDiff15": -2.1,
            "jungleCsDiff15": 0.0,
            "killDiff15": 0.43,
            "lolalytics": {
              "games": 233,
              "winRate": 46.35,
              "winRateDelta": -1.22
            },
            "ugg": {
              "games": 206,
              "winRate": 48.54,
              "winRateDelta": -2.31
            },
            "agreement": 0,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 157,
            "name": "Yasuo",
            "slug": "yasuo"
          },
          "stats": {
            "games": 419,
            "winRate": 46.06,
            "winRateDelta": -2.66,
            "goldDiff15": 401.0,
            "xpDiff15": 466.0,
            "csDiff15": 5.9,
            "jungleCsDiff15": 0.1,
            "killDiff15": 0.75,
            "lolalytics": {
              "games": 419,
              "winRate": 46.06,
              "winRateDelta": -2.66
            },
            "ugg": {
              "games": 361,
              "winRate": 49.86,
              "winRateDelta": -2.02
            },
            "agreement": 0,
            "sources": 2
          }
        }
      ],
      "ally": {
        "champion": {
          "id": 51,
          "name": "Caitlyn",
          "slug": "caitlyn"
        },
        "isSelf": false,
        "stats": {
          "games": 2523,
          "winRate": 45.62,
          "winRateDelta": 0.81,
          "goldDiff15": 424.0,
          "xpDiff15": 259.0,
          "csDiff15": 9.4,
          "jungleCsDiff15": -0.0,
          "killDiff15": 0.28,
          "lolalytics": {
            "games": 2523,
            "winRate": 45.62,
            "winRateDelta": 0.81
          },
          "ugg": {
            "games": 2182,
            "winRate": 48.21,
            "winRateDelta": 0.95
          },
          "agreement": 2,
          "sources": 2
        }
      }
    },
    {
      "champion": {
        "id": 412,
        "name": "Thresh",
        "slug": "thresh"
      },
      "role": "utility",
      "roleGuessed": true,
      "games": 50367,
      "winRate": 51.82,
      "loading": false,
      "counters": [
        {
          "champion": {
            "id": 526,
            "name": "Rell",
            "slug": "rell"
          },
          "stats": {
            "games": 1737,
            "winRate": 51.53,
            "winRateDelta": 3.11,
            "goldDiff15": 2.0,
            "xpDiff15": -80.0,
            "csDiff15": -0.9,
            "jungleCsDiff15": 0.0,
            "killDiff15": -0.08,
            "lolalytics": {
              "games": 1737,
              "winRate": 51.53,
              "winRateDelta": 3.11
            },
            "ugg": {
              "games": 1550,
              "winRate": 54.06,
              "winRateDelta": 2.43
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 497,
            "name": "Rakan",
            "slug": "rakan"
          },
          "stats": {
            "games": 1844,
            "winRate": 50.38,
            "winRateDelta": 3.63,
            "goldDiff15": -50.0,
            "xpDiff15": -54.0,
            "csDiff15": 0.0,
            "jungleCsDiff15": -0.0,
            "killDiff15": -0.1,
            "lolalytics": {
              "games": 1844,
              "winRate": 50.38,
              "winRateDelta": 3.63
            },
            "ugg": {
              "games": 1659,
              "winRate": 52.56,
              "winRateDelta": 2.69
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 201,
            "name": "Braum",
            "slug": "braum"
          },
          "stats": {
            "games": 2430,
            "winRate": 49.92,
            "winRateDelta": 2.69,
            "goldDiff15": -134.0,
            "xpDiff15": -53.0,
            "csDiff15": -1.5,
            "jungleCsDiff15": -0.0,
            "killDiff15": -0.17,
            "lolalytics": {
              "games": 2430,
              "winRate": 49.92,
              "winRateDelta": 2.69
            },
            "ugg": {
              "games": 2237,
              "winRate": 51.72,
              "winRateDelta": 2.07
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 518,
            "name": "Neeko",
            "slug": "neeko"
          },
          "stats": {
            "games": 579,
            "winRate": 50.09,
            "winRateDelta": 5.2,
            "goldDiff15": 233.0,
            "xpDiff15": 117.0,
            "csDiff15": -0.0,
            "jungleCsDiff15": 0.1,
            "killDiff15": 0.13,
            "lolalytics": {
              "games": 579,
              "winRate": 50.09,
              "winRateDelta": 5.2
            },
            "ugg": {
              "games": 505,
              "winRate": 51.09,
              "winRateDelta": 3.09
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 37,
            "name": "Sona",
            "slug": "sona"
          },
          "stats": {
            "games": 757,
            "winRate": 48.75,
            "winRateDelta": 2.2,
            "goldDiff15": 38.0,
            "xpDiff15": 50.0,
            "csDiff15": -3.3,
            "jungleCsDiff15": 0.1,
            "killDiff15": 0.04,
            "lolalytics": {
              "games": 757,
              "winRate": 48.75,
              "winRateDelta": 2.2
            },
            "ugg": {
              "games": 717,
              "winRate": 51.32,
              "winRateDelta": 1.65
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 12,
            "name": "Alistar",
            "slug": "alistar"
          },
          "stats": {
            "games": 2622,
            "winRate": 48.44,
            "winRateDelta": 0.59,
            "goldDiff15": -132.0,
            "xpDiff15": -141.0,
            "csDiff15": -2.2,
            "jungleCsDiff15": -0.0,
            "killDiff15": -0.13,
            "lolalytics": {
              "games": 2622,
              "winRate": 48.44,
              "winRateDelta": 0.59
            },
            "ugg": {
              "games": 2375,
              "winRate": 50.19,
              "winRateDelta": -0.14
            },
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 147,
            "name": "Seraphine",
            "slug": "seraphine"
          },
          "stats": {
            "games": 1519,
            "winRate": 47.66,
            "winRateDelta": 1.83,
            "goldDiff15": 224.0,
            "xpDiff15": 182.0,
            "csDiff15": -0.8,
            "jungleCsDiff15": 0.0,
            "killDiff15": 0.22,
            "lolalytics": {
              "games": 1519,
              "winRate": 47.66,
              "winRateDelta": 1.83
            },
            "ugg": {
              "games": 1367,
              "winRate": 50.11,
              "winRateDelta": 1.79
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 60,
            "name": "Elise",
            "slug": "elise"
          },
          "stats": {
            "games": 374,
            "winRate": 48.66,
            "winRateDelta": 3.22,
            "goldDiff15": 620.0,
            "xpDiff15": -153.0,
            "csDiff15": -2.1,
            "jungleCsDiff15": 0.2,
            "killDiff15": 0.3,
            "lolalytics": {
              "games": 374,
              "winRate": 48.66,
              "winRateDelta": 3.22
            },
            "ugg": {
              "games": 345,
              "winRate": 49.57,
              "winRateDelta": 1.31
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 7,
            "name": "LeBlanc",
            "slug": "leblanc"
          },
          "stats": {
            "games": 258,
            "winRate": 48.06,
            "winRateDelta": 2.91,
            "goldDiff15": 478.0,
            "xpDiff15": -66.0,
            "csDiff15": -0.9,
            "jungleCsDiff15": 0.1,
            "killDiff15": 0.34,
            "lolalytics": {
              "games": 258,
              "winRate": 48.06,
              "winRateDelta": 2.91
            },
            "ugg": {
              "games": 243,
              "winRate": 50.62,
              "winRateDelta": 3.04
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 143,
            "name": "Zyra",
            "slug": "zyra"
          },
          "stats": {
            "games": 481,
            "winRate": 46.57,
            "winRateDelta": 2.28,
            "goldDiff15": 271.0,
            "xpDiff15": 151.0,
            "csDiff15": 0.1,
            "jungleCsDiff15": 0.2,
            "killDiff15": 0.2,
            "lolalytics": {
              "games": 481,
              "winRate": 46.57,
              "winRateDelta": 2.28
            },
            "ugg": {
              "games": 416,
              "winRate": 51.44,
              "winRateDelta": 4.49
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 57,
            "name": "Maokai",
            "slug": "maokai"
          },
          "stats": {
            "games": 441,
            "winRate": 47.17,
            "winRateDelta": 0.25,
            "goldDiff15": -18.0,
            "xpDiff15": -141.0,
            "csDiff15": -0.2,
            "jungleCsDiff15": 0.1,
            "killDiff15": -0.03,
            "lolalytics": {
              "games": 441,
              "winRate": 47.17,
              "winRateDelta": 0.25
            },
            "ugg": {
              "games": 402,
              "winRate": 50.25,
              "winRateDelta": 0.78
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 161,
            "name": "Vel'Koz",
            "slug": "velkoz"
          },
          "stats": {
            "games": 706,
            "winRate": 47.17,
            "winRateDelta": 0.27,
            "goldDiff15": 475.0,
            "xpDiff15": 330.0,
            "csDiff15": 3.1,
            "jungleCsDiff15": 0.1,
            "killDiff15": 0.54,
            "lolalytics": {
              "games": 706,
              "winRate": 47.17,
              "winRateDelta": 0.27
            },
            "ugg": {
              "games": 627,
              "winRate": 49.92,
              "winRateDelta": 1.48
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 78,
            "name": "Poppy",
            "slug": "poppy"
          },
          "stats": {
            "games": 411,
            "winRate": 46.96,
            "winRateDelta": 0.99,
            "goldDiff15": 111.0,
            "xpDiff15": -118.0,
            "csDiff15": -0.8,
            "jungleCsDiff15": 0.0,
            "killDiff15": 0.07,
            "lolalytics": {
              "games": 411,
              "winRate": 46.96,
              "winRateDelta": 0.99
            },
            "ugg": {
              "games": 365,
              "winRate": 49.86,
              "winRateDelta": 0.93
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 3,
            "name": "Galio",
            "slug": "galio"
          },
          "stats": {
            "games": 239,
            "winRate": 47.7,
            "winRateDelta": 1.12,
            "goldDiff15": 32.0,
            "xpDiff15": -38.0,
            "csDiff15": 0.7,
            "jungleCsDiff15": 0.0,
            "killDiff15": -0.04,
            "lolalytics": {
              "games": 239,
              "winRate": 47.7,
              "winRateDelta": 1.12
            },
            "ugg": {
              "games": 220,
              "winRate": 49.09,
              "winRateDelta": -0.59
            },
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 888,
            "name": "Renata Glasc",
            "slug": "renata"
          },
          "stats": {
            "games": 866,
            "winRate": 46.54,
            "winRateDelta": 1.45,
            "goldDiff15": -40.0,
            "xpDiff15": 26.0,
            "csDiff15": -2.0,
            "jungleCsDiff15": 0.0,
            "killDiff15": -0.03,
            "lolalytics": {
              "games": 866,
              "winRate": 46.54,
              "winRateDelta": 1.45
            },
            "ugg": {
              "games": 764,
              "winRate": 49.87,
              "winRateDelta": 2.16
            },
            "agreement": 2,
            "sources": 2
          }
        }
      ],
      "ally": {
        "champion": {
          "id": 99,
          "name": "Lux",
          "slug": "lux"
        },
        "isSelf": false,
        "stats": {
          "games": 1088,
          "winRate": 44.49,
          "winRateDelta": 0.16,
          "goldDiff15": 329.0,
          "xpDiff15": 230.0,
          "csDiff15": 2.3,
          "jungleCsDiff15": 0.1,
          "killDiff15": 0.22,
          "lolalytics": {
            "games": 1088,
            "winRate": 44.49,
            "winRateDelta": 0.16
          },
          "ugg": {
            "games": 972,
            "winRate": 47.43,
            "winRateDelta": 0.87
          },
          "agreement": 2,
          "sources": 2
        }
      }
    }
  ],
};

/** Jayce in the jungle and Hecarim top, for dragging them around the preview. */
const PREVIEW_ALTERNATES: Record<number, Partial<Record<Role, EnemyMatchups>>> = {
  126: {
    "jungle": {
      "champion": {
        "id": 126,
        "name": "Jayce",
        "slug": "jayce"
      },
      "role": "jungle",
      "roleGuessed": true,
      "games": 9428,
      "winRate": 47.16,
      "loading": false,
      "counters": [
        {
          "champion": {
            "id": 76,
            "name": "Nidalee",
            "slug": "nidalee"
          },
          "stats": {
            "games": 333,
            "winRate": 56.46,
            "winRateDelta": 3.59,
            "goldDiff15": 175.0,
            "xpDiff15": 145.0,
            "csDiff15": -2.9,
            "jungleCsDiff15": 0.5,
            "killDiff15": 0.55,
            "lolalytics": {
              "games": 333,
              "winRate": 56.46,
              "winRateDelta": 3.59
            },
            "ugg": {
              "games": 310,
              "winRate": 60.0,
              "winRateDelta": 4.69
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 245,
            "name": "Ekko",
            "slug": "ekko"
          },
          "stats": {
            "games": 206,
            "winRate": 56.8,
            "winRateDelta": 5.65,
            "goldDiff15": -71.0,
            "xpDiff15": 47.0,
            "csDiff15": -2.4,
            "jungleCsDiff15": -0.6,
            "killDiff15": 0.3,
            "lolalytics": {
              "games": 206,
              "winRate": 56.8,
              "winRateDelta": 5.65
            },
            "ugg": {
              "games": 206,
              "winRate": 59.22,
              "winRateDelta": 6.42
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 104,
            "name": "Graves",
            "slug": "graves"
          },
          "stats": {
            "games": 564,
            "winRate": 54.61,
            "winRateDelta": 4.82,
            "goldDiff15": 37.0,
            "xpDiff15": 239.0,
            "csDiff15": 1.0,
            "jungleCsDiff15": 4.4,
            "killDiff15": 0.29,
            "lolalytics": {
              "games": 564,
              "winRate": 54.61,
              "winRateDelta": 4.82
            },
            "ugg": {
              "games": 527,
              "winRate": 56.55,
              "winRateDelta": 4.56
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 163,
            "name": "Taliyah",
            "slug": "taliyah"
          },
          "stats": {
            "games": 223,
            "winRate": 53.81,
            "winRateDelta": 0.86,
            "goldDiff15": -49.0,
            "xpDiff15": 180.0,
            "csDiff15": -0.1,
            "jungleCsDiff15": 2.7,
            "killDiff15": 0.1,
            "lolalytics": {
              "games": 223,
              "winRate": 53.81,
              "winRateDelta": 0.86
            },
            "ugg": {
              "games": 200,
              "winRate": 54.0,
              "winRateDelta": -1.16
            },
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 203,
            "name": "Kindred",
            "slug": "kindred"
          },
          "stats": {
            "games": 234,
            "winRate": 52.14,
            "winRateDelta": 0.39,
            "goldDiff15": -33.0,
            "xpDiff15": 93.0,
            "csDiff15": -1.6,
            "jungleCsDiff15": -0.4,
            "killDiff15": 0.49,
            "lolalytics": {
              "games": 234,
              "winRate": 52.14,
              "winRateDelta": 0.39
            },
            "ugg": {
              "games": 219,
              "winRate": 53.42,
              "winRateDelta": -0.82
            },
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 62,
            "name": "Wukong",
            "slug": "wukong"
          },
          "stats": {
            "games": 219,
            "winRate": 51.14,
            "winRateDelta": -0.63,
            "goldDiff15": -577.0,
            "xpDiff15": -249.0,
            "csDiff15": -3.2,
            "jungleCsDiff15": -4.1,
            "killDiff15": 0.24,
            "lolalytics": {
              "games": 219,
              "winRate": 51.14,
              "winRateDelta": -0.63
            },
            "ugg": {
              "games": 206,
              "winRate": 54.37,
              "winRateDelta": 0.98
            },
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 120,
            "name": "Hecarim",
            "slug": "hecarim"
          },
          "stats": {
            "games": 346,
            "winRate": 52.6,
            "winRateDelta": 0.85,
            "goldDiff15": -37.0,
            "xpDiff15": 441.0,
            "csDiff15": -1.3,
            "jungleCsDiff15": 7.9,
            "killDiff15": 0.29,
            "lolalytics": {
              "games": 346,
              "winRate": 52.6,
              "winRateDelta": 0.85
            },
            "ugg": {
              "games": 320,
              "winRate": 52.81,
              "winRateDelta": -0.9
            },
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 5,
            "name": "Xin Zhao",
            "slug": "xinzhao"
          },
          "stats": {
            "games": 201,
            "winRate": 49.75,
            "winRateDelta": -1.69,
            "goldDiff15": null,
            "xpDiff15": null,
            "csDiff15": null,
            "jungleCsDiff15": null,
            "killDiff15": null,
            "lolalytics": {
              "games": 201,
              "winRate": 49.75,
              "winRateDelta": -1.69
            },
            "ugg": null,
            "agreement": 0,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 121,
            "name": "Kha'Zix",
            "slug": "khazix"
          },
          "stats": {
            "games": 403,
            "winRate": 50.62,
            "winRateDelta": 0.24,
            "goldDiff15": 19.0,
            "xpDiff15": 67.0,
            "csDiff15": -3.6,
            "jungleCsDiff15": 1.0,
            "killDiff15": 0.63,
            "lolalytics": {
              "games": 403,
              "winRate": 50.62,
              "winRateDelta": 0.24
            },
            "ugg": {
              "games": 387,
              "winRate": 52.97,
              "winRateDelta": 0.27
            },
            "agreement": 2,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 59,
            "name": "Jarvan IV",
            "slug": "jarvaniv"
          },
          "stats": {
            "games": 494,
            "winRate": 50.0,
            "winRateDelta": -0.64,
            "goldDiff15": -359.0,
            "xpDiff15": -70.0,
            "csDiff15": -2.6,
            "jungleCsDiff15": -2.7,
            "killDiff15": 0.11,
            "lolalytics": {
              "games": 494,
              "winRate": 50.0,
              "winRateDelta": -0.64
            },
            "ugg": {
              "games": 464,
              "winRate": 53.45,
              "winRateDelta": 0.84
            },
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 517,
            "name": "Sylas",
            "slug": "sylas"
          },
          "stats": {
            "games": 363,
            "winRate": 50.96,
            "winRateDelta": -0.05,
            "goldDiff15": -411.0,
            "xpDiff15": -166.0,
            "csDiff15": -2.1,
            "jungleCsDiff15": -6.8,
            "killDiff15": 0.17,
            "lolalytics": {
              "games": 363,
              "winRate": 50.96,
              "winRateDelta": -0.05
            },
            "ugg": {
              "games": 311,
              "winRate": 51.77,
              "winRateDelta": -1.24
            },
            "agreement": 0,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 31,
            "name": "Cho'Gath",
            "slug": "chogath"
          },
          "stats": {
            "games": 236,
            "winRate": 50.42,
            "winRateDelta": -2.42,
            "goldDiff15": -620.0,
            "xpDiff15": -102.0,
            "csDiff15": -2.9,
            "jungleCsDiff15": -0.3,
            "killDiff15": 0.25,
            "lolalytics": {
              "games": 236,
              "winRate": 50.42,
              "winRateDelta": -2.42
            },
            "ugg": {
              "games": 216,
              "winRate": 51.39,
              "winRateDelta": -2.68
            },
            "agreement": 0,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 876,
            "name": "Lillia",
            "slug": "lillia"
          },
          "stats": {
            "games": 220,
            "winRate": 48.18,
            "winRateDelta": -3.98,
            "goldDiff15": null,
            "xpDiff15": null,
            "csDiff15": null,
            "jungleCsDiff15": null,
            "killDiff15": null,
            "lolalytics": {
              "games": 220,
              "winRate": 48.18,
              "winRateDelta": -3.98
            },
            "ugg": null,
            "agreement": 0,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 234,
            "name": "Viego",
            "slug": "viego"
          },
          "stats": {
            "games": 380,
            "winRate": 49.74,
            "winRateDelta": -0.32,
            "goldDiff15": -117.0,
            "xpDiff15": 154.0,
            "csDiff15": -2.4,
            "jungleCsDiff15": 2.5,
            "killDiff15": 0.49,
            "lolalytics": {
              "games": 380,
              "winRate": 49.74,
              "winRateDelta": -0.32
            },
            "ugg": {
              "games": 342,
              "winRate": 52.34,
              "winRateDelta": 0.22
            },
            "agreement": 1,
            "sources": 2
          }
        },
        {
          "champion": {
            "id": 141,
            "name": "Kayn",
            "slug": "kayn"
          },
          "stats": {
            "games": 238,
            "winRate": 49.16,
            "winRateDelta": -1.34,
            "goldDiff15": -305.0,
            "xpDiff15": -233.0,
            "csDiff15": -1.6,
            "jungleCsDiff15": -3.9,
            "killDiff15": 0.14,
            "lolalytics": {
              "games": 238,
              "winRate": 49.16,
              "winRateDelta": -1.34
            },
            "ugg": {
              "games": 222,
              "winRate": 51.8,
              "winRateDelta": -0.07
            },
            "agreement": 0,
            "sources": 2
          }
        }
      ],
      "ally": {
        "champion": {
          "id": 246,
          "name": "Qiyana",
          "slug": "qiyana"
        },
        "isSelf": false,
        "stats": {
          "games": 503,
          "winRate": 50.5,
          "winRateDelta": 2.03,
          "goldDiff15": -19.0,
          "xpDiff15": 37.0,
          "csDiff15": -3.0,
          "jungleCsDiff15": 0.4,
          "killDiff15": 0.29,
          "lolalytics": {
            "games": 503,
            "winRate": 50.5,
            "winRateDelta": 2.03
          },
          "ugg": {
            "games": 465,
            "winRate": 51.83,
            "winRateDelta": 0.81
          },
          "agreement": 2,
          "sources": 2
        }
      }
    }
  },
  120: {
    "top": {
      "champion": {
        "id": 120,
        "name": "Hecarim",
        "slug": "hecarim"
      },
      "role": "top",
      "roleGuessed": true,
      "games": 40,
      "winRate": 55.0,
      "loading": false,
      "counters": [],
      "ally": {
        "champion": {
          "id": 266,
          "name": "Aatrox",
          "slug": "aatrox"
        },
        "isSelf": false,
        "stats": {
          "games": 1,
          "winRate": 0.0,
          "winRateDelta": -44.28,
          "goldDiff15": 923.0,
          "xpDiff15": 3085.0,
          "csDiff15": 54.0,
          "jungleCsDiff15": 0.0,
          "killDiff15": 1.0,
          "lolalytics": null,
          "ugg": {
            "games": 1,
            "winRate": 0.0,
            "winRateDelta": -44.28
          },
          "agreement": 0,
          "sources": 2
        }
      }
    }
  }
};

/** The preview board after dragging a pick to `role`, with that lane's real counters when known. */
export function previewWithRole(board: MatchupBoard, championId: number, role: Role): MatchupBoard {
  const moved = board.enemies.find((enemy) => enemy.champion.id === championId);
  if (!moved || moved.role === role) return board;
  const place = (enemy: EnemyMatchups, lane: Role | null): EnemyMatchups =>
    (lane && PREVIEW_ALTERNATES[enemy.champion.id]?.[lane]) ||
    PREVIEW_MATCHUPS.enemies.find((first) => first.champion.id === enemy.champion.id && first.role === lane) ||
    { ...enemy, role: lane, counters: [], ally: null };
  return {
    ...board,
    enemies: board.enemies.map((enemy) => {
      if (enemy === moved) return { ...place(enemy, role), roleGuessed: false };
      if (enemy.role === role) return place(enemy, moved.role);
      return enemy;
    }),
  };
}

/** `?counters=waiting` shows Champ Select before enemy picks, `?counters=loading` mid-download. */
function previewMatchups(): MatchupBoard | undefined {
  const mode = new URLSearchParams(window.location.search).get("counters");
  if (mode === "waiting") return undefined;
  if (mode === "loading") {
    return {
      ...PREVIEW_MATCHUPS,
      enemies: PREVIEW_MATCHUPS.enemies.slice(0, 2).map((enemy) => ({
        ...enemy,
        loading: true,
        counters: [],
        ally: enemy.ally && { ...enemy.ally, stats: null },
      })),
    };
  }
  return PREVIEW_MATCHUPS;
}

/** The preview board with the chosen site's win rates up front, as the backend would send it. */
export function previewForProvider(board: MatchupBoard, provider: CountersProvider): MatchupBoard {
  if (provider === board.provider) return board;
  const withMain = <T extends { stats: MatchupStats | null }>(entry: T): T => {
    if (!entry.stats) return entry;
    const main =
      provider === "ugg"
        ? (entry.stats.ugg ?? entry.stats.lolalytics)
        : (entry.stats.lolalytics ?? entry.stats.ugg);
    return { ...entry, stats: main ? { ...entry.stats, ...main } : entry.stats };
  };
  return {
    ...board,
    provider,
    enemies: board.enemies.map((enemy) => ({
      ...enemy,
      counters: enemy.counters.map(withMain),
      ally: enemy.ally && withMain(enemy.ally),
    })),
  };
}

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
      champSelect: {
        participants: new URLSearchParams(window.location.search).has("enemies")
          ? [...PREVIEW_PARTICIPANTS, ...PREVIEW_ENEMIES]
          : PREVIEW_PARTICIPANTS,
      },
      matchups: previewMatchups(),
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
