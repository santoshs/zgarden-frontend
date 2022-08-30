use url::{ParseError, Url};

use crate::log;

pub fn parse_url(_url: &String, location: web_sys::Location) -> Url {
    let mut url = _url.to_string();

    loop {
        let parsed_url = Url::parse(&url);
        match parsed_url {
            Ok(u) => return u,
            Err(e) => match e {
                ParseError::RelativeUrlWithoutBase => {
                    url = location.origin().unwrap() + &url;
                }
                _ => {
                    log(&format!("{:?}", e));
                    panic!();
                }
            },
        };
    }
}
