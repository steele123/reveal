export interface ChampSelect {
  participants: Participant[];
}

export interface Participant {
  cid: string;
  game_name: string;
  game_tag: string;
  muted: boolean;
  name?: string;
  pid: string;
  puuid: string;
  region: string;
  assigned_position?: string;
  cell_id?: number;
  pick_turn?: number;
}

const POSITION_LABELS: Record<string, string> = {
  top: "Top",
  jungle: "Jungle",
  middle: "Mid",
  bottom: "ADC",
  utility: "Support",
};

export function formatAssignedPosition(position: string): string {
  return (
    POSITION_LABELS[position.toLowerCase()] ||
    `${position.slice(0, 1).toUpperCase()}${position.slice(1)}`
  );
}
