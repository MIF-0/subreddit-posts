use crate::reddit_client::{AuthRedditClient, DeleteRequest};
use crate::user::User;
use crate::OAUTH_REDDIT_URL;
use log::{debug, info};
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
struct PostComment {
    return_rtjson: bool,
    text: String,
    thing_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Comment {
    pub id: String,
    pub name: String,
    pub body: String,
    pub upvotes: u64,
    pub subreddit: String,
}

pub async fn submit_comment(client: &AuthRedditClient, comment: String, post_id: String) {
    let post_id = format!("t3_{}", post_id);
    info!("Post id {:?}", post_id.as_str());

    /*    let url = format!("{}/by_id/{}", OAUTH_REDDIT_URL, post_id);
    let result = add_headers(client.get(url)
        .bearer_auth(token))
        .send()
        .await;

    let body = result
        .expect("Result is empty")
        .text().await
        .expect("Body is empty");

    info!("Result get listing body is {:?}", body);*/

    let comment = PostComment {
        return_rtjson: false,
        text: comment,
        thing_id: post_id,
    };

    let url = format!("{}/api/comment", OAUTH_REDDIT_URL);
    client.post(url.as_str(), Some(&comment)).await;
}

pub async fn delete_all_comments(client: &AuthRedditClient, user: &User) {
    let comments = retrieve_comments(client, user).await;
    info!("Comments {:?}", comments);

    for comment in comments {
        let delete_request = DeleteRequest::new_json(comment.name.as_str());
        client.delete(&delete_request).await;
    }
}

async fn retrieve_comments(client: &AuthRedditClient, user: &User) -> Vec<Comment> {
    let url = format!(
        "{}{}comments?limit=1000",
        OAUTH_REDDIT_URL,
        user.url.as_str()
    );
    let body = client.get(url.as_str()).await;

    let value: Value = serde_json::from_str(body.as_str()).expect("Json format expected");
    debug!("The json value is {}", value);
    let comments = value["data"]["children"].as_array().unwrap();

    let comments: Vec<Comment> = comments
        .iter()
        .map(|comment| Comment {
            id: String::from(comment["data"]["id"].as_str().unwrap()),
            name: String::from(comment["data"]["name"].as_str().unwrap()),
            body: String::from(comment["data"]["body"].as_str().unwrap()),
            upvotes: comment["data"]["ups"].as_u64().unwrap(),
            subreddit: String::from(comment["data"]["subreddit"].as_str().unwrap()),
        })
        .collect();
    comments
}

pub async fn retrieve_all_with(
    client: &AuthRedditClient,
    user: &User,
    filter: impl Fn(&Comment) -> bool,
) -> Vec<Comment> {
    let mut after: Option<String> = None;
    let mut result: Vec<Comment> = Vec::new();
    loop {
        let (comments, new_after) = retrieve_max(client, user, after.clone()).await;
        info!("Retrieved {} comments", comments.len());

        after = new_after;
        if comments.len().eq(&0) {
            break;
        }
        for comment in comments {
            if after.is_some() {
                let value = after.clone().unwrap();
                if value.eq(&comment.name) {
                    continue;
                }
            }
            if filter(&comment) {
                result.push(comment);
            }
        }

        if after.is_none() {
            break;
        }
    }
    return result;
}

async fn retrieve_max(
    client: &AuthRedditClient,
    user: &User,
    after: Option<String>,
) -> (Vec<Comment>, Option<String>) {
    let mut url = format!(
        "{}{}comments?limit=1000",
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
    let comments = value["data"]["children"].as_array().unwrap();

    let result = comments
        .iter()
        .map(|comment| Comment {
            id: String::from(comment["data"]["id"].as_str().unwrap()),
            name: String::from(comment["data"]["name"].as_str().unwrap()),
            body: String::from(comment["data"]["body"].as_str().unwrap()),
            upvotes: comment["data"]["ups"].as_u64().unwrap(),
            subreddit: String::from(comment["data"]["subreddit"].as_str().unwrap()),
        })
        .collect();

    (result, after.map(|val| String::from(val)))
}
