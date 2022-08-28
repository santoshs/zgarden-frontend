use std::panic;
use url::Url;
use wasm_bindgen::prelude::*;

#[macro_use]
extern crate lazy_static;

mod book;
mod page;

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

lazy_static! {
    static ref BOOK: book::Book = Book::new();
}

#[wasm_bindgen]
pub async fn setup() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    crate::log("wasm setup");
    let window = web_sys::window().expect("Global window does not exist");
    let document = window.document().expect("Expecting a document on window");
    let page = document.get_element_by_id("page-0");
    match page {
        Some(p) => {
            crate::log("adding home page");
            BOOK.add_home_page(p);
        }
        None => log("Cannot get element with id page-0"),
    }
}

fn get_origin(u: Url) -> String {
    let mut url = Url::parse(&u[..url::Position::BeforePath]).unwrap();

    match url.set_host(Some(u.host_str().unwrap())) {
        Ok(()) => {}
        Err(e) => log(&e.to_string()),
    }
    url.set_scheme(u.scheme()).unwrap();
    if let Some(p) = u.port() {
        url.set_port(Some(p)).unwrap();
    }

    let mut chars = url.as_str().chars();
    chars.next_back();
    return chars.as_str().to_string();
}
