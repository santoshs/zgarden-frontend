use anyhow::{anyhow, Error};
use reqwest::{self, header, header::CONTENT_TYPE};
use serde::Deserialize;
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;

use crate::get_book;
use crate::typeset_math;
use crate::utils::parse_url;

#[derive(Clone)]
pub struct Page {
    pub url: String,
    pub element: web_sys::Element,
    pub id: String,
    pub document: web_sys::Document,
}

#[derive(Deserialize)]
struct NoteNode {
    title: String,
    content: String,
}

impl Page {
    pub async fn init(&self) -> Result<(), Error> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        crate::log("page initialization");

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        let res = client.get(self.url.clone()).send().await?;
        if res.status().is_client_error() || res.status().is_server_error() {
            return Err(anyhow!(res
                .status()
                .canonical_reason()
                .or(Some("Note not found"))
                .unwrap()));
        }
        let note = res.json::<NoteNode>().await?;
        let title = self.document.create_element("h1").unwrap();
        title.class_list().add_1("title").unwrap();
        title.set_inner_html(&note.title);
        self.element.append_child(&title).unwrap();

        let content = self.document.create_element("div").unwrap();
        content.set_inner_html(&note.content);
        self.element.append_child(&content).unwrap();
        typeset_math();

        self.setup_links().await;

        Ok(())
    }

    pub async fn setup_links(&self) {
        let book = get_book().await;
        let links = self
            .element
            .query_selector_all("a")
            .unwrap()
            .dyn_into::<web_sys::NodeList>()
            .unwrap();

        crate::log(&format!("Num Links in page: {}", links.length()));

        for i in 0..links.length() {
            let c = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
                let target = e.target().unwrap().dyn_into::<web_sys::Element>().unwrap();
                let url = target.get_attribute("href").unwrap();
                let location = book.window().location();
                let parsed_url = parse_url(&url, location);

                e.prevent_default();
                e.stop_immediate_propagation();
                e.stop_propagation();

                spawn_local(book.add_page(parsed_url.to_string(), target));
            }) as Box<dyn FnMut(_)>);

            let link = links
                .item(i)
                .unwrap()
                .dyn_into::<web_sys::HtmlElement>()
                .unwrap();

            if let Some(url) = link.get_attribute("href") {
                let window = web_sys::window().expect("Global window does not exist");
                let location = window.location();
                let u = parse_url(&url, location.clone());
                match book.get_page(u.to_string()).await {
                    Some(_) => {
                        link.class_list().add_1("visited").unwrap();
                    }
                    None => {
                        if crate::get_origin(u.clone()) != location.origin().unwrap() {
                            link.set_attribute("target", "_blank").unwrap_throw();
                            continue;
                        }
                    }
                };

                // If it is a url fragment, like footnotes, then skip
                if u.fragment().is_some() {
                    continue;
                }

                link.set_onclick(Some(c.as_ref().unchecked_ref()));
                c.forget();
            }
        }
    }
}
