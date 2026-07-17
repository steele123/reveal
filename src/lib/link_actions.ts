import { writeText } from "@tauri-apps/api/clipboard";
import { open } from "@tauri-apps/api/shell";
import type { Participant } from "$lib/champ_select";
import type { MultiProvider } from "$lib/config";
import { isTauriRuntime } from "$lib/runtime";

function lobbyRegion(participants: Participant[]): string {
  const region = participants[0]?.region || "NA";
  return region === "SG2" ? "SG" : region;
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

  if (provider === "deeplol") {
    return `https://deeplol.gg/multi/${region}/${encodeURIComponent(joinedParticipants(participants, "#"))}`;
  }

  if (provider === "ugg") {
    return `https://u.gg/multisearch?region=${region.toLowerCase()}1&summoners=${encodeURIComponent(joinedParticipants(participants, "-"))}`;
  }

  if (provider === "tracker") {
    return `https://tracker.gg/lol/multisearch/${region}/${encodeURIComponent(joinedParticipants(participants, "#"))}`;
  }

  return `https://www.op.gg/multisearch/${region}?summoners=${encodeURIComponent(joinedParticipants(participants, "#"))}`;
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
