use async_mutex::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
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

        self.insert_page(page.clone(), url, "page-0".to_string())
            .await;
        self.page_list.lock().await.push(page.clone());
        page.setup_links().await;
    }

    pub async fn add_page(&'static self, url: String, link_node: web_sys::Element) {
        crate::log(&format!("Adding page for {}", url));
        let page = match self.get_page(url.clone()).await {
            Some(page) => page,
            None => {
                let id = Uuid::new_v4().to_string();
                let element = self.document.create_element("div").unwrap();

                element.class_list().add_2("note", "col-4").unwrap();
                element.set_id(&id);

                let page = Arc::new(Page {
                    url: url.clone(),
                    element: element.clone(),
                    id: id.clone(),
                    document: self.document.clone(),
                });

                let init_result = page.init().await;
                if init_result.is_err() {
                    crate::alert::show_alert(
                        Some("Error"),
                        None,
                        &init_result.unwrap_err().to_string(),
                    );

                    return;
                }

                // Remove all pages to the right if origin page is in the middle
                let anchor: &web_sys::HtmlAnchorElement =
                    link_node.dyn_ref::<web_sys::HtmlAnchorElement>().unwrap();
                match anchor.closest(".note") {
                    Ok(elem) => match elem {
                        Some(e) => {
                            self.remove_pages_right_of(e.id()).await;
                        }
                        None => log("No parent found"),
                    },
                    Err(e) => log(&format!("{:?}", e)),
                }

                self.page_list.lock().await.push(page.clone());
                self.element.append_child(&element).unwrap();
                self.insert_page(page.clone(), url, id.clone()).await;

                page
            }
        };

        page.element.scroll_into_view();
        self.document.document_element().unwrap().set_scroll_top(0);
    }

    async fn insert_page(&self, page: Arc<Page>, url: String, id: String) {
        self.pages_by_url.lock().await.insert(url, page.clone());
        self.pages_by_id.lock().await.insert(id, page);
    }

    pub async fn get_page(&self, url: String) -> Option<Arc<Page>> {
        self.pages_by_url.lock().await.get(&url).cloned()
    }

    pub fn window(&self) -> web_sys::Window {
        self.window.clone()
    }

    pub fn document(&self) -> web_sys::Document {
        self.document.clone()
    }

    pub async fn search(&self, search_term: String) {
        if search_term.trim().len() < 3 {
            crate::alert::show_alert(
                Some("Search"),
                None,
                "More than three characters needed for searching",
            );
            return;
        }
        log(&format!("searching for .. {}", search_term));

        let mut results: HashMap<String, (String, usize)> = HashMap::new();
        for s in search_term.split(' ') {
            if let Some(urls) = self.search_index.get(&s.to_lowercase()) {
                for u in urls.iter() {
                    results
                        .entry(u.0.to_string())
                        .and_modify(|value| value.1 += 1)
                        .or_insert((u.1.clone(), 1));
                }
            }
        }

        self.show_search_results(search_term, results).await;
    }

    pub async fn show_search_results(
        &self,
        search_term: String,
        results: HashMap<String, (String, usize)>,
    ) {
        // Sort the results based on the number of hits per URL
        let mut sorted: Vec<_> = results.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));

        if sorted.is_empty() {
            crate::alert::show_alert(
                Some("Search"),
                None,
                &format!("No notes found for <i>{}</i>", search_term),
            );
            return;
        }

        let search_results = self.document.create_element("div").unwrap();
        let title = self.document.create_element("h1").unwrap();
        title.set_inner_html(&format!("Notes containing <i>{}</i>", search_term));
        search_results.append_child(&title).unwrap();

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

        let pages = self.pages_by_id.lock().await;
        let home = pages.get("page-0").unwrap();
        home.element.set_text_content(None);
        home.element.append_child(&search_results).unwrap();
        home.setup_links().await;

        // pop rest of the stack
    }

    pub async fn remove_page(&self, page: &Page) {
        page.element.remove();
        self.pages_by_id.lock().await.remove(&page.id);
        self.pages_by_url.lock().await.remove(&page.url);
    }

    async fn remove_pages_right_of(&self, page_id: String) {
        let mut page_list = self.page_list.lock().await;
        let len = page_list.len();
        let start_index = page_list.iter().position(|x| x.id == page_id).unwrap() + 1;
        if start_index >= page_list.len() {
            return;
        }
        for p in page_list.iter().skip(start_index) {
            self.remove_page(p).await;
        }
        page_list.drain(start_index..len);
    }
}
