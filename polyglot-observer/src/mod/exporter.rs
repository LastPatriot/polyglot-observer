use crate::traits::Exporter;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LokiExporter {
    loki_url: String,
    client: Client,
    target_language: String,
}

impl LokiExporter {
    pub fn new(loki_url: String, target_language: String) -> Self {
        Self {
            loki_url,
            client: Client::new(),
            target_language,
        }
    }
}

#[async_trait]
impl Exporter for LokiExporter {
    async fn export(&self, namespace: &str, pod: &str, container: &str, localized_text: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string();

        let payload = json!({
            "streams": [{
                "stream": {
                    "namespace": namespace,
                    "pod": pod,
                    "container": container,
                    "language": self.target_language,
                    "origin": "lingo-observer"
                },
                "values": [
                    [now, localized_text]
                ]
            }]
        });

        let _ = self.client.post(&self.loki_url)
            .json(&payload)
            .send()
            .await;
    }
}
