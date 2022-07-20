use std::panic;
use url::{ParseError, Url};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::Element;

mod page;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn console_log(s: &str);
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

macro_rules! _log {
    ($($t:tt)*) => (crate::console_log(&format_args!($($t)*).to_string()))
}

pub(crate) use _log;

pub fn log(s: &str) {
    _log!("{}", s);
}

#[wasm_bindgen]
pub fn setup() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let window = web_sys::window().expect("Global window does not exist");
    let document = window.document().expect("Expecting a document on window");
    let page = document.get_element_by_id("main");
    match page {
        Some(p) => {
            page_init(p);
        }
        None => log("Cannot get element with class page"),
    }
}

pub fn page_init(page: Element) {
    let links = page
        .query_selector_all("a")
        .unwrap()
        .dyn_into::<web_sys::NodeList>()
        .unwrap();
    setup_links(links);
}

pub fn setup_links(links: web_sys::NodeList) {
    log(&format!("Num Links in page: {}", links.length()));

    for i in 0..links.length() {
        let c = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            let target = e.target().unwrap().dyn_into::<web_sys::Element>().unwrap();
            let mut url = target.get_attribute("href").unwrap();
            let window = web_sys::window().expect("Global window does not exist");

            let location = window.location();
            loop {
                let parsed_url = Url::parse(&url);
                match parsed_url {
                    Ok(u) => {
                        console_log(u.as_str());
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

            e.prevent_default();
            e.stop_immediate_propagation();
            e.stop_propagation();

            spawn_local(page::add_page(url + "/index.json"));
        }) as Box<dyn FnMut(_)>);

        log("adding click event");
        links
            .item(i)
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap()
            .set_onclick(Some(c.as_ref().unchecked_ref()));
        c.forget();
    }
}
