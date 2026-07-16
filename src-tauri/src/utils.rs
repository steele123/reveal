use crate::lobby::{Lobby, Participant};
use urlencoding::encode;

pub fn create_opgg_link(summoners: &[Participant], region: &str) -> String {
    let base_url = format!("https://www.op.gg/multisearch/{}?summoners=", region);
    let participants = join_participants(summoners, '#');
    let encoded_path = encode(&participants);
    format!("{}{}", base_url, encoded_path)
}

pub fn create_deeplol_link(summoners: &[Participant], region: &str) -> String {
    let base_url = format!("https://deeplol.gg/multi/{}/", region);
    let participants = join_participants(summoners, '#');
    let encoded_path = encode(&participants);
    format!("{}{}", base_url, encoded_path)
}

pub fn create_ugg_link(summoners: &[Participant], region: &str) -> String {
    let base_url = format!("https://u.gg/multisearch?region={}", region.to_lowercase());
    let participants = join_participants(summoners, '-');
    let encoded_path = encode(&participants);
    format!("{}&summoners={}", base_url, encoded_path)
}

pub fn create_tracker_link(summoners: &[Participant], region: &str) -> String {
    let base_url = format!("https://tracker.gg/lol/multisearch/{}/", region);
    let participants = join_participants(summoners, '#');
    let encoded_path = encode(&participants);
    format!("{}{}", base_url, encoded_path)
}

pub fn create_multi_link(lobby: &Lobby, region: &str, site: &str) -> anyhow::Result<String> {
    if lobby.participants.is_empty() {
        anyhow::bail!("cannot create a multi link without participants");
    }

    let link = match site {
        "opgg" => create_opgg_link(&lobby.participants, region),
        "deeplol" => create_deeplol_link(&lobby.participants, region),
        "ugg" => create_ugg_link(&lobby.participants, &format!("{}1", region)),
        "tracker" => create_tracker_link(&lobby.participants, region),
        _ => anyhow::bail!("unsupported multi provider: {site}"),
    };

    Ok(link)
}

pub fn display_champ_select(lobby: &Lobby, region: &str, site: &str) -> anyhow::Result<()> {
    println!("Team: {}", join_participants(&lobby.participants, '#'));
    let link = create_multi_link(lobby, region, site)?;
    open::that(link)?;
    Ok(())
}

fn join_participants(participants: &[Participant], tag_separator: char) -> String {
    participants
        .iter()
        .map(|participant| {
            format!(
                "{}{}{}",
                participant.game_name, tag_separator, participant.game_tag
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lobby() -> Lobby {
        Lobby {
            participants: vec![Participant {
                cid: "champ-select".to_string(),
                game_name: "Player One".to_string(),
                game_tag: "NA1".to_string(),
                muted: false,
                name: "Player One".to_string(),
                pid: "pid".to_string(),
                puuid: "puuid".to_string(),
                region: "NA".to_string(),
            }],
        }
    }

    #[test]
    fn creates_provider_specific_links() {
        let lobby = lobby();

        assert_eq!(
            create_multi_link(&lobby, "NA", "opgg").unwrap(),
            "https://www.op.gg/multisearch/NA?summoners=Player%20One%23NA1"
        );
        assert_eq!(
            create_multi_link(&lobby, "NA", "ugg").unwrap(),
            "https://u.gg/multisearch?region=na1&summoners=Player%20One-NA1"
        );
    }

    #[test]
    fn rejects_invalid_link_requests() {
        assert!(create_multi_link(
            &Lobby {
                participants: vec![]
            },
            "NA",
            "opgg"
        )
        .is_err());
        assert!(create_multi_link(&lobby(), "NA", "unknown").is_err());
    }
}
