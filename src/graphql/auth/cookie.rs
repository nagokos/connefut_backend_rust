use cookie::Cookie;
use hyper::{header::COOKIE, HeaderMap};

pub fn get_cookie_from_header(headers: &HeaderMap) -> Option<Vec<&str>> {
    match headers.get(COOKIE) {
        Some(v) => match v.to_str().ok() {
            Some(c) => {
                let cookie_vec: Vec<&str> = c.split(';').collect();
                Some(cookie_vec)
            }
            None => None,
        },
        None => None,
    }
}

pub fn get_value_from_cookie(cookies: Vec<&str>, want_cookie_name: &str) -> Option<String> {
    for c in cookies {
        match Cookie::parse(c) {
            Ok(parsed_cookie) => {
                if parsed_cookie.name() == want_cookie_name {
                    return Some(parsed_cookie.value().to_string());
                }
            }
            Err(e) => {
                tracing::error!("cookie parse error: {:?}", e);
            }
        }
    }
    None
}
