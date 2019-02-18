use std::str::FromStr;

use structopt::StructOpt;

use super::{CliCommand, GlobalFlags};

pub const AFTER_HELP: &str = r#"EXAMPLES:
    To install the latest version of a package:
    $ deck profile -i firefox

    To install a specific version of a package:
    $ deck profile -i firefox:67.0.0-alpha1

    To remove a package:
    $ deck profile -r firefox

    To upgrade a package to the latest version:
    $ deck profile -u firefox

    To revert the most recent transaction:
    $ deck profile -R

    To roll back the previous two transactions:
    $ deck profile -R 2

    To switch to generation 4:
    $ deck profile -S 4

    To switch to the generation 2 versions ahead:
    $ deck profile -S +2

    To perform multiple operations in a single transaction:
    $ deck profile -S +1 -i emacs -r ffmpeg -u firefox

When either `--revert` or `--switch` is specified alongside the other
operations, it is performed first and then the rest are performed. The
dedicated commands `install`, `remove`, `upgrade`, `revert`, and
`switch` are all convenient aliases for their respective flags in
`profile`.
"#;

#[derive(Debug, StructOpt)]
pub struct Profile {
    /// Profile to apply the transaction
    #[structopt(
        short = "p",
        long = "profile",
        empty_values = false,
        value_name = "PROFILE_NAME"
    )]
    profile: Option<String>,
    // FIXME: Must specify '' until 'https://github.com/clap-rs/clap/issues/1354' gets resolved.
    /// Roll back one or more package transactions (e.g. '', '2')
    #[structopt(
        short = "R",
        long = "revert",
        value_name = "PATTERN",
        display_order = 1,
        conflicts_with = "switch",
        raw(required_unless_one = "&[\"remove\", \"switch\", \"install\", \"upgrade\"]")
    )]
    revert: Option<Revert>,
    /// Switch to a generation (e.g. 3, -2, +5)
    #[structopt(
        short = "S",
        long = "switch",
        value_name = "PATTERN",
        allow_hyphen_values = true,
        empty_values = false,
        display_order = 2,
        raw(required_unless_one = "&[\"remove\", \"revert\", \"install\", \"upgrade\"]")
    )]
    switch: Option<Switch>,
    /// Install a package
    #[structopt(
        short = "i",
        long = "install",
        empty_values = false,
        value_name = "PACKAGE",
        display_order = 3,
        raw(required_unless_one = "&[\"remove\", \"revert\", \"switch\", \"upgrade\"]")
    )]
    install: Vec<String>,
    /// Remove an existing package
    #[structopt(
        short = "r",
        long = "remove",
        empty_values = false,
        value_name = "PACKAGE",
        display_order = 4,
        raw(required_unless_one = "&[\"install\", \"revert\", \"switch\", \"upgrade\"]")
    )]
    remove: Vec<String>,
    /// Upgrade a package to the latest version
    #[structopt(
        short = "u",
        long = "upgrade",
        empty_values = false,
        value_name = "PACKAGE",
        display_order = 5,
        raw(required_unless_one = "&[\"install\", \"remove\", \"revert\", \"switch\"]")
    )]
    upgrade: Vec<String>,
}

#[derive(Debug)]
enum Revert {
    Previous,
    Several(u8),
}

impl FromStr for Revert {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Revert::Previous)
        } else {
            s.parse()
                .map(Revert::Several)
                .map_err(|_| "invalid generation".to_string())
        }
    }
}

#[derive(Debug)]
enum Switch {
    Specific(u8),
    Forward(u8),
    Previous(u8),
}

impl FromStr for Switch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('+') {
            s.splitn(2, '+')
                .nth(1)
                .ok_or("invalid pattern".to_string())
                .and_then(|s| s.parse().map_err(|_| "not a valid generation".to_string()))
                .map(Switch::Forward)
        } else if s.starts_with('-') {
            s.splitn(2, '-')
                .nth(1)
                .ok_or("invalid pattern".to_string())
                .and_then(|s| s.parse().map_err(|_| "not a valid generation".to_string()))
                .map(Switch::Previous)
        } else {
            s.parse()
                .map(Switch::Specific)
                .map_err(|_| "not a valid generation".to_string())
        }
    }
}

impl CliCommand for Profile {
    fn run(self, _flags: GlobalFlags) -> Result<(), String> {
        unimplemented!()
    }
}
