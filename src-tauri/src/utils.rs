use crate::lobby::{Lobby, Participant};
use urlencoding::encode;

/// Multi-search providers Reveal can open. Saved configs naming anything else fall back to OP.GG.
pub const MULTI_PROVIDERS: &[&str] = &["opgg", "ugg", "fow"];

pub fn create_opgg_link(summoners: &[Participant], region: &str) -> String {
    let base_url = format!("https://op.gg/lol/multisearch/{}?summoners=", region);
    let participants = join_participants(summoners, '#');
    let encoded_path = encode(&participants);
    format!("{}{}", base_url, encoded_path)
}

pub fn create_ugg_link(summoners: &[Participant], region: &str) -> String {
    let base_url = format!(
        "https://u.gg/lol/multisearch?region={}",
        riot_platform_id(region)
    );
    let participants = join_participants(summoners, '-');
    let encoded_path = encode(&participants);
    format!("{}&summoners={}", base_url, encoded_path)
}

/// Maps League's web region (e.g. "NA", "EUNE", "LAS") to Riot's platform ID, which U.GG uses.
fn riot_platform_id(region: &str) -> String {
    let platform = match region.to_uppercase().as_str() {
        "NA" | "NA1" => "na1",
        "EUW" | "EUW1" => "euw1",
        "EUNE" | "EUN1" => "eun1",
        "KR" => "kr",
        "BR" | "BR1" => "br1",
        "JP" | "JP1" => "jp1",
        "RU" => "ru",
        "OCE" | "OC1" => "oc1",
        "TR" | "TR1" => "tr1",
        "LAN" | "LA1" => "la1",
        "LAS" | "LA2" => "la2",
        "ME" | "ME1" => "me1",
        "SG" | "SG2" => "sg2",
        "PH" | "PH2" => "ph2",
        "TH" | "TH2" => "th2",
        "TW" | "TW2" => "tw2",
        "VN" | "VN2" => "vn2",
        _ => return format!("{}1", region.to_lowercase()),
    };
    platform.to_string()
}

pub fn create_fow_link(summoners: &[Participant], region: &str) -> String {
    let base_url = format!("https://www.fow.lol/multi/{}#", region.to_lowercase());
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
        "ugg" => create_ugg_link(&lobby.participants, region),
        "fow" => create_fow_link(&lobby.participants, region),
        _ => anyhow::bail!("unsupported multi provider: {site}"),
    };

    Ok(link)
}

pub fn display_champ_select(lobby: &Lobby, region: &str, site: &str) -> anyhow::Result<()> {
    log_info!(
        "Opening {site} multi-search for {} participants in region {region}",
        lobby.participants.len()
    );
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
                assigned_position: None,
                cell_id: None,
                pick_turn: None,
            }],
        }
    }

    #[test]
    fn creates_provider_specific_links() {
        let lobby = lobby();

        assert_eq!(
            create_multi_link(&lobby, "NA", "opgg").unwrap(),
            "https://op.gg/lol/multisearch/NA?summoners=Player%20One%23NA1"
        );
        assert_eq!(
            create_multi_link(&lobby, "NA", "ugg").unwrap(),
            "https://u.gg/lol/multisearch?region=na1&summoners=Player%20One-NA1"
        );
        assert_eq!(
            create_multi_link(&lobby, "KR", "ugg").unwrap(),
            "https://u.gg/lol/multisearch?region=kr&summoners=Player%20One-NA1"
        );
        assert_eq!(
            create_multi_link(&lobby, "SG", "fow").unwrap(),
            "https://www.fow.lol/multi/sg#Player%20One%23NA1"
        );
    }

    #[test]
    fn creates_fow_multi_links_with_multiple_riot_ids() {
        let mut lobby = lobby();
        lobby.participants.push(Participant {
            cid: "champ-select".to_string(),
            game_name: "Second Player".to_string(),
            game_tag: "0911".to_string(),
            muted: false,
            name: "Second Player".to_string(),
            pid: "second-pid".to_string(),
            puuid: "second-puuid".to_string(),
            region: "SG".to_string(),
            assigned_position: None,
            cell_id: None,
            pick_turn: None,
        });

        assert_eq!(
            create_multi_link(&lobby, "SG", "fow").unwrap(),
            "https://www.fow.lol/multi/sg#Player%20One%23NA1%2CSecond%20Player%230911"
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
        assert!(create_multi_link(&lobby(), "NA", "deeplol").is_err());
        assert!(create_multi_link(&lobby(), "NA", "tracker").is_err());
    }

    #[test]
    fn maps_web_regions_to_ugg_platform_ids() {
        for (region, platform) in [
            ("NA", "na1"),
            ("EUNE", "eun1"),
            ("KR", "kr"),
            ("OCE", "oc1"),
            ("LAN", "la1"),
            ("LAS", "la2"),
            ("RU", "ru"),
            ("SG", "sg2"),
            ("VN2", "vn2"),
        ] {
            assert_eq!(riot_platform_id(region), platform);
        }
    }
}
