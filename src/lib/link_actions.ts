import { writeText } from "@tauri-apps/api/clipboard";
import { open } from "@tauri-apps/api/shell";
import type { Participant } from "$lib/champ_select";
import type { MultiProvider } from "$lib/config";
import { isTauriRuntime } from "$lib/runtime";

function lobbyRegion(participants: Participant[]): string {
  const region = participants[0]?.region || "NA";
  return region === "SG2" ? "SG" : region;
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

  if (provider === "fow") {
    return `https://www.fow.lol/multi/${region.toLowerCase()}#${encodeURIComponent(joinedParticipants(participants, "#"))}`;
  }

  return `https://op.gg/lol/multisearch/${region}?summoners=${encodeURIComponent(joinedParticipants(participants, "#"))}`;
}

export function createOpggProfileLink(participant: Participant): string {
  const region = participant.region === "SG2" ? "SG" : participant.region;
  const profile = `${participant.game_name}-${participant.game_tag}`;
  return `https://op.gg/lol/summoners/${region.toLowerCase()}/${encodeURIComponent(profile)}`;
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
