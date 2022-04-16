use log::{debug, info};
use crate::{OAUTH_REDDIT_URL};

use serde_derive::{Serialize, Deserialize};
use serde_json::Value;
use crate::reddit_client::AuthRedditClient;

#[derive(Serialize, Deserialize, Debug)]
pub struct FlairInfo {
    sub_reddit: String,
    text: String,
    id: String,
}

pub async fn retrieve_flairs_for(subreddits: Vec<&str>, client: &AuthRedditClient) -> Vec<FlairInfo> {
    let mut result = Vec::new();
    for subreddit in subreddits {
        let mut flairs = retrieve_flairs(subreddit, client).await;
        info!("Flairs for {} \n is {:?}", subreddit, flairs);
        result.append(&mut flairs);
    }
    result
}

async fn retrieve_flairs(subreddit: &str, client: &AuthRedditClient) -> Vec<FlairInfo> {
    let url = format!("{}/r/{}/api/link_flair_v2.json?raw_json=1", OAUTH_REDDIT_URL, subreddit);

    let body = client.get(url.as_str()).await;
    let json: Value = serde_json::from_str(body.as_str()).expect("Json format expected");
    debug!("The json value is {}", json);
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
