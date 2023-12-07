use machineid_rs::{Encryption, HWIDComponent, IdBuilder};

pub struct ApiClient {
    pub client: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String, license_key: String) -> Self {
        let hwid = IdBuilder::new(Encryption::SHA256)
            .add_component(HWIDComponent::SystemID)
            .add_component(HWIDComponent::CPUID)
            .add_component(HWIDComponent::DriveSerial)
            .build("r3v341")
            .unwrap();

        let ua = format!("Reveal/{}", env!("CARGO_PKG_VERSION"));
        let client = reqwest::Client::builder()
            .user_agent(ua)
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert("System", hwid.parse().unwrap());
                headers.insert("License", license_key.parse().unwrap());
                headers
            })
            .build()
            .unwrap();

        Self { client, base_url }
    }

    pub async fn get(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.client.get(&format!("{}/{}", self.base_url, path)).send().await
    }

    pub async fn post(
        &self,
        path: &str,
        body: impl serde::Serialize,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .post(&format!("{}/{}", self.base_url, path))
            .json(&body)
            .send()
            .await
    }
}
