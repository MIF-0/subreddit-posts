use std::thread::Thread;
use std::{thread, time};
use log::info;
use reqwest::Client;
use serde_derive::{Serialize, Deserialize};
use serde_json::Value;
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
    pub comment: Option<String>,
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
    pub comment: Option<String>,
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
    comment: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Comment {
    return_rtjson: bool,
    text: String,
    thing_id: String,
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
        let comment = main_post_info.comment.clone().or(post.comment.clone());

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
            comment,
        }
    }
}

pub async fn post(posts: Posts, token: &str) {
    let final_posts: Vec<FinalPost> = create_final_posts(posts);
    info!("Final posts: {:?}", final_posts);
    let client = reqwest::Client::builder()
        .build()
        .expect("error during client build");

    for post in final_posts {
        let full_ulr = submit_post(token, &client, &post).await;
        info!("post url is {:?}", full_ulr);

        if post.comment.is_some() && full_ulr.is_some() {
            submit_comment(token, &client, post.comment.clone().unwrap(), full_ulr.unwrap()).await;
        }
    }
}

async fn submit_post(token: &str, client: &Client, post: &FinalPost) -> Option<String> {
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

    // limit 60 post in a second
    let ten_millis = time::Duration::from_millis(21);
    thread::sleep(ten_millis);
    let post_url = retrieve_post_url(post, body);

    match post_url {
        Some(url) => retrieve_post_id(url),
        None => None
    }
}

fn retrieve_post_url(post: &FinalPost, body: String) -> Option<String> {
    let json: Value = serde_json::from_str(body.as_str()).expect("Json format expected");
    let value = json.get("jquery").expect("jquery field");
    let values = value.as_array().expect("some values");
    for value in values {
        let subvalues = value.as_array().expect("some values");
        info!("array value {:?}", value);
        for subvalue in subvalues {
            info!("array SUB value {:?}", subvalue);
            let possible_value = subvalue.as_array();
            if possible_value.is_none() {
                continue;
            }
            let possible_value = possible_value.unwrap();
            if possible_value.len() == 0 {
                continue;
            }
            let possible_value = possible_value[0].as_str().unwrap_or("");
            info!("Value to check {:?}", possible_value);
            if possible_value.contains(format!("www.reddit.com/r/{}/comments", post.subreddit).as_str()) {
                return Some(String::from(possible_value));
            }
        }
    }
    None
}

fn retrieve_post_id(post_url: String) -> Option<String> {
    let parts: Vec<&str> = post_url.split("/").collect();
    let comment_position = parts.iter().position(|part| *part == "comments");
    if comment_position.is_none() {
        return None;
    }
    let id_position = comment_position.unwrap() + 1;

    parts.get(id_position).map(|id| String::from(*id))
}

async fn submit_comment(token: &str, client: &Client, comment: String, post_id: String) {
    let post_id = format!("t3_{}", post_id);
    info!("Post id {:?}", post_id.as_str());

    let url = format!("{}/by_id/{}", OAUTH_REDDIT_URL, post_id);
    let result = client.get(url)
        .bearer_auth(token)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept-Language", "en-us")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3865.90 Safari/537.36")
        .send()
        .await;

    let body = result
        .expect("Result is empty")
        .text().await
        .expect("Body is empty");

    info!("Result get listing body is {:?}", body);


    let comment = Comment {
        return_rtjson: false,
        text: comment,
        thing_id: post_id,
    };

    let url = format!("{}/api/comment", OAUTH_REDDIT_URL);

    let result = client.post(url)
        .bearer_auth(token)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept-Language", "en-us")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3865.90 Safari/537.36")
        .body(serde_urlencoded::to_string(&comment).expect("serialize issue during obtain auth token"))
        .send()
        .await;

    let body = result
        .expect("Result is empty")
        .text().await
        .expect("Body is empty");

    info!("Result submit body is {:?}", body);

    // limit 60 post in a second
    let ten_millis = time::Duration::from_millis(21);
    thread::sleep(ten_millis);
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