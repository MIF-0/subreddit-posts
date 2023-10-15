use crate::OAUTH_REDDIT_URL;
use log::{debug, info};
use reqwest::{Client, RequestBuilder};
use serde::Serialize;
use serde_derive::{Deserialize, Serialize};
use std::{thread, time};

pub struct AuthRedditClient {
    client: Client,
    auth_token: String,
}

impl AuthRedditClient {
    pub fn new(auth_token: String) -> AuthRedditClient {
        let client = reqwest::Client::builder()
            .build()
            .expect("error during client build");

        AuthRedditClient { client, auth_token }
    }

    pub async fn get(&self, url: &str) -> String {
        let result = Self::add_headers(self.client.get(url).bearer_auth(self.auth_token.as_str()))
            .send()
            .await;
        debug!("Result body of GET {},  is {:?}", url, result);

        let body = result
            .expect("Result is empty")
            .text()
            .await
            .expect("Body is empty");
        debug!("Result body of GET {},  is {:?}", url, body);

        // limit 60 post in a second
        let ten_millis = time::Duration::from_millis(21);
        thread::sleep(ten_millis);

        return body;
    }

    pub async fn post<T: Serialize>(&self, url: &str, body: Option<T>) -> String {
        let post_request_builder =
            Self::add_headers(self.client.post(url).bearer_auth(self.auth_token.as_str()));
        let post_request_builder = match body {
            Some(value) => post_request_builder.body(
                serde_urlencoded::to_string(&value)
                    .expect("serialize issue during obtain auth token"),
            ),
            None => post_request_builder,
        };

        let result = post_request_builder.send().await;
        debug!("Result body of POST {},  is {:?}", url, result);

        let body = result
            .expect("Result is empty")
            .text()
            .await
            .expect("Body is empty");
        debug!("Result body of POST {},  is {:?}", url, body);

        // limit 60 post in a second
        let sleep_time = time::Duration::from_millis(30_000);
        thread::sleep(sleep_time);

        return body;
    }

    pub async fn delete(&self, delete_request: &DeleteRequest) {
        let should_not_be_deleted1 = "u9px12"; //
        let should_not_be_deleted2 = "uaysta";
        let should_not_be_deleted3 = "uayzn9";
        let should_not_be_deleted4 = "uayoa0"; //
        let should_not_be_deleted5 = "uap9z2"; //
        let should_not_be_deleted6 = "uah6ce"; //
        let should_not_be_deleted7 = "uek777"; //
        if delete_request.id.contains(should_not_be_deleted1)
            || delete_request.id.contains(should_not_be_deleted2)
            || delete_request.id.contains(should_not_be_deleted3)
            || delete_request.id.contains(should_not_be_deleted4)
            || delete_request.id.contains(should_not_be_deleted5)
            || delete_request.id.contains(should_not_be_deleted6)
            || delete_request.id.contains(should_not_be_deleted7)
        {
            info!("will not delete");
            return;
        }
        let url = format!("{}/api/del", OAUTH_REDDIT_URL);
        let result = Self::add_headers(self.client.post(url).bearer_auth(self.auth_token.as_str()))
            .body(
                serde_urlencoded::to_string(&delete_request)
                    .expect("serialize issue during obtain auth token"),
            )
            .send()
            .await;

        debug!("Result of deletion is {:?}", result);

        let body = result
            .expect("Result is empty")
            .text()
            .await
            .expect("Body is empty");
        debug!("Result of deletion is {:?}", body);

        // limit 60 post in a second
        let ten_millis = time::Duration::from_millis(21);
        thread::sleep(ten_millis);
    }

    pub fn add_headers(builder: RequestBuilder) -> RequestBuilder {
        builder
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept-Language", "en-us")
            .header(
                "User-Agent",
                "rust: z8fkUNU-Wwaw-HBlQvjT1Q (by MIF) v 1.0.0",
            )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteRequest {
    pub id: String,
    pub api_type: String,
}

impl DeleteRequest {
    pub fn new_json(id: &str) -> DeleteRequest {
        DeleteRequest {
            id: String::from(id),
            api_type: String::from("json"),
        }
    }
}
