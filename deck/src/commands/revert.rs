use structopt::StructOpt;

use super::{CliCommand, GlobalFlags};

pub const AFTER_HELP: &str = r#"EXAMPLES:
    To roll back the most recent package transaction:
    $ deck revert

    To revert three package transactions back:
    $ deck revert 3

This command is a convenient shorthand for `deck profile -R <PATTERN>`.
To see the current transaction history, run `deck generations`.
"#;

#[derive(Debug, StructOpt)]
pub struct Revert {
    /// Profile from which to revert transactions
    #[structopt(
        short = "p",
        long = "profile",
        empty_values = false,
        value_name = "PROFILE_NAME"
    )]
    profile: Option<String>,
    /// Number of transactions to revert
    #[structopt(empty_values = false, value_name = "PATTERN", default_value = "1")]
    num_to_revert: u8,
}

impl CliCommand for Revert {
    fn run(self, _flags: GlobalFlags) -> Result<(), String> {
        unimplemented!()
    }
}
