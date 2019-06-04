use crate::error::*;
use crate::ty::{UrlParser, Urls};
use log::*;
use scraper::{Html, Selector};
use url::Url;

/// Url parser/scraper
///
/// # Arguments
/// base: a source url of the document to be parsed, used for resolving of relative links
/// html: a html document to be parsed
/// url_parser: a UrlParser function used for generating urls
///
/// Return value: a `Urls` containing all scraped urls, matching the criteria of given url_parser.

pub fn parse(base: Url, html: &str, url_parser: UrlParser) -> Result<Urls> {
    let doc = Html::parse_document(&html);
    let selector = Selector::parse("a").map_err(|_| err_msg("failed to parse selector"))?;

    Ok(doc
        .select(&selector)
        // filter out anchors without the href atrribute
        .filter_map(|a| a.value().attr("href"))
        // try to parse as an Url object
        .filter_map({
            let base = base.clone();

            move |link| {
                if let Ok(url) = (url_parser)(&base, link) {
                    Some(url)
                } else {
                    debug!("Skipping url: {:?}", link);
                    None
                }
            }
        })
        // clear fragments, to avoid multiple crawlings
        .map(|mut url| {
            url.set_fragment(None);
            url
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn simple() {
        let data = r#"<!doctype html>
        <html>
            <head>
                <title>Parse test</title>
            </head>
            <body>
                This is some <a href="testing.html">test</a> of the parser.
                Url with different base <a href="http://google.com/search?q=google">google</a>.
                Url with full domain <a href="http://example.com/some/path/second.html">test</a>.
            </body>
        </html>
        "#;

        let parsed = parse(url!("http://example.com/base.html"), &data, parse_url).unwrap();

        assert_eq!(
            parsed,
            hashset! { url!("http://example.com/testing.html"),
            url!("http://example.com/some/path/second.html") }
        )
    }

    #[test]
    fn with_fragments() {
        let data = r#"<!doctype html>
        <html>
            <head>
                <title>Parse test</title>
            </head>
            <body>
                <a href="index.html#first">first</a>
                <a href="index.html#second">first</a>
            </body>
        </html>
        "#;

        let parsed = parse(url!("http://example.com/base.html"), &data, parse_url).unwrap();

        assert_eq!(parsed, hashset! { url!("http://example.com/index.html") })
    }

    #[test]
    fn mixed() {
        let data = r#"<!doctype html>
        <html>
            <head>
                <title>Parse test</title>
            </head>
            <body>
                This shoulnd't be parsed http://example.com/foo.html

                <a href="/bar.html">bar</a>
            </body>
        </html>
        "#;

        let parsed = parse(url!("http://example.com/base.html"), &data, parse_url).unwrap();

        assert_eq!(parsed, hashset! { url!("http://example.com/bar.html") })
    }

}
