use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version, App, Arg};
use std::net::ToSocketAddrs;

pub(super) fn setup_cli<'a, 'b>() -> App<'a, 'b> {
    const DEFAULT_SERVER_BIND: &str = "localhost:8000";

    app_from_crate!().arg(
        Arg::with_name("address")
            .takes_value(true)
            .help("Address to bind to (e.g. 'localhost:8888')")
            .required(false)
            .default_value(DEFAULT_SERVER_BIND)
            .validator(|s| {
                s.to_socket_addrs()
                    .map(|_| ())
                    .map_err(|_| "invalid socket address".to_owned())
            })
            .short("a")
            .long("address"),
    )
}
