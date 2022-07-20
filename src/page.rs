use crate::{console_log, page_init};
use reqwest::{self, header, header::CONTENT_TYPE};
use serde::Deserialize;

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
    // alert(&res.text().await.unwrap());

    let window = web_sys::window().expect("Global window does not exist");
    let document = window.document().expect("Expecting a document on window");
    let page = document.get_element_by_id("main");
    match page {
        Some(p) => {
            let elem = document.create_element("div").unwrap();
            elem.set_class_name("note");
            elem.set_class_name("col-4");
            p.append_child(&elem).unwrap();

            #[derive(Deserialize)]
            struct NoteNode {
                title: String,
                content: String,
            }
            let note = res.json::<NoteNode>().await.unwrap();
            let title = document.create_element("h2").unwrap();
            title.set_inner_html(&note.title);
            elem.append_child(&title).unwrap();

            let content = document.create_element("div").unwrap();
            content.set_inner_html(&note.content);
            elem.append_child(&content).unwrap();
            //elem.set_inner_html(&note.content);

            page_init(elem);
        }
        None => console_log("Cannot get main element"),
    }
}
