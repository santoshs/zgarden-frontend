use crate::alert;
use reqwest::{self, header, header::CONTENT_TYPE};

pub async fn add_page(url: String) {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();
    let res = client.get(url).send().await.unwrap();
    alert(&res.text().await.unwrap());
}
