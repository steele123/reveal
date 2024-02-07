use urlencoding::encode;
use crate::lobby::{Lobby, Participant};

pub fn create_opgg_link(summoners: &Vec<Participant>, region: String) -> String {
    let base_url = format!("https://www.op.gg/multisearch/{}?summoners=", region);
    let mut link_path = String::new();
    for summoner in summoners {
        let full_tag = format!("{}#{}", summoner.game_name, summoner.game_tag);
        link_path.push_str(&full_tag);
        link_path.push(',');
    }
    link_path.pop();

    let encoded_path = encode(&link_path);
    format!("{}{}", base_url, encoded_path)
}

pub fn display_champ_select(lobby: &Lobby, region: String) {
    if lobby.participants.is_empty() {
        return;
    }

    let mut team_string = String::new();
    for summoner in lobby.participants.iter() {
        let participant = format!("{}#{} ({})", summoner.game_name, summoner.game_tag, summoner.name);
        team_string.push_str(&participant);
        if summoner.name != lobby.participants.last().unwrap().name {
            team_string.push_str(", ");
        }
    }

    println!("Team: {}", team_string);
    let link = create_opgg_link(&lobby.participants, region);
    match open::that(&link) {
        Ok(_) => {}
        Err(_) => {
            println!("Failed to open link in browser");
        }
    }
}