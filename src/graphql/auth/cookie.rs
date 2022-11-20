use axum_extra::extract::cookie::Cookie;
use hyper::{header::COOKIE, HeaderMap};

pub fn get_cookie_from_header(headers: &HeaderMap) -> Option<Vec<&str>> {
    headers.get(COOKIE).and_then(|v| {
        v.to_str().ok().map(|c| {
            let cookie = c.split(';').collect::<Vec<&str>>();
            cookie
        })
    })
}

// todo 複数取得できるようにする
pub fn get_value_from_cookie(headers: &HeaderMap, want_cookie_name: &str) -> Option<String> {
    let cookie = get_cookie_from_header(headers)?;

    for c in cookie {
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
