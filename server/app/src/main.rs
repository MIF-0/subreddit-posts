use std::{fs, io};
use actix_web::{App, HttpResponse, HttpServer, Responder};
use actix_web::web::{Data, Query};
use log::info;
use reqwest::Client;
use serde_derive::Deserialize;
use subreddit_posts_logic::comment::delete_all_comments;
use subreddit_posts_logic::data_store::DataStore;
use subreddit_posts_logic::environment::Environment;
use subreddit_posts_logic::flairs::retrieve_flairs_for;
use subreddit_posts_logic::in_memory_data_store::InMemoryDataStore;
use subreddit_posts_logic::login::{auth_token_for, request_login};
use subreddit_posts_logic::post::{post, Posts};
use subreddit_posts_logic::reddit_client::AuthRedditClient;
use subreddit_posts_logic::user;
use subreddit_posts_logic::user::info;

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv::from_filename("server/.env")
        .or_else(|_| dotenv::from_filename(".env"))
        .expect(".env file not found");
    env_logger::init();
    info!("Starting server");
    let env = Environment::read_env();
    info!("Env {:?}", env);

    let data = Data::new(InMemoryDataStore::new());

    HttpServer::new(move || App::new()
        .app_data(data.clone())
        .service(hello)
        .service(login)
        .service(login_callback)
        .service(upload)
        .service(flairs)
        .service(delete_comments)
    )
        .bind("127.0.0.1:9090")?
        .run()
        .await
}

#[actix_web::get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

#[actix_web::get("/reddit/login")]
async fn login(data: Data<InMemoryDataStore>) -> impl Responder {
    let settings = Environment::read_env();
    info!("Calling login");
    let login_request_id = request_login(settings).await;
    data.store_login_request_id(login_request_id);

    HttpResponse::Ok().body("Request sent")
}

#[actix_web::get("/reddit/login-callback")]
async fn login_callback((params, data): (Query<Params>, Data<InMemoryDataStore>)) -> impl Responder {
    info!("I was called");
    info!("params {:?}", params);

    let login_request_id = params.state.as_ref().expect("expect state as field, but it doesn't exist");
    if data
        .retrieve_login_request_id()
        .ne(login_request_id.as_str()) {
        panic!("Login request id are not same!!!")
    }
    let env = Environment::read_env();
    let token = auth_token_for(&params.code.clone().expect("Expect Code field, but it doesn't exist"), env).await;
    data.store_auth_token(token.access_token.clone());
    HttpResponse::Ok().body("Ok")
}

#[actix_web::get("/reddit/post")]
async fn upload(data: Data<InMemoryDataStore>) -> impl Responder {
    let content = fs::read_to_string("server/.posts")
        .or_else(|_| fs::read_to_string(".posts"))
        .expect("Something went wrong reading the file with posts");

    let auth_token = data.retrieve_auth_token();
    let posts: Posts = serde_json::from_str(&content).expect("JSON was not well-formatted");
    let client = AuthRedditClient::new(auth_token);
    post(posts, &client).await;

    try_post(".posts.possible", &client).await;
    try_post(".posts.promo_links", &client).await;

    HttpResponse::Ok().body("Uploaded")
}

#[actix_web::get("/reddit/comments/delete")]
async fn delete_comments(data: Data<InMemoryDataStore>) -> impl Responder {
    info!("Deleting all comments");

    let auth_token = data.retrieve_auth_token();
    let client = AuthRedditClient::new(auth_token);

    let user = user::info(&client).await;
    info!("user {:?}", user);

    delete_all_comments(&user, &client).await;

    info!("Deleted all comments");

    HttpResponse::Ok().body("deleted")
}

async fn try_post(file_name: &str, client: &AuthRedditClient) {
    let content = fs::read_to_string(format!("server/{}", file_name))
        .or_else(|_| fs::read_to_string(file_name));
    if content.is_ok() {
        let content = content.unwrap();
        let posts: Posts = serde_json::from_str(&content).expect("JSON was not well-formatted");
        post(posts, client).await;
    }
}

#[actix_web::get("/reddit/flairs")]
async fn flairs(data: Data<InMemoryDataStore>) -> impl Responder {
    let content = fs::read_to_string("server/.subreddits")
        .or_else(|_| fs::read_to_string(".subreddits"))
        .expect("Something went wrong reading the file with posts");


    let client = AuthRedditClient::new(data.retrieve_auth_token());
    let flair_info = retrieve_flairs_for(content.split(", ").collect(),
                                         &client,
    ).await;

    info!("Retrieved flairs: {:?}", flair_info);

    HttpResponse::Ok().body("retrieved")
}

#[derive(Debug, Deserialize)]
pub struct Params {
    error: Option<String>,
    code: Option<String>,
    state: Option<String>,
}
