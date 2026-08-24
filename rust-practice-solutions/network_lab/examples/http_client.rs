use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
struct GetResponse {
    url: String,
}

#[derive(Debug, Serialize)]
struct CreateRequest<'a> {
    name: &'a str,
}

async fn get_with_retry(
    client: &Client,
    url: &str,
    attempts: usize,
) -> Result<reqwest::Response, reqwest::Error> {
    assert!(attempts > 0);
    let mut delay = Duration::from_millis(10);

    for attempt in 0..attempts {
        match client.get(url).send().await {
            Ok(response) if response.status().is_success() => return Ok(response),
            Ok(response) if response.status().is_server_error() && attempt + 1 < attempts => {}
            Ok(response) => return response.error_for_status(),
            Err(error) if attempt + 1 < attempts => eprintln!("retry: {error}"),
            Err(error) => return Err(error),
        }
        sleep(delay).await;
        delay = (delay * 2).min(Duration::from_secs(1));
    }
    unreachable!()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url =
        std::env::var("DEMO_HTTP_BASE_URL").unwrap_or_else(|_| "https://httpbin.org".to_owned());
    let client = Client::builder()
        .user_agent("rust-network-verify/0.1")
        .timeout(Duration::from_secs(5))
        .build()?;

    let response = client
        .get(format!("{base_url}/get"))
        .query(&[("page", "1"), ("limit", "10")])
        .send()
        .await?
        .error_for_status()?;
    let data: GetResponse = response.json().await?;
    println!("GET {}", data.url);

    let created = client
        .post(format!("{base_url}/post"))
        .json(&CreateRequest { name: "An" })
        .send()
        .await?
        .error_for_status()?;
    println!("POST status {}", created.status());

    let _ = get_with_retry(&client, &format!("{base_url}/status/200"), 3).await?;
    Ok(())
}
