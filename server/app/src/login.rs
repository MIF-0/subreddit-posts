use std::collections::HashMap;
use log::info;
use string_template::Template;
use uuid::Uuid;
use webbrowser;
use crate::environment::Environment;
use serde_derive::{Serialize, Deserialize};
use crate::AuthToken;

//https://github.com/reddit-archive/reddit/wiki/OAuth2

const REQUEST_LOGIN_URL: &str = "https://www.reddit.com/api/v1/authorize?client_id={{APP_ID}}&response_type=code&state={{LOGIN_REQUEST_ID}}&redirect_uri={{APP_REDIRECT_URL}}&duration=temporary&scope={{APP_SCOPE}}";

pub async fn request_login(settings: Environment) -> String {
    let template_login = Template::new(REQUEST_LOGIN_URL);

    let login_request_id = Uuid::new_v4();
    let login_request_id = login_request_id.to_string();

    let mut args = HashMap::new();
    args.insert("APP_ID", settings.application_id.as_str());
    args.insert("LOGIN_REQUEST_ID", login_request_id.as_str());
    args.insert("APP_REDIRECT_URL", settings.application_redirection_link.as_str());
    args.insert("APP_SCOPE", settings.application_scope.as_str());

    let url = template_login.render(&args);
    info!("Opening browser...");
    webbrowser::open(&url).expect("failed to open URL");

    login_request_id
}

pub async fn auth_token_for(code: &str, settings: Environment) -> AuthToken {
    info!("Trying to receive token for code {}", code);
    let client = reqwest::Client::builder()
        .build()
        .expect("error during client build");

    // This will POST a body of `{"lang":"rust","body":"json"}`
    let data = NetworkData {
        grant_type: String::from("authorization_code"),
        code: String::from(code),
        redirect_uri: settings.application_redirection_link.clone(),

    };

    let result = client.post("https://www.reddit.com/api/v1/access_token")
        .basic_auth(settings.application_id.as_str(), Some(settings.application_secret.as_str()))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept-Language", "en-us")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3865.90 Safari/537.36")
        .body(serde_urlencoded::to_string(&data).expect("serialize issue during obtain auth token"))
        .send()
        .await;

    let body = result
        .expect("Result is empty")
        .text().await
        .expect("Body is empty");
    info!("Result body is {:?}", body);

    serde_json::from_str(&body)
        .expect("Bad body spec")
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkData {
    grant_type: String,
    code: String,
    redirect_uri: String,
}

