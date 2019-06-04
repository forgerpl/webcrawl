use crate::error::*;
use crate::ty::{AsStr, FetchBuffer};
use futures::{Future, Stream};
use log::*;
use reqwest::r#async::{Chunk, Client};
use std::str::from_utf8;
use url::Url;

impl AsStr for Chunk {
    fn as_str(&self) -> &str {
        from_utf8(self.as_ref()).unwrap()
    }
}

/// Simple document fetcher, based on reqwest library
///
/// It will fetch given document, returning a pair (effective_url, FetchBuffer), which can then
/// be parsed.

pub fn fetch(url: Url) -> Box<dyn Future<Item = Option<(Url, FetchBuffer)>, Error = Error> + Send> {
    let client = Client::new();

    debug!("Fetching {}", url);

    // TODO: differentiate between errors

    Box::new(
        client
            .get(url.clone())
            .send()
            .and_then(|response| {
                let real_url = response.url().clone();
                response
                    .into_body()
                    .concat2()
                    .and_then(move |chunks| Ok((real_url, chunks)))
            })
            .and_then(move |(real_url, chunk)| {
                let r: FetchBuffer = Box::new(chunk);

                Ok(Some((real_url, r)))
            })
            .or_else(|_| Ok(None)),
    )
}
