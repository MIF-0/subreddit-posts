use std::collections::HashMap;
use log::info;
use serde_derive::{Serialize, Deserialize};
use crate::REDDIT_URL;

#[derive(Serialize, Deserialize, Debug)]
struct ImageData {
    pub filepath: String,
    pub mimetype: String,
}


#[derive(Serialize, Deserialize, Debug)]
struct MediaResponse {
    asset: Option<String>,
    args: Vec<MediaArgs>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MediaArgs {
    action: String,
    fields: Vec<MediaArgs>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Field {
    name: String,
    value: String
}

pub async fn upload_media(mime_prefix: &str, media_path: &str) -> (String, Option<String>) {
    let mime_types = HashMap::from([
        ("png", "image/png"),
        ("mov", "video/quicktime"),
        ("mp4", "video/mp4"),
        ("jpg", "image/jpeg"),
        ("jpeg", "image/jpeg"),
        ("gif", "image/gif"),
    ]);
    let parts = media_path.split(".");
    let file_extension = parts.last().expect("No extension found");
    let mime_type = *mime_types.get(file_extension).expect(&format!("Can't find mimy type for extension {}", file_extension));
    if !mime_type.starts_with(mime_prefix) {
        panic!("Wrong file extension {} with expected mime type {}", file_extension, mime_prefix)
    }
    let image_data = ImageData {
        filepath: String::from(media_path),
        mimetype: String::from(mime_type),
    };

    let url = format!("{}/api/media/asset.json", REDDIT_URL);
    let client = reqwest::Client::builder()
        .build()
        .expect("error during client build");
    let result = client.post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept-Language", "en-us")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/77.0.3865.90 Safari/537.36")
        .body(serde_urlencoded::to_string(&image_data).expect("serialize issue during obtain auth token"))
        .send()
        .await;
    let body = result
        .expect("Result is empty")
        .text().await
        .expect("Body is empty");
    info!("Result body is {:?}", body);

/*    let upload_media_response = serde_json::from_str(&body)
        .expect("Bad body spec");*/

    (String::from(media_path), Some(String::from(media_path)))
}


