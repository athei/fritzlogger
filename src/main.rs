#![recursion_limit = "1024"]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::correctness)]

use backend::Dispatcher;
use errors::*;

use clap::ArgMatches;
use error_chain::quick_main;
use reqwest::r#async::*;
use tokio::prelude::*;
use tokio::timer::Interval;

use std::borrow::Borrow;
use std::time::{Duration, Instant};

mod auth;
mod backend;
mod cli;
mod device;
mod settings;
mod xml;

mod errors {
    error_chain::error_chain! {}
}

fn fetch_body(request: RequestBuilder) -> impl Future<Item = String, Error = Error> {
    request
        .send()
        .and_then(|res| res.into_body().concat2())
        .map_err(|e| Error::with_chain(e, "Receiving response failed"))
        .and_then(|body| {
            String::from_utf8(body.into_iter().collect())
                .chain_err(|| "Decoding response as UTF-8 failed")
        })
}

fn print_errors<T: Borrow<Error>>(e: T) {
    let stderr = &mut ::std::io::stderr();
    let errmsg = "Error writing to stderr";

    writeln!(stderr, "Error: {}", e.borrow()).expect(errmsg);

    for e in e.borrow().iter().skip(1) {
        writeln!(stderr, "Caused by: {}", e).expect(errmsg);
    }
}

fn app(
    client: Client,
    settings: settings::Base,
    poll_interval: Duration,
) -> impl Future<Item = (), Error = Error> {
    auth::auth(
        &client,
        &settings.url,
        settings.username.clone(),
        settings.password.clone(),
    )
    .and_then(move |sid| {
        Interval::new(Instant::now(), poll_interval)
            .for_each(move |t| {
                device::devicelistinfos(&client, &settings.url, &sid)
                    .map(move |list| Dispatcher::dispatch(t, &list))
                    .or_else(|e| {
                        let err = Error::with_chain(e, "Failed getting device infos");
                        print_errors(&err);
                        Ok(())
                    })
            })
            .map_err(|e| Error::with_chain(e, "Interval failed"))
    })
}

fn command_run(args: &ArgMatches<'static>) -> Result<()> {
    let cfg_path = args
        .value_of("config")
        .chain_err(|| "Config file must be specified")?;
    settings::load(cfg_path)?;
    let settings: settings::Base = settings::get_base()?;
    Dispatcher::init(&settings.backends)?;
    let client = Client::new();
    let poll_interval = Duration::from_secs(settings.interval);
    let app = app(client, settings, poll_interval).map_err(print_errors);
    tokio::run(app);
    Ok(())
}

fn command_defconfig(_: &ArgMatches<'static>) -> Result<()> {
    print!("{}", settings::defaults()?);
    Ok(())
}

fn run() -> Result<()> {
    match cli::get_args().subcommand() {
        ("run", Some(sub)) => command_run(&sub),
        ("defconfig", Some(sub)) => command_defconfig(&sub),
        _ => Err("Inconsistent command line. This is a bug.".into()),
    }
}

quick_main!(run);
