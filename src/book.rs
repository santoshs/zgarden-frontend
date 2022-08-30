use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use wasm_bindgen::JsCast;

use crate::{log, page::Page};

pub struct Book {
    window: web_sys::Window,
    document: web_sys::Document,
    element: web_sys::Element,
    pages_by_url: Arc<Mutex<HashMap<String, Arc<Page>>>>,
    pages_by_id: Arc<Mutex<HashMap<String, Arc<Page>>>>,
    page_list: Mutex<Vec<Arc<Page>>>,
    search_index: HashMap<String, HashSet<(String, String)>>,
}

unsafe impl Sync for Book {}
impl Book {
    pub async fn new() -> Book {
        crate::log("Creating new book");
        let window = web_sys::window().expect("Global window does not exist");
        let document = window.document().expect("Expecting a document on window");
        let element = document.get_element_by_id("main").unwrap();
        Book {
            window: window.clone(),
            document,
            pages_by_url: Arc::new(Mutex::new(HashMap::new())),
            pages_by_id: Arc::new(Mutex::new(HashMap::new())),
            element,
            page_list: Mutex::new(Vec::new()),
            search_index: crate::get_search_index(window).await,
        }
    }

    pub async fn add_home_page(&'static self, elem: web_sys::Element) {
        crate::log("Adding home page");
        let url = "/".to_string();
        let page = Arc::new(Page {
            url: url.clone(),
            id: "page-0".to_string(),
            element: elem,
            document: self.document.clone(),
        });

        self.insert_page(page.clone(), url, "page-0".to_string());
        self.page_list.lock().unwrap().push(page.clone());
        page.setup_links().await;

        log(&format!(
            "Total number of search items: {}",
            self.search_index.len()
        ));
    }

    pub async fn add_page(&'static self, url: String, link_node: web_sys::Element) {
        crate::log(&format!("Adding page for {}", url));
        let page = match self.get_page(url.clone()) {
            Some(page) => page,
            None => {
                let id = Uuid::new_v4().to_string();
                let element = self.document.create_element("div").unwrap();

                element.class_list().add_2("note", "col-4").unwrap();
                element.set_id(&id);

                let anchor: &web_sys::HtmlAnchorElement =
                    link_node.dyn_ref::<web_sys::HtmlAnchorElement>().unwrap();
                match anchor.closest(".note") {
                    Ok(elem) => match elem {
                        Some(e) => {
                            log(&e.id());
                            let mut page_list = self.page_list.lock().unwrap();
                            let len = page_list.len();
                            let parent_index =
                                page_list.iter().position(|x| x.id == e.id()).unwrap() + 1;
                            if parent_index < page_list.len() {
                                for p in page_list.iter().skip(parent_index) {
                                    p.element.remove();
                                    self.pages_by_id.lock().unwrap().remove(&p.id);
                                    self.pages_by_url.lock().unwrap().remove(&p.url);
                                }
                            }
                            page_list.drain(parent_index..len);
                        }
                        None => log("No parent found"),
                    },
                    Err(e) => log(&format!("{:?}", e)),
                }

                let page = Arc::new(Page {
                    url: url.clone(),
                    element: element.clone(),
                    id: id.clone(),
                    document: self.document.clone(),
                });

                log(&format!(
                    "number of page in list: {}",
                    self.page_list.lock().unwrap().len()
                ));
                self.page_list.lock().unwrap().push(page.clone());
                self.element.append_child(&element).unwrap();

                self.insert_page(page.clone(), url, id.clone());
                page.init().await;
                page
            }
        };

        page.element.scroll_into_view();
        self.document.document_element().unwrap().set_scroll_top(0);
    }

    fn insert_page(&self, page: Arc<Page>, url: String, id: String) {
        self.pages_by_url.lock().unwrap().insert(url, page.clone());
        self.pages_by_id.lock().unwrap().insert(id, page.clone());
    }

    pub fn get_page(&self, url: String) -> Option<Arc<Page>> {
        match self.pages_by_url.lock() {
            Ok(p) => match p.get(&url) {
                Some(page) => Some(page.clone()),
                None => None,
            },
            Err(e) => {
                crate::console_log(&e.to_string());
                None
            }
        }
    }

    pub fn window(&self) -> web_sys::Window {
        self.window.clone()
    }

    pub fn search(&self, search_term: String) {
        if search_term.trim().len() < 3 {
            // Show a alert
            return;
        }
        log(&format!("searching for .. {}", search_term));

        let mut results: HashMap<String, (String, usize)> = HashMap::new();
        for s in search_term.split(" ") {
            if let Some(urls) = self.search_index.get(&s.to_lowercase()) {
                for u in urls.iter() {
                    results
                        .entry(u.0.to_string())
                        .and_modify(|value| value.1 += 1)
                        .or_insert((u.1.clone(), 1));
                }
            }
        }

        log(&format!("Found {} results", results.len()));
        self.show_search_results(search_term, results);
    }

    pub fn show_search_results(
        &self,
        search_term: String,
        results: HashMap<String, (String, usize)>,
    ) {
        let search_modal = self.document.get_element_by_id("search-modal").unwrap();
        let search_results = search_modal
            .query_selector("#search-results")
            .unwrap()
            .unwrap();
        search_modal
            .query_selector("#search-title")
            .unwrap()
            .unwrap()
            .set_text_content(Some(&format!(
                "{} notes found for {}",
                results.len(),
                search_term
            )));

        // Sort the results based on the number of hits per URL
        let mut sorted: Vec<_> = results.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        search_modal.class_list().add_1("active").unwrap();

        if sorted.is_empty() {
            search_results.set_text_content(Some("No results found"));
            return;
        }

        for s in sorted {
            let result_element = self.document.create_element("div").unwrap();
            let link_element = self
                .document
                .create_element("a")
                .unwrap()
                .dyn_into::<web_sys::HtmlAnchorElement>()
                .unwrap();
            link_element.set_href(s.0);
            link_element.set_inner_html(&s.1 .0);
            result_element.append_child(&link_element).unwrap();
            search_results.append_child(&result_element).unwrap();
        }
    }
}
