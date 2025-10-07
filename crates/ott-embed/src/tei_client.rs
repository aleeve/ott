use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct TextEmbedding {
    client: Arc<reqwest::Client>,
    url: String,
}

impl TextEmbedding {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: Arc::new(
                reqwest::Client::builder()
                    .timeout(Duration::from_secs(30))
                    .build()
                    .expect("Failed to create HTTP client"),
            ),
        }
    }

    pub async fn embed(&self, message: &str) -> Result<Vec<f32>, String> {
        // Create JSON payload
        let payload = json!({
            "inputs": message,
        });

        // Send POST request
        let response = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|error| error.to_string())?;

        let data = response
            .json::<Vec<Vec<f32>>>()
            .await
            .map_err(|e| e.to_string())?[0]
            .clone();

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use crate::tei_client::TextEmbedding;

    #[fixture]
    fn tei_url() -> String {
        "http://localhost:8080".to_string()
    }

    #[fixture]
    fn tei_client(tei_url: String) -> TextEmbedding {
        TextEmbedding::new(tei_url.as_str())
    }

    #[rstest]
    #[tokio::test]
    async fn embed_string(tei_client: TextEmbedding) {
        let resp = tei_client.embed("This is a test").await;
        assert!(resp.is_ok());
    }
}
