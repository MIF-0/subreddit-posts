use crate::comment::submit_comment;
use crate::reddit_client::{AuthRedditClient, DeleteRequest};
use crate::user::User;
use crate::OAUTH_REDDIT_URL;
use log::{debug, info};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

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

impl FinalPost {
    fn new(main_post_info: &MainPostInfo, post: &Post, url: String) -> FinalPost {
        let mut title = post
            .title_override
            .clone()
            .unwrap_or(main_post_info.title.clone());
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

#[derive(Serialize, Deserialize, Debug)]
pub struct PostInfo {
    pub id: String,
    pub upvotes: u64,
    pub name: String,
    pub subreddit: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct PostInfos {
    after: Option<String>,
    posts: Vec<PostInfo>,
}

pub async fn post(posts: Posts, client: &AuthRedditClient) {
    let final_posts: Vec<FinalPost> = create_final_posts(posts);
    info!("Final posts: {:?}", final_posts);
    for post in final_posts {
        let full_ulr = submit_post(&client, &post).await;
        info!("post url is {:?}", full_ulr);

        if post.comment.is_some() && full_ulr.is_some() {
            submit_comment(&client, post.comment.clone().unwrap(), full_ulr.unwrap()).await;
        }
    }
}

pub async fn delete_with_upvotes_lt(client: &AuthRedditClient, user: &User, min_votes: u64) {
    let posts = retrieve_all_posts_with(client, user, |post| post.upvotes < min_votes).await;
    for post in posts {
        let delete_request = DeleteRequest::new_json(post.name.as_str());
        info!("Will delete {:?}", post);
        client.delete(&delete_request).await;
    }
    info!("Posts deleted");
}

pub async fn retrieve_all_posts_with(
    client: &AuthRedditClient,
    user: &User,
    filter: impl Fn(&PostInfo) -> bool,
) -> Vec<PostInfo> {
    let mut after: Option<String> = None;
    let mut result: Vec<PostInfo> = Vec::new();
    loop {
        let posts = retrieve_all_posts(client, user, after.clone()).await;
        info!("Retrieved {} posts", posts.posts.len());

        after = posts.after.clone();
        if posts.posts.len().eq(&0) {
            break;
        }
        for post in posts.posts {
            if after.is_some() {
                let value = after.clone().unwrap();
                if value.eq(&post.name) {
                    continue;
                }
            }
            if filter(&post) {
                result.push(post);
            }
        }

        if after.is_none() {
            break;
        }
    }
    return result;
}

async fn submit_post(client: &AuthRedditClient, post: &FinalPost) -> Option<String> {
    let url = format!("{}/r/{}/api/submit", OAUTH_REDDIT_URL, post.subreddit);

    let body = client.post(url.as_str(), Some(post)).await;

    let post_url = retrieve_post_url(post, body);
    match post_url {
        Some(url) => retrieve_post_id(url),
        None => None,
    }
}

fn retrieve_post_url(post: &FinalPost, body: String) -> Option<String> {
    let json: Value = serde_json::from_str(body.as_str()).expect("Json format expected");
    let value = json.get("jquery");
    if value.is_none() {
        return None;
    }
    let value = value.expect("jquery field");
    let values = value.as_array().expect("some values");
    for value in values {
        let subvalues = value.as_array().expect("some values");
        debug!("array value {:?}", value);
        for subvalue in subvalues {
            debug!("array SUB value {:?}", subvalue);
            let possible_value = subvalue.as_array();
            if possible_value.is_none() {
                continue;
            }
            let possible_value = possible_value.unwrap();
            if possible_value.len() == 0 {
                continue;
            }
            let possible_value = possible_value[0].as_str().unwrap_or("");
            debug!("Value to check {:?}", possible_value);
            if possible_value
                .contains(format!("www.reddit.com/r/{}/comments", post.subreddit).as_str())
            {
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

fn create_final_posts(posts: Posts) -> Vec<FinalPost> {
    posts
        .posts
        .iter()
        .filter(|value| value.need_to_be_posted.unwrap_or(true))
        .map(|post| {
            let body = post
                .body_override
                .clone()
                .unwrap_or(posts.main_post_info.body.clone());
            let url = match posts.main_post_info.post_type.as_str() {
                "link" => body,
                _ => panic!("Unsupported post type"),
            };
            FinalPost::new(&posts.main_post_info, post, url)
        })
        .collect()
}

async fn retrieve_all_posts(
    client: &AuthRedditClient,
    user: &User,
    after: Option<String>,
) -> PostInfos {
    let mut url = format!(
        "{}{}submitted?limit=1000",
        OAUTH_REDDIT_URL,
        user.url.as_str()
    );
    if after.is_some() {
        url = format!("{}&after={}", url, after.unwrap());
    }
    let body = client.get(url.as_str()).await;

    let value: Value = serde_json::from_str(body.as_str()).expect("Json format expected");
    debug!("The json value is {}", value);

    let after = value["data"]["after"].as_str();

    let posts = value["data"]["children"].as_array().unwrap();
    let posts: Vec<PostInfo> = posts
        .iter()
        .map(|post| PostInfo {
            id: String::from(post["data"]["id"].as_str().unwrap()),
            upvotes: post["data"]["ups"].as_u64().unwrap(),
            name: String::from(post["data"]["name"].as_str().unwrap()),
            subreddit: String::from(post["data"]["subreddit"].as_str().unwrap()),
        })
        .collect();
    PostInfos {
        after: after.map(|val| String::from(val)),
        posts,
    }
}
