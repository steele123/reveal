import { writeText } from "@tauri-apps/api/clipboard";
import { open } from "@tauri-apps/api/shell";
import type { Participant } from "$lib/champ_select";
import { MULTI_PROVIDERS, type MultiProvider } from "$lib/config";
import { isTauriRuntime } from "$lib/runtime";

export const DISCORD_URL = "https://lobbyreveal.app/discord";

// Lobby chat reports Riot platform IDs ("NA1", "LA2") or chat servers ("EU1" for EUW);
// sites want web regions. Reveal now sends web regions, but saved lobbies may not.
const WEB_REGIONS: Record<string, string> = {
  NA1: "NA",
  EU1: "EUW",
  EU2: "EUNE",
  EUW1: "EUW",
  EUN1: "EUNE",
  BR1: "BR",
  JP1: "JP",
  OC1: "OCE",
  TR1: "TR",
  LA1: "LAN",
  LA2: "LAS",
  ME1: "ME",
  SG2: "SG",
  PH2: "PH",
  TH2: "TH",
  TW2: "TW",
  VN2: "VN",
};

/** "na1" -> "NA", "LA2" -> "LAS"; web regions like "EUW" pass through. */
export function webRegion(region: string): string {
  const upper = (region || "NA").toUpperCase();
  return WEB_REGIONS[upper] ?? upper;
}

function lobbyRegion(participants: Participant[]): string {
  return webRegion(participants[0]?.region || "NA");
}

// Maps League's web region (e.g. "NA", "EUNE", "LAS") to Riot's platform ID, which U.GG uses.
const RIOT_PLATFORM_IDS: Record<string, string> = {
  NA: "na1",
  NA1: "na1",
  EUW: "euw1",
  EUW1: "euw1",
  EUNE: "eun1",
  EUN1: "eun1",
  KR: "kr",
  BR: "br1",
  BR1: "br1",
  JP: "jp1",
  JP1: "jp1",
  RU: "ru",
  OCE: "oc1",
  OC1: "oc1",
  TR: "tr1",
  TR1: "tr1",
  LAN: "la1",
  LA1: "la1",
  LAS: "la2",
  LA2: "la2",
  ME: "me1",
  ME1: "me1",
  SG: "sg2",
  SG2: "sg2",
  PH: "ph2",
  PH2: "ph2",
  TH: "th2",
  TH2: "th2",
  TW: "tw2",
  TW2: "tw2",
  VN: "vn2",
  VN2: "vn2",
};

function riotPlatformId(region: string): string {
  return (
    RIOT_PLATFORM_IDS[region.toUpperCase()] ?? `${region.toLowerCase()}1`
  );
}

// Maps League's web region (or a Riot platform ID) to Porofessor's region slug.
const POROFESSOR_REGIONS: Record<string, string> = {
  NA1: "na",
  EUW1: "euw",
  EUN1: "eune",
  BR1: "br",
  JP1: "jp",
  OC1: "oce",
  TR1: "tr",
  LA1: "lan",
  LA2: "las",
  ME1: "me",
  SG2: "sg",
  PH2: "ph",
  TH2: "th",
  TW2: "tw",
  VN2: "vn",
};

function porofessorRegion(region: string): string {
  return POROFESSOR_REGIONS[region.toUpperCase()] ?? region.toLowerCase();
}

function joinedParticipants(
  participants: Participant[],
  separator: "#" | "-",
): string {
  return participants
    .map(
      (participant) =>
        `${participant.game_name}${separator}${participant.game_tag}`,
    )
    .join(",");
}

export function createMultiSearchLink(
  participants: Participant[],
  provider: MultiProvider,
): string {
  const region = lobbyRegion(participants);

  if (provider === "ugg") {
    return `https://u.gg/lol/multisearch?region=${riotPlatformId(region)}&summoners=${encodeURIComponent(joinedParticipants(participants, "-"))}`;
  }

  if (provider === "laddereye") {
    return `https://laddereye.com/multi?q=${encodeURIComponent(joinedParticipants(participants, "#"))}&region=${riotPlatformId(region).toUpperCase()}`;
  }

  if (provider === "porofessor") {
    const players = participants
      .map((participant) =>
        encodeURIComponent(`${participant.game_name}-${participant.game_tag}`),
      )
      .join(",");
    return `https://porofessor.gg/pregame/${porofessorRegion(region)}/${players}`;
  }

  if (provider === "fow") {
    return `https://www.fow.lol/multi/${region.toLowerCase()}#${encodeURIComponent(joinedParticipants(participants, "#"))}`;
  }

  return `https://op.gg/lol/multisearch/${region}?summoners=${encodeURIComponent(joinedParticipants(participants, "#"))}`;
}

/** Riot IDs as "Name#TAG, Name#TAG", which multi-search sites accept when pasted. */
export function formatRiotIds(participants: Participant[]): string {
  return participants
    .map((participant) => `${participant.game_name}#${participant.game_tag}`)
    .join(", ");
}

export function providerLabel(provider: MultiProvider): string {
  return MULTI_PROVIDERS.find(({ value }) => value === provider)?.label ?? "OP.GG";
}

/** One player's page on the chosen site. */
export function createProfileLink(
  participant: Participant,
  provider: MultiProvider,
): string {
  const region = webRegion(participant.region);
  const riotId = encodeURIComponent(`${participant.game_name}-${participant.game_tag}`);

  if (provider === "ugg") {
    return `https://u.gg/lol/profile/${riotPlatformId(region)}/${riotId}/overview`;
  }

  if (provider === "laddereye") {
    return `https://laddereye.com/player/${riotPlatformId(region).toUpperCase()}/${riotId}`;
  }

  // Porofessor has no profile pages, but its pre-game lookup works for a single player.
  if (provider === "porofessor") {
    return `https://porofessor.gg/pregame/${porofessorRegion(region)}/${riotId}`;
  }

  if (provider === "fow") {
    return `https://www.fow.lol/find/${region.toLowerCase()}/${riotId}`;
  }

  return `https://op.gg/lol/summoners/${region.toLowerCase()}/${riotId}`;
}

export async function openExternalLink(link: string): Promise<void> {
  if (isTauriRuntime()) {
    await open(link);
    return;
  }

  window.open(link, "_blank", "noopener,noreferrer");
}

export async function copyLink(link: string): Promise<void> {
  if (isTauriRuntime()) {
    await writeText(link);
    return;
  }

  await navigator.clipboard.writeText(link);
}
