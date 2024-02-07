export interface ChampSelect {
  participants: Participant[];
}

export interface Participant {
  cid: String;
  game_name: String;
  game_tag: String;
  muted: boolean;
  name?: String;
  pid: String;
  puuid: String;
  region: String;
}
