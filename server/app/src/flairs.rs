use log::info;
use crate::{OAUTH_REDDIT_URL};

use serde_derive::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct FlairInfo {
    sub_reddit: String,
    text: String,
    id: String,
}

pub async fn retrieve_flairs_for(subreddits: Vec<&str>, auth_token: &str) -> Vec<FlairInfo> {
    let mut result = Vec::new();
    for subreddit in subreddits {
        let mut flairs = retrieve_flairs(subreddit, auth_token).await;
        info!("Flairs for {} \n is {:?}", subreddit, flairs);
        result.append(&mut flairs);
    }
    result
}

async fn retrieve_flairs(subreddit: &str, auth_token: &str) -> Vec<FlairInfo> {
    let url = format!("{}/r/{}/api/link_flair_v2.json?raw_json=1", OAUTH_REDDIT_URL, subreddit);
    let client = reqwest::Client::builder()
        .build()
        .expect("error during client build");

    let result = client.get(url)
        .bearer_auth(auth_token)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept-Language", "en-us")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3865.90 Safari/537.36")
        .send()
        .await;

    let body = result
        .expect("Result is empty")
        .text().await
        .expect("Body is empty");

    info!("Result body is {:?}", body);
    let json: Value = serde_json::from_str(body.as_str()).expect("Json format expected");
    info!("The json value is {}", json);
    json.as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|value|
            FlairInfo {
                sub_reddit: String::from(subreddit),
                text: value["text"].to_string(),
                id: value["id"].to_string(),
            })
        .collect()
}
