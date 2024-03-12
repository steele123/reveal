use serde::{Deserialize, Serialize};
use shaco::rest::RESTClient;

#[derive(Debug, Serialize, Deserialize)]
pub struct Participant {
    pub cid: String,
    pub game_name: String,
    pub game_tag: String,
    pub muted: bool,
    pub name: String,
    pub pid: String,
    pub puuid: String,
    pub region: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lobby {
    pub participants: Vec<Participant>,
}

pub async fn get_lobby_info(app_client: &RESTClient) -> Lobby {
    let team: Lobby = serde_json::from_value(
        app_client
            .get("/chat/v5/participants".to_string())
            .await
            .unwrap(),
    )
    .unwrap();

    // filter out all cids that contain champ-select
    let team_participants = team
        .participants
        .into_iter()
        .filter(|p| p.cid.contains("champ-select"))
        .collect::<Vec<Participant>>();

    let team = Lobby {
        participants: team_participants,
    };

    team
}
