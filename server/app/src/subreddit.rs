use crate::reddit_client::{AuthRedditClient, DeleteRequest};
use crate::user::User;
use crate::{comment, post};
use log::info;

pub async fn get_all_from(client: &AuthRedditClient, user: &User, sub_reddit: String) {
    let posts = post::retrieve_all_posts_with(client, user, |post| {
        post.subreddit
            .to_lowercase()
            .contains(&sub_reddit.to_lowercase())
    })
    .await;
    info!("Found {} posts", posts.len());
    for post in posts {
        info!("Found {:?} post", post);
    }

    let comments = comment::retrieve_all_with(client, user, |comment| {
        comment
            .subreddit
            .to_lowercase()
            .contains(&sub_reddit.to_lowercase())
    })
    .await;
    info!("Found {} comments", comments.len());
    for comment in comments {
        info!("Found {:?} comment", comment);
    }
}

pub async fn delete_all_from(client: &AuthRedditClient, user: &User, sub_reddit: String) {
    let posts = post::retrieve_all_posts_with(client, user, |post| {
        post.subreddit
            .to_lowercase()
            .contains(&sub_reddit.to_lowercase())
    })
    .await;
    info!("Found {} posts", posts.len());
    for post in posts {
        let delete_request = DeleteRequest::new_json(post.name.as_str());
        info!("Will delete post {:?}", post);
        client.delete(&delete_request).await;
    }

    let comments = comment::retrieve_all_with(client, user, |comment| {
        comment
            .subreddit
            .to_lowercase()
            .contains(&sub_reddit.to_lowercase())
    })
    .await;
    info!("Found {} comments", comments.len());
    for comment in comments {
        let delete_request = DeleteRequest::new_json(comment.name.as_str());
        info!("Will delete comment {:?}", comment);
        client.delete(&delete_request).await;
    }
}

pub async fn delete_comments_from(client: &AuthRedditClient, user: &User, sub_reddit: String) {
    let comments = comment::retrieve_all_with(client, user, |comment| {
        comment
            .subreddit
            .to_lowercase()
            .contains(&sub_reddit.to_lowercase())
    })
    .await;
    info!("Found {} comments", comments.len());
    for comment in comments {
        let delete_request = DeleteRequest::new_json(comment.name.as_str());
        info!("Will delete comment {:?}", comment);
        client.delete(&delete_request).await;
    }
}
