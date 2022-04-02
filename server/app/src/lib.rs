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

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        let url = String::from("https://www.reddit.com/r/youareart/comments/tuijnl/my_new_picturef/");
        let parts: Vec<&str> = url.split("/").collect();
        let comment_position = parts.iter().position(|part| *part == "comments")
            .expect("comment in link");
        let id_position = comment_position + 1;
        let id = parts.get(id_position).map(|id| String::from(*id));
        println!("{:?}", parts);
        println!("{:?}", id);
    }
}