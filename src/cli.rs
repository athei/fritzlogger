use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub fn get_args() -> ArgMatches<'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("run")
                .about("Run the daemon that collects data in foreground")
                .arg(
                    Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .value_name("FILE")
                        .help("Path to the configuration file.")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("defconfig")
                .about("Output a complete config containing all default values"),
        )
        .get_matches()
}
