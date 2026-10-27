use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionInfo {
    pub web_region: String,
}

impl RegionInfo {
    /// The region multi-search and profile sites use, e.g. "NA", "EUW" or "SG".
    pub fn web(&self) -> String {
        web_region(&self.web_region)
    }
}

/// Maps a Riot platform ID ("NA1", "SG2") or chat server ("EU1") to its web region
/// ("NA", "SG", "EUW"). Web regions pass through, uppercased.
pub fn web_region(region: &str) -> String {
    let upper = region.to_uppercase();
    let web = match upper.as_str() {
        "" | "NA1" => "NA",
        "EUW1" | "EU1" => "EUW",
        "EUN1" | "EU2" => "EUNE",
        "BR1" => "BR",
        "JP1" => "JP",
        "OC1" => "OCE",
        "TR1" => "TR",
        "LA1" => "LAN",
        "LA2" => "LAS",
        "ME1" => "ME",
        "SG2" => "SG",
        "PH2" => "PH",
        "TH2" => "TH",
        "TW2" => "TW",
        "VN2" => "VN",
        _ => return upper,
    };
    web.to_string()
}

#[cfg(test)]
mod tests {
    use super::web_region;

    #[test]
    fn maps_platform_ids_and_chat_servers_to_web_regions() {
        for (input, expected) in [
            ("na1", "NA"),
            ("NA", "NA"),
            ("eu1", "EUW"),
            ("EUW1", "EUW"),
            ("euw", "EUW"),
            ("eu2", "EUNE"),
            ("EUN1", "EUNE"),
            ("LA1", "LAN"),
            ("LA2", "LAS"),
            ("OC1", "OCE"),
            ("SG2", "SG"),
            ("PH2", "PH"),
            ("TH2", "TH"),
            ("TW2", "TW"),
            ("VN2", "VN"),
            ("KR", "KR"),
            ("RU", "RU"),
            ("", "NA"),
        ] {
            assert_eq!(web_region(input), expected, "{input}");
        }
    }
}
