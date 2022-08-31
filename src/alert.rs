use wasm_bindgen::{closure::Closure, JsCast};

// pub fn str_to_jsarray(strings: Vec<&str>) -> Array {
//     let arr = Array::new();
//     for s in strings {
//         let js = JsValue::from_str(s);
//         arr.push(&js);
//     }
//     arr
// }

fn create_toast(
    document: &web_sys::Document,
    title_text: Option<&str>,
    message: &str,
) -> web_sys::Element {
    // The main toast
    let toast = document.create_element("div").unwrap();
    // toast.set_attribute("role", "alert").unwrap();
    // toast.set_attribute("aria-live", "assertive").unwrap();
    // toast.set_attribute("aria-atomic", "true").unwrap();
    toast.set_class_name("toast");

    // toast header
    let toast_header = document.create_element("div").unwrap();
    toast_header.set_class_name("toast-header");
    toast.append_child(&toast_header).unwrap();

    // Title and subtitle
    let title = document.create_element("strong").unwrap();
    title.set_class_name("me-auto");
    title.set_text_content(title_text);

    let sub_title = document.create_element("small").unwrap();
    sub_title.set_text_content(None);

    // The close button in the header
    let close = document.create_element("button").unwrap();
    close.class_list().add_1("btn-close").unwrap();
    close.set_attribute("type", "button").unwrap();

    toast_header.append_child(&title).unwrap();
    toast_header.append_child(&sub_title).unwrap();
    toast_header.append_child(&close).unwrap();

    let toast_body = document.create_element("div").unwrap();
    toast_body.set_class_name("toast-body");
    toast_body.set_inner_html(&message);
    toast.append_child(&toast_body).unwrap();

    let toast_1 = toast.clone();
    let window = web_sys::window().expect("Global window does not exist");
    let closure: Closure<dyn FnMut()> = Closure::new(move || {
        toast_1.remove();
    });
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            5000,
        )
        .unwrap();

    close
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap()
        .set_onclick(Some(closure.as_ref().unchecked_ref()));
    closure.forget();

    toast
}

pub fn show_alert(title: Option<&str>, message: &str) {
    let window = web_sys::window().expect("Global window does not exist");
    let document = window.document().expect("Expecting a document on window");

    if let Some(alert_container) = document.get_element_by_id("alert-container") {
        let toast = create_toast(&document, title, message);
        alert_container
            .insert_before(&toast, alert_container.first_child().as_ref())
            .unwrap();

        toast.class_list().add_2("fade", "show").unwrap();
        toast.class_list().remove_1("hide").unwrap();
    } else {
        crate::log("No #alert-container found");
    }
}
