import type { ChampSelect } from "$lib/champ_select";

interface PreviewState {
  connected: boolean;
  state: string;
  champSelect: ChampSelect | null;
}

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
}));

export function getPreviewState(): PreviewState {
  const preview = new URLSearchParams(window.location.search).get("preview");

  if (preview === "champ-select") {
    return {
      connected: true,
      state: "ChampSelect",
      champSelect: { participants: PREVIEW_PARTICIPANTS },
    };
  }

  if (preview === "disconnected") {
    return { connected: false, state: "Unknown", champSelect: null };
  }

  if (preview === "in-game") {
    return { connected: true, state: "InProgress", champSelect: null };
  }

  return { connected: true, state: "Lobby", champSelect: null };
}
