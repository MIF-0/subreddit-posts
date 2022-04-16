use log::info;
use serde_derive::{Serialize, Deserialize};
use serde_json::Value;
use crate::OAUTH_REDDIT_URL;
use crate::reddit_client::AuthRedditClient;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    id: String,
    oauth_client_id: String,
    pub name: String,
    pub display_name: String,
    pub display_name_prefixed: String,
    pub url: String,
}


pub async fn info(client: &AuthRedditClient) -> User {
    let url = format!("{}/api/v1/me", OAUTH_REDDIT_URL);

    let body = client.get(url.as_str()).await;

    let value: Value = serde_json::from_str(body.as_str()).expect("Json format expected");
    info!("The json value is {}", value);
    let subreddit = &value["subreddit"];
    User {
        id: String::from(value["id"].as_str().unwrap()),
        oauth_client_id: String::from(value["oauth_client_id"].as_str().unwrap()),
        name: String::from(subreddit["name"].as_str().unwrap()),
        display_name: String::from(subreddit["display_name"].as_str().unwrap()),
        display_name_prefixed: String::from(subreddit["display_name_prefixed"].as_str().unwrap()),
        url: String::from(subreddit["url"].as_str().unwrap()),
    }
}