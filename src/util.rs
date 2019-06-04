use hyper::{Body, Response, StatusCode};
use std::borrow::Cow;
use url::Url;

pub(super) fn find_query_param<'a, 'b>(
    url: &'a Url,
    name: &'b str,
) -> std::result::Result<Cow<'a, str>, StatusCode> {
    Ok(url
        .query_pairs()
        .find_map(|(key, value)| if key == name { Some(value) } else { None })
        .ok_or_else(|| StatusCode::BAD_REQUEST)?)
}

pub(super) fn get_result(result: std::result::Result<Body, StatusCode>) -> Response<Body> {
    let mut response = Response::builder();

    match result {
        Ok(body) => response
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(body),
        Err(status) => response.status(status).body(Body::empty()),
    }
    .expect("failed to create response")
}
