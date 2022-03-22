use log::info;
use serde_derive::{Serialize, Deserialize};
use uuid::Uuid;
use crate::OAUTH_REDDIT_URL;

#[derive(Serialize, Deserialize, Debug)]
pub struct Posts {
    pub main_post_info: MainPostInfo,
    pub posts: Vec<Post>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MainPostInfo {
    pub post_type: String,
    pub body: String,
    pub title: String,
    pub nsfw: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    pub subreddit: String,
    pub body_override: Option<String>,
    pub title_override: Option<String>,
    pub additional_title: Option<String>,
    pub flair_id: Option<String>,
    pub flair_name: Option<String>,
    pub need_to_be_posted: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FinalPost {
    sr: String,
    resubmit: bool,
    sendreplies: bool,
    title: String,
    nsfw: bool,
    spoiler: bool,
    subreddit: String,
    flair_id: Option<String>,
    flair_name: Option<String>,
    url: String,
    kind: String,
}

impl FinalPost {
    fn new(main_post_info: &MainPostInfo, post: &Post, url: String) -> FinalPost {
        let mut title = post.title_override.clone().unwrap_or(main_post_info.title.clone());
        if post.additional_title.is_some() {
            title.push_str(&post.additional_title.clone().unwrap());
        }
        let nsfw = main_post_info.nsfw;
        let subreddit = post.subreddit.clone();
        let flair_id = post.flair_id.clone();
        let flair_name = post.flair_name.clone();
        let kind = main_post_info.post_type.clone();

        FinalPost {
            sr: subreddit.clone(),
            resubmit: false,
            sendreplies: false,
            title,
            nsfw,
            spoiler: false,
            subreddit,
            flair_id,
            flair_name,
            url,
            kind,
        }
    }
}

pub async fn post(posts: Posts, token: &str) {
    let final_posts: Vec<FinalPost> = create_final_posts(posts);
    info!("Final posts: {:?}", final_posts);
    let client = reqwest::Client::builder()
        .build()
        .expect("error during client build");

    for post in final_posts  {
        let url = format!("{}/r/{}/api/submit", OAUTH_REDDIT_URL, post.subreddit);

        let result = client.post(url)
            .bearer_auth(token)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept-Language", "en-us")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3865.90 Safari/537.36")
            .body(serde_urlencoded::to_string(&post).expect("serialize issue during obtain auth token"))
            .send()
            .await;

        let body = result
            .expect("Result is empty")
            .text().await
            .expect("Body is empty");

        info!("Result submit body is {:?}", body);
    }
}

fn create_final_posts(posts: Posts) -> Vec<FinalPost> {
    posts.posts
        .iter()
        .filter(|value| value.need_to_be_posted.unwrap_or(true))
        .map(|post| {
            let body = post.body_override.clone().unwrap_or(posts.main_post_info.body.clone());
            let url = match posts.main_post_info.post_type.as_str() {
               "link" => body,
                _ => panic!("Unsupported post type")
            };
            FinalPost::new(&posts.main_post_info, post, url)
        })
        .collect()
}