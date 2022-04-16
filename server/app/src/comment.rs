use log::{debug, info};
use crate::OAUTH_REDDIT_URL;
use serde_derive::{Serialize, Deserialize};
use serde_json::Value;
use crate::reddit_client::{AuthRedditClient, DeleteRequest};
use crate::user::User;

#[derive(Serialize, Deserialize, Debug)]
struct PostComment {
    return_rtjson: bool,
    text: String,
    thing_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Comment {
    id: String,
    name: String,
    body: String,
    upvotes: u64,
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

pub async fn delete_all_comments(user: &User, client: &AuthRedditClient) {
    let comments = retrieve_comments(user, client).await;
    info!("Comments {:?}", comments);

    for comment in comments {
        let delete_request = DeleteRequest::new_json(comment.name.as_str());
        client.delete(&delete_request).await;
    }
}



async fn retrieve_comments(user: &User, client: &AuthRedditClient) -> Vec<Comment> {
    let url = format!("{}{}comments?limit=1000", OAUTH_REDDIT_URL, user.url.as_str());
    let body = client.get(url.as_str()).await;

    let value: Value = serde_json::from_str(body.as_str()).expect("Json format expected");
    debug!("The json value is {}", value);
    let comments = value["data"]["children"].as_array().unwrap();

    let comments: Vec<Comment> = comments.iter().map(
        |comment| {
            Comment {
                id: String::from(comment["data"]["id"].as_str().unwrap()),
                name: String::from(comment["data"]["name"].as_str().unwrap()),
                body: String::from(comment["data"]["body"].as_str().unwrap()),
                upvotes: comment["data"]["ups"].as_u64().unwrap(),
            }
        }
    ).collect();
    comments
}