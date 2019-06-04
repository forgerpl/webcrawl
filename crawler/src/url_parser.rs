use crate::error::*;
use url::Url;

/// Parse given url using provided base
///
/// The purpose of this function is to deduce effective url from given base and scraped target.
/// It will return an error if target cannot be converted to a valid Url.
///
/// This implementation will also check if the result origin matches base origin
/// `(protocol, host, port)`

pub fn parse_url(base: &Url, target: &str) -> std::result::Result<Url, UrlParseError> {
    // test origin
    let url = base.join(target).map_err(UrlParseError::Parse)?;

    if url.origin() == base.origin() {
        Ok(url)
    } else {
        Err(UrlParseError::BadOrigin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_url {
        ($url1: expr, $url2: expr) => {{
            let s1 = $url1.to_string();
            let s2 = $url2.to_string();

            assert_eq!(s1, s2, "urls are not equal");
        }};
    }

    #[test]
    fn absolute_within_origin() {
        assert_url!(
            parse_url(&url!("http://test.domain"), "http://test.domain/foo.html").unwrap(),
            url!("http://test.domain/foo.html")
        );
    }

    #[test]
    fn absolute_outside_origin() {
        assert!(parse_url(&url!("http://test.domain"), "http://example.com/foo.html").is_err());
    }

    #[test]
    fn relative_within_origin() {
        assert_url!(
            parse_url(&url!("http://test.domain/a/b/bar.html"), "/a/b/foo.html").unwrap(),
            url!("http://test.domain/a/b/foo.html")
        );
    }

    #[test]
    fn relative_up_within_origin() {
        assert_url!(
            parse_url(&url!("http://test.domain/a/b/bar.html"), "../c/foo.html").unwrap(),
            url!("http://test.domain/a/c/foo.html")
        );
    }

    #[test]
    fn absolute_up_within_origin() {
        assert_url!(
            parse_url(&url!("http://test.domain/a/b/bar.html"), "/c/foo.html").unwrap(),
            url!("http://test.domain/c/foo.html")
        );
    }

    #[test]
    fn relative_sub_within_origin() {
        assert_url!(
            parse_url(&url!("http://test.domain/a/b/bar.html"), "c/foo.html").unwrap(),
            url!("http://test.domain/a/b/c/foo.html")
        );
    }
}
