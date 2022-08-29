use reqwest::{self, header, header::CONTENT_TYPE};
use serde::Deserialize;
use url::{ParseError, Url};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;

use crate::get_book;
use crate::{console_log, log, typeset_math};

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
    pub async fn init(&self) {
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
        let res = client.get(self.url.clone()).send().await;
        match res {
            Ok(r) => {
                let note = match r.json::<NoteNode>().await {
                    Ok(n) => n,
                    Err(e) => {
                        log(&e.to_string());
                        return;
                    }
                };
                let title = self.document.create_element("h2").unwrap();
                title.set_inner_html(&note.title);
                self.element.append_child(&title).unwrap();

                let content = self.document.create_element("div").unwrap();
                content.set_inner_html(&note.content);
                self.element.append_child(&content).unwrap();
                typeset_math();

                self.setup_links();
            }
            Err(e) => log(&e.to_string()),
        }
    }

    pub fn setup_links(&self) {
        let book = get_book();
        let links = self
            .element
            .query_selector_all("a")
            .unwrap()
            .dyn_into::<web_sys::NodeList>()
            .unwrap();

        crate::log(&format!("Num Links in page: {}", links.length()));

        for i in 0..links.length() {
            let link = links.item(i).clone();
            let c = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
                let target = e.target().unwrap().dyn_into::<web_sys::Element>().unwrap();
                let mut url = target.get_attribute("href").unwrap();
                let location = book.window().location();

                loop {
                    let parsed_url = Url::parse(&url);
                    match parsed_url {
                        Ok(_) => break,
                        Err(e) => match e {
                            ParseError::RelativeUrlWithoutBase => {
                                url = location.origin().unwrap() + &url;
                            }
                            _ => console_log(&format!("{:?}", e)),
                        },
                    };
                }

                e.prevent_default();
                e.stop_immediate_propagation();
                e.stop_propagation();

                spawn_local(book.add_page(url, link.clone()));
            }) as Box<dyn FnMut(_)>);

            let link = links
                .item(i)
                .unwrap()
                .dyn_into::<web_sys::HtmlElement>()
                .unwrap();

            let href = link.get_attribute("href");
            match href {
                Some(x) => {
                    let window = web_sys::window().expect("Global window does not exist");
                    let location = window.location();
                    let mut url = x;
                    loop {
                        let parsed_url = Url::parse(&url);
                        match parsed_url {
                            Ok(u) => {
                                match book.get_page(u.to_string()) {
                                    Some(_) => {
                                        link.class_list().add_1("visited").unwrap();
                                    }
                                    None => {
                                        if crate::get_origin(u.clone())
                                            != location.origin().unwrap()
                                        {
                                            link.set_attribute("target", "_blank").unwrap_throw();
                                            break;
                                        }
                                    }
                                };
                                // If it is a url fragment, like footnotes, then skip
                                if let Some(_) = u.fragment() {
                                    break;
                                }

                                link.set_onclick(Some(c.as_ref().unchecked_ref()));
                                c.forget();
                                break;
                            }
                            Err(e) => match e {
                                ParseError::RelativeUrlWithoutBase => {
                                    url = location.origin().unwrap() + &url;
                                }
                                _ => console_log(&format!("{:?}", e)),
                            },
                        };
                    }
                }
                None => {}
            }
        }
    }
}
