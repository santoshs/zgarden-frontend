use async_once::AsyncOnce;
use lazy_static::lazy_static;
use reqwest::{self, header, header::CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::panic;
use url::Url;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlFormElement;

extern crate lazy_static;

mod book;
mod page;
mod utils;

use book::Book;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn console_log(s: &str);
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(module = "/helper.js", js_name = "typesetMath")]
    pub fn typeset_math();
}

macro_rules! _log {
    ($($t:tt)*) => (crate::console_log(&format_args!($($t)*).to_string()))
}

pub(crate) use _log;

pub fn log(s: &str) {
    _log!("{}", s);
}

async fn get_book() -> &'static Book {
    log("Getting book");
    lazy_static! {
        static ref BOOK: AsyncOnce<Book> = AsyncOnce::new(Book::new());
    }

    BOOK.get().await
}

#[wasm_bindgen]
pub async fn setup() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    crate::log("wasm setup");
    let window = web_sys::window().expect("Global window does not exist");
    let document = window.document().expect("Expecting a document on window");
    let page = document.get_element_by_id("page-0");

    setup_search(document, get_book().await);
    match page {
        Some(p) => {
            get_book().await.add_home_page(p).await;
        }
        None => log("Cannot get element with id page-0"),
    }
}

fn setup_search(document: web_sys::Document, book: &'static Book) {
    let form: HtmlFormElement = document
        .get_element_by_id("search-form")
        .unwrap()
        .dyn_into::<web_sys::HtmlFormElement>()
        .unwrap();

    let c = Closure::wrap(Box::new(move |e: web_sys::Event| {
        let form = e
            .target()
            .unwrap()
            .dyn_into::<web_sys::HtmlFormElement>()
            .unwrap();

        let form_data = web_sys::FormData::new_with_form(&form).unwrap();
        let search_term = form_data.get("search-input").as_string().unwrap();

        spawn_local(book.search(search_term));
        e.prevent_default();
    }) as Box<dyn FnMut(_)>);

    form.set_onsubmit(Some(c.as_ref().unchecked_ref()));
    c.forget();
}

fn get_origin(u: Url) -> String {
    let mut url = Url::parse(&u[..url::Position::BeforePath]).unwrap();

    if let Some(host_str) = u.host_str() {
        match url.set_host(Some(host_str)) {
            Ok(()) => {}
            Err(e) => log(&e.to_string()),
        }
        url.set_scheme(u.scheme()).unwrap();
        if let Some(p) = u.port() {
            url.set_port(Some(p)).unwrap();
        }

        let mut chars = url.as_str().chars();
        chars.next_back();
        chars.as_str().to_string()
    } else {
        log(&format!("Invalid URL: Invalid host string in URL: {}", u));
        u.to_string()
    }
}

// fn print_type_of<T>(_: &T) {
//     console_log(&format!("variable type is: {}", std::any::type_name::<T>()))
// }

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchData {
    title: String,
    content: String,
    id: String,
    url: String,
}

async fn get_search_index(window: web_sys::Window) -> HashMap<String, HashSet<(String, String)>> {
    let location = window.location();
    let url = location.origin().unwrap() + "/search/index.json";
    let mut headers = header::HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );

    log("Initialising search");

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();
    let search_db = match client.get(url.clone()).send().await {
        Ok(resp) => match resp.json::<Vec<SearchData>>().await {
            Ok(n) => n,
            Err(e) => {
                log(&e.to_string());
                Vec::new()
            }
        },
        Err(e) => {
            log(&e.to_string());
            Vec::new()
        }
    };

    let mut index: HashMap<String, HashSet<(String, String)>> = HashMap::new();
    for s in search_db {
        let text = s.title.clone() + " " + &s.content;

        for k in text.split(' ') {
            let v = index.entry(k.to_string()).or_default();

            v.insert((s.url.to_string(), s.title.to_string()));
        }
    }

    log(&format!("Number of search index entries: {}", index.len()));
    index
}
