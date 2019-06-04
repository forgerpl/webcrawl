pub(crate) use failure::{err_msg, Error};
use url;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum UrlParseError {
    Parse(url::ParseError),
    BadOrigin,
}
