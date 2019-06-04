use failure::Error;
use tokio::prelude::*;
type Result<T> = std::result::Result<T, Error>;

use crawler::prelude::*;
use crawler::ty::FetchBuffer;
use futures::lazy;
use hashbrown::HashMap;

use lazy_static::lazy_static;

static PAGE0: &str = r#""<!doctype html>
<html>
    <head>
        <title>Parse test - index</title>
    </head>
    <body>
        <a href="weird/path/first.html">first</a>
    </body>
</html>
"#;

static PAGE1: &str = r#""<!doctype html>
<html>
    <head>
        <title>Parse test - page1</title>
    </head>
    <body>
        Absolute page: <a href="/three.html">three</a>
        Url with different base <a href="http://google.com/search?q=google">google</a>.
        Url with full domain <a href="https://example.com/some/path/second.html">test</a>.
    </body>
</html>
"#;

static PAGE2: &str = r#""<!doctype html>
<html>
    <head>
        <title>Parse test - page2</title>
    </head>
    <body>
        Absolute page: <a href="/three.html">three</a>
        Relative <a href="some/path/fourth.html">test</a>.
    </body>
</html>
"#;

static PAGE3: &str = r#""<!doctype html>
<html>
    <head>
        <title>Parse test - page3</title>
    </head>
    <body>
        This is an alias: <a href="/redirect.html">redirect -> index.html</a>
        This doesn't exist: <a href="/missing.html">missing</a>
    </body>
</html>
"#;

static PAGE4: &str = r#""<!doctype html>
<html>
    <head>
        <title>Parse test - page4</title>
    </head>
    <body>
        Nothing interesting here.
        http://example.com/foobar.html
    </body>
</html>
"#;

lazy_static! {
    static ref PAGES: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::with_capacity(5);
        map.insert("https://example.com/index.html", PAGE0);
        map.insert("https://example.com/weird/path/first.html", PAGE1);
        map.insert("https://example.com/some/path/second.html", PAGE2);
        map.insert("https://example.com/three.html", PAGE3);
        map.insert("https://example.com/some/path/some/path/fourth.html", PAGE4);

        map
    };
}

pub fn fetch(url: Url) -> Box<dyn Future<Item = Option<(Url, FetchBuffer)>, Error = Error> + Send> {
    Box::new(lazy(move || {
        let mut url_str = url.to_string();
        let mut real_url = url.clone();

        if url_str == "https://example.com/redirect.html" {
            url_str = "https://example.com/index.html".to_string();
            real_url = Url::parse(&url_str).unwrap();
        }

        if let Some(page) = PAGES.get(&url_str.as_ref()) {
            let r: FetchBuffer = Box::new(*page);

            Ok(Some((real_url, r)))
        } else {
            // not found
            Ok(None)
        }
    }))
}

fn tokio_run<F>(f: F) -> std::result::Result<F::Item, F::Error>
where
    F: IntoFuture,
    F::Future: Send + 'static,
    F::Item: Send + 'static,
    F::Error: Send + 'static,
{
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(f.into_future())
}

macro_rules! urls {
    () => {{
        use crawler::prelude::Urls;

        Urls::new()
    }};

    ( $($value:expr),+ $(,)* ) => {{
        use crawler::prelude::Urls;
        use url::Url;

        let mut hash = Urls::new();
        $(
            hash.insert(Url::parse($value).unwrap());
        )*

        hash
    }};
}

#[test]
fn smoke() -> Result<()> {
    let crawler = Crawler::new("https://example.com/index.html", fetch, |base, html| {
        parse(base, html, parse_url)
    })?;

    let (sink, stream) = crawler.split();

    let fut = stream
        .buffer_unordered(5)
        .forward(sink)
        .and_then(|(stream, sink)| {
            let crawler = stream.into_inner().reunite(sink)?;

            Ok(crawler.into_result())
        });

    let result = tokio_run(fut)?;

    let expected = urls! {
        "https://example.com/some/path/some/path/fourth.html",
        "https://example.com/index.html",
        "https://example.com/weird/path/first.html",
        "https://example.com/three.html",
        "https://example.com/some/path/second.html",
    };

    assert_eq!(result, expected);

    Ok(())
}
