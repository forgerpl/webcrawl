#![deny(missing_docs)]

//! Website Crawling library
//!
//! This library facilitates building web url scrapers.
//! It is easily composable due to being future-based.
//!
//! For simple use case, see [Crawler](struct.Crawler.html) example.

use error::*;

pub use reqwest::IntoUrl;
pub use ty::{Fetcher, Parser, Urls};
pub use url::Url;

use futures::{Async, AsyncSink, Future, Poll, Sink, Stream};
use ty::Token;

mod error;
#[macro_use]
mod macros;
mod fetcher;
mod parser;
pub mod prelude;
pub mod ty;
mod url_parser;

/// Website crawler
///
/// This object is meant to be used with split stream and sink parts
/// to build the crawler processing pipeline.
/// Both document fetcher and parser can be customized with user-provided functions.
///
/// # Example
///
/// ```no_run
/// # use failure::Error;
/// # type Result<T> = std::result::Result<T, Error>;
/// use crawler::prelude::*;
/// use futures::{Future, Stream, Sink, lazy};
///
/// # fn main() -> Result<()> {
/// let crawler = Crawler::new(
///     "http://example.com",
///     fetch,
///     |base, html| parse(base, html, parse_url),
/// )?;
///
/// let (sink, stream) = crawler.split();
///
/// stream
///     // limit to 100 concurrent connections
///     .buffer_unordered(100)
///     .forward(sink)
///     .and_then(|(stream, sink)| {
///         let crawler = stream.into_inner().reunite(sink)?;
///
///         Ok(crawler.into_result())
///     });
/// # Ok(())
/// # }
/// ```

pub struct Crawler {
    /// all crawled urls, including redirected ones
    crawled: Urls,
    /// urls to be crawled
    queue: Urls,
    /// the resulting urls, without redirects
    effective: Urls,
    /// active tasks counter
    token: Token,

    /// a document fetching function
    fetcher: Fetcher,
    /// document parser
    parser: Parser,
}

impl Crawler {
    /// Create new Crawler
    ///
    /// # Arguments
    /// start: a starting url to be used as a seed for the crawler
    /// fetcher: a Fetcher used for linked documents retrieval
    /// parser: a Parser used for Url extraction

    pub fn new(start: impl IntoUrl, fetcher: Fetcher, parser: Parser) -> Result<Self> {
        let start = start.into_url()?;

        let queue = {
            let mut q = Urls::new();
            q.insert(start);
            q
        };

        Ok(Crawler {
            crawled: Urls::new(),
            queue,
            effective: Urls::new(),
            token: Token::new(()),
            fetcher,
            parser,
        })
    }

    /// Return all extracted Urls
    ///
    /// Calling this method only makes sense after the Crawler finishes crawling.

    pub fn into_result(self) -> Urls {
        self.effective
    }
}

impl Stream for Crawler {
    type Item = Box<dyn Future<Item = Option<CrawlerPayload>, Error = Error> + Send>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let url = self.queue.iter().next().cloned();

        if let Some(url) = url {
            // remove from the queue
            // unfortunately this performs additional lookup
            // but it shouldn't a bottleneck
            self.queue.remove(&url);

            self.crawled.insert(url.clone());

            let parser = self.parser;

            let token = self.token.clone();

            Ok(Async::Ready(Some(Box::new(
                (self.fetcher)(url.clone()).and_then(move |opt| {
                    if let Some((url, buffer)) = opt {
                        (parser)(url.clone(), buffer.as_str()).map(move |parsed| {
                            Some(CrawlerPayload::new(url.clone(), parsed, token))
                        })
                    } else {
                        Ok(None)
                    }
                }),
            ))))
        } else {
            // as the place when this is increased is here
            // there shouldn't be any problems with concurrent increments
            // 1 == only self
            if Token::strong_count(&self.token) > 1 {
                Ok(Async::NotReady)
            } else {
                Ok(Async::Ready(None))
            }
        }
    }
}

impl Sink for Crawler {
    type SinkItem = Option<CrawlerPayload>;
    type SinkError = Error;

    fn start_send(
        &mut self,
        item: Self::SinkItem,
    ) -> std::result::Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        // token will be dropped after we populate the queue
        // so the stream will have a chance to react

        if let Some(item) = item {
            let CrawlerPayload {
                source,
                urls,
                token: _token,
            } = item;
            self.crawled.insert(source.clone());
            self.effective.insert(source);
            self.queue.extend(urls.difference(&self.crawled).cloned());
        }

        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(Async::Ready(()))
    }
}

// this is required, because Debug cannot be auto-derived
// for function pointers with reference arguments
impl std::fmt::Debug for Crawler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Crawler")
    }
}

/// A type used by the Crawler Stream-Sink pipeline
///
/// The crawling queue can be additionally seeded by attaching an additional
/// `CrawlerPayload` stream to the sink.

#[derive(Debug)]
pub struct CrawlerPayload {
    /// url of the source document for URLs
    source: Url,
    /// parsed URLs
    urls: Urls,
    /// queue processing token
    token: Token,
}

impl CrawlerPayload {
    /// Construct new payload
    ///
    /// # Arguments
    /// source: source Url of scraped URLs
    /// urls: a set of scraped URLs
    /// token: an opaque liveness control type; for manual use pass Token::new()

    pub fn new(source: Url, urls: Urls, token: Token) -> Self {
        Self {
            source,
            urls,
            token,
        }
    }
}
