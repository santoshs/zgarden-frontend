use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::page::Page;

pub struct Book {
    // window: web_sys::Window,
    document: web_sys::Document,
    element: web_sys::Element,
    pages: Arc<Mutex<HashMap<String, Arc<Page>>>>,
}

unsafe impl Sync for Book {}
impl Book {
    pub fn new() -> Book {
        let window = web_sys::window().expect("Global window does not exist");
        let document = window.document().expect("Expecting a document on window");
        let element = document.get_element_by_id("main").unwrap();
        Book {
            document,
            pages: Arc::new(Mutex::new(HashMap::new())),
            element,
        }
    }

    pub fn add_home_page(&'static self, elem: web_sys::Element) {
        let url = "/".to_string();
        let page = Arc::new(Page {
            url: url.clone(),
            id: "page-0".to_string(),
            element: elem,
            document: self.document.clone(),
        });

        self.pages.lock().unwrap().insert(url.clone(), page.clone());
        page.setup_links(self);
    }

    pub async fn add_page(&'static self, url: String) {
        let page = match self.get_page(url.clone()) {
            Some(page) => page,
            None => {
                let id = Uuid::new_v4().to_string();
                let element = self.document.create_element("iv").unwrap();
                element.set_class_name("note");
                element.set_class_name("col-4");

                self.element.append_child(&element).unwrap();

                let page = Arc::new(Page {
                    url: url.clone(),
                    element: element.clone(),
                    id,
                    document: self.document.clone(),
                });

                self.pages.lock().unwrap().insert(url, page.clone());
                page.init(self).await;
                page
            }
        };

        page.element.scroll_into_view();
    }

    pub fn get_page(&self, url: String) -> Option<Arc<Page>> {
        match self.pages.lock() {
            Ok(p) => match p.get(&url.clone()) {
                Some(page) => Some(page.clone()),
                None => None,
            },
            Err(e) => {
                crate::console_log(&e.to_string());
                None
            }
        }
    }
}
