//! Web server-based web crawler
//!
//! The API of the server is very simple:
//!
//! # Get all crawled domains
//! `GET /api/domains`
//!
//! # Schedule a crawl
//! `POST /api/crawl`
//!
//! ## Payload:
//!
//! ```json
//! {
//!     url: "http://example.com",
//!     throttle: 50,
//! }
//! ```
//!
//! ### where:
//! - `url`: an url to be crawled
//! - `throttle`: a maximum number of concurrent requests
//!
//! ## Response:
//!
//! ```json
//! {
//!     "id": "http://example.com"
//! }
//! ```
//!
//! ## Additional status codes:
//! - `400` - if the payload is malformed, or it contains invalid URL
//! - `409` - if the crawl is already pending
//!
//! # Get results of the crawl
//! `GET /api/results?id={id}`
//!
//! ## Response
//!
//! A json list of retrieved URLs
//!
//! ## Additional status codes:
//! - `202` - if the crawl is pending and the result is not yet available
//! - `404` - if the `id` is not present in the results cache
//!
//! # Get number of results of the crawl
//! `GET /api/results/count?id={id}`
//!
//! ## Response:
//!
//! ```json
//! {
//!     "http://example.com": 123
//! }
//! ```
//!
//! ## Additional status codes:
//! - `202` - if the crawl is pending and the result is not yet available
//! - `404` - if the `id` is not present in the results cache

use error::*;
use log::*;

use crawler::prelude::*;

use cli::setup_cli;
use util::{find_query_param, get_result};

use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use serde_derive::{Deserialize, Serialize};
use serde_json;
use tokio::prelude::*;
use url::Url;

use hashbrown::{hash_map::Entry, HashMap};
use std::borrow::Borrow;
use std::net::ToSocketAddrs;
use std::sync::{Arc, RwLock};

use std::str::from_utf8;

use env_logger;

mod cli;
mod error;
mod util;

#[derive(Debug)]
enum CrawlStatus {
    Pending,
    Finished(Urls),
}

type Registry = Arc<RwLock<HashMap<String, CrawlStatus>>>;

fn schedule(url: Url, throttle: usize, registry: Registry) {
    let origin = url.origin().ascii_serialization();

    tokio::spawn(future::lazy(move || {
        info!("Scheduling crawl of {}", url);

        let crawler = Crawler::new(url, fetch, |base, html| parse(base, html, parse_url)).unwrap();

        let (sink, stream) = crawler.split();

        stream
            .buffer_unordered(throttle)
            .forward(sink)
            .and_then(|(stream, sink)| {
                let crawler = stream.into_inner().reunite(sink)?;

                Ok(crawler.into_result())
            })
            .and_then(move |urls| {
                let len = urls.len();
                let mut reg = registry.write().expect("failed to write to registry");
                info!(
                    "Finished crawling domain {}, retrieved {} urls",
                    origin, len
                );

                reg.insert(origin, CrawlStatus::Finished(urls));

                Ok(())
            })
            .map_err(|_| ())
    }));
}

fn main() -> Result<()> {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "webcrawl=info");
    env_logger::Builder::from_env(env).init();

    // storage for the results of the crawl operation
    let registry: Arc<RwLock<HashMap<String, CrawlStatus>>> = Arc::new(RwLock::new(HashMap::new()));

    let api = move || {
        let registry = registry.clone();

        move |req: Request<Body>|
        -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
            let path = req.uri().path();
            let method = req.method();

            let response = match (method, path) {
                (&Method::GET, "/api/domains") => {

                    let result = (|| {
                        let registry = registry.read()
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                        let names = registry.keys().collect::<Vec<_>>();
                        let resp = serde_json::to_string(&names)
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                        Ok(Body::from(resp))
                    })();

                    get_result(result)
                }
                (&Method::GET, "/api/results/count") => {
                    let result = (|| {
                        let uri = req.uri().to_string();
                        let url = Url::parse("http://dummy")
                            .and_then(|url| url.join(&uri))
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                        let id = find_query_param(&url, "id")?;

                        let registry = registry.read()
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                        let id: &str = id.borrow();

                        let urls = registry.get(id)
                            .ok_or_else(|| StatusCode::NOT_FOUND)?;

                        if let CrawlStatus::Finished(urls) = urls {
                            let count = urls.len();

                            let resp = {
                                let mut h = HashMap::with_capacity(1);
                                h.insert(id, count);
                                h
                            };

                            let resp = serde_json::to_string(&resp)
                                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                                Ok(Body::from(resp))
                        } else {
                            Err(StatusCode::ACCEPTED)
                        }

                    })();

                    get_result(result)
                }
                (&Method::GET, "/api/results") => {
                    let result = (|| {
                        let uri = req.uri().to_string();
                        let url = Url::parse("http://dummy")
                            .and_then(|url| url.join(&uri))
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                        let id = find_query_param(&url, "id")?;

                        let registry = registry.read()
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                        let id: &str = id.borrow();

                        let urls = registry.get(id)
                            .ok_or_else(|| StatusCode::NOT_FOUND)?;

                        if let CrawlStatus::Finished(urls) = urls {
                            let urls = urls
                                .iter()
                                .map(|url| url.to_string())
                                .collect::<Vec<_>>();
                            let resp = serde_json::to_string(&urls)
                                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                                Ok(Body::from(resp))
                        } else {
                            Err(StatusCode::ACCEPTED)
                        }

                    })();

                    get_result(result)
                }
                (&Method::POST, "/api/crawl") => {
                    #[derive(Debug, Deserialize)]
                    struct CrawlRequest<'a> {
                        url: &'a str,
                        throttle: usize,
                    }

                    #[derive(Debug, Serialize)]
                    struct CrawlResponse<'a> {
                        id: &'a str,
                    }

                    let process = req.into_body()
                    .concat2()
                    .map({
                        let registry = registry.clone();

                        move |chunk| {
                            let result = (move || {
                            let body = from_utf8(&chunk)
                                // invalid utf-8
                                .map_err(|_| StatusCode::BAD_REQUEST)?;
                            let apireq = serde_json::from_str::<CrawlRequest>(body)
                                // invalid json
                                .map_err(|_| StatusCode::BAD_REQUEST)?;
                            let url = Url::parse(apireq.url)
                                // invalid url in the payload
                                .map_err(|_| StatusCode::BAD_REQUEST)?;

                            let origin = url.origin().ascii_serialization();

                            let apiresp = CrawlResponse {
                                id: &origin,
                            };

                            let serialized = serde_json::to_string(&apiresp)
                                // response serialization error
                                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                            {
                                let mut reg = registry.write()
                                    // unable to acquire lock
                                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                                if let Entry::Vacant(e) = reg.entry(origin) {
                                    e.insert(CrawlStatus::Pending);

                                    schedule(url, apireq.throttle, registry.clone());

                                    // scheduled
                                    Ok(Body::from(serialized))
                                } else {
                                    // already scheduled/available
                                    Err(StatusCode::CONFLICT)
                                }
                            }
                        })();

                        get_result(result)
                    }});

                    return Box::new(process)
                }
                _ => {
                    get_result(Err(StatusCode::NOT_FOUND))
                }
            };

            Box::new(future::ok(response))
        }
    };

    let args = setup_cli().get_matches();

    let addr = args
        .value_of("address")
        .unwrap()
        .to_socket_addrs()?
        .next()
        .unwrap();

    info!("Starting server on {}", addr);

    let server = Server::bind(&addr)
        .serve(move || service_fn(api()))
        .map_err(|e| error!("server error: {}", e));

    tokio::run(server);

    Ok(())
}
