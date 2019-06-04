//! Type and trait definitions

use crate::error::*;
use futures::Future;
use hashbrown::HashSet;
use std::sync::Arc;
use url::Url;

/// A set of URLs
pub type Urls = HashSet<Url>;
pub(crate) type Token = Arc<()>;

/// An opaque type that can be converted to &str for parsing
pub type FetchBuffer = Box<dyn AsStr + Send>;

/// A document fetcher type, allowing for pluggable custom fetcher implementations
pub type Fetcher =
    fn(url: Url) -> Box<dyn Future<Item = Option<(Url, FetchBuffer)>, Error = Error> + Send>;
/// A document parser type, allowing for pluggable custom parser implementations
pub type Parser = for<'a> fn(base: Url, html: &'a str) -> Result<Urls>;
/// A url parser to be used by the selected Parser
pub type UrlParser =
    for<'a, 'b> fn(base: &'a Url, target: &'b str) -> std::result::Result<Url, UrlParseError>;

/// Convert given type to &str
pub trait AsStr {
    /// Return type's value as a string slice
    fn as_str(&self) -> &str;
}

impl AsStr for &'static str {
    fn as_str(&self) -> &str {
        *self
    }
}
