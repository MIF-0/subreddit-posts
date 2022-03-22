pub mod environment;
pub mod login;
pub mod data_store;
pub mod in_memory_data_store;
pub mod post;
pub mod flairs;
mod media;

use serde_derive::{Serialize, Deserialize};
const REDDIT_URL: &str = "https://www.reddit.com";
const OAUTH_REDDIT_URL: &str = "https://oauth.reddit.com";

#[derive(Debug, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    token_type: String,
    expires_in: u32,
    scope: String,
    refresh_token: Option<String>,
}