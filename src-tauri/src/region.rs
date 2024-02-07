use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionInfo {
    pub locale: String,
    pub region: String,
    pub web_language: String,
    pub web_region: String,
}
