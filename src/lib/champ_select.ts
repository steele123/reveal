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
}
