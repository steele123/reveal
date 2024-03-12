use crate::lobby::{Lobby, Participant};
use urlencoding::encode;

pub fn create_opgg_link(summoners: &Vec<Participant>, region: &str) -> String {
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

pub fn create_deeplol_link(summoners: &Vec<Participant>, region: &str) -> String {
    let base_url = format!("https://deeplol.gg/multi/{}/", region);
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

pub fn create_ugg_link(summoners: &Vec<Participant>, region: String) -> String {
    let base_url = format!("https://u.gg/multisearch?region={}", region.to_lowercase());
    let mut link_path = String::new();
    for summoner in summoners {
        let full_tag = format!("{}-{}", summoner.game_name, summoner.game_tag);
        link_path.push_str(&full_tag);
        link_path.push(',');
    }
    link_path.pop();

    let encoded_path = encode(&link_path);
    format!("{}&summoners={}", base_url, encoded_path)
}

pub fn create_tracker_link(summoners: &Vec<Participant>, region: &str) -> String {
    let base_url = format!("https://tracker.gg/lol/multisearch/{}/", region);
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

pub fn display_champ_select(lobby: &Lobby, region: &str, site: &String) {
    if lobby.participants.is_empty() {
        return;
    }

    let mut team_string = String::new();
    for summoner in lobby.participants.iter() {
        let participant = format!(
            "{}#{} ({})",
            summoner.game_name, summoner.game_tag, summoner.name
        );
        team_string.push_str(&participant);
        if summoner.name != lobby.participants.last().unwrap().name {
            team_string.push_str(", ");
        }
    }

    println!("Team: {}", team_string);
    let link = match site.as_str() {
        "opgg" => create_opgg_link(&lobby.participants, region),
        "deeplol" => create_deeplol_link(&lobby.participants, region),
        "ugg" => create_ugg_link(&lobby.participants, format!("{}1", region)),
        "tracker" => create_tracker_link(&lobby.participants, region),
        _ => panic!("Invalid site"),
    };

    match open::that(&link) {
        Ok(_) => {}
        Err(_) => {
            println!("Failed to open link in browser");
        }
    }
}
