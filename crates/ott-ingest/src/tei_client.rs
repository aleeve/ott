use serde_json::{Error, json};
use std::time::Duration;
use tokio::sync::OnceCell;

static HTTP_CLIENT: OnceCell<reqwest::Client> = OnceCell::const_new();

// Simple function to get the shared client
async fn get_client() -> &'static reqwest::Client {
    HTTP_CLIENT
        .get_or_init(|| async {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client")
        })
        .await
}

async fn send_json_post(url: &str, message: &str) -> Result<Vec<f32>, String> {
    let client = get_client().await;

    // Create JSON payload
    let payload = json!({
        "inputs": message,
    });

    // Send POST request
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    // TODO
    let data = response.json::<Vec<Vec<f32>>>().await.unwrap()[0].clone();
    Ok(data)
}
