//! This module contains all useful imports for this crate

pub use crate::ty::{Fetcher, Parser, Urls};
pub use crate::Crawler;

pub use crate::fetcher::fetch;
pub use crate::parser::parse;
pub use crate::url_parser::parse_url;

pub use reqwest::IntoUrl;
pub use url::Url;
