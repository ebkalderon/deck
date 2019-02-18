use structopt::StructOpt;

use super::{CliCommand, GlobalFlags};

pub const AFTER_HELP: &str = r#"EXAMPLES:
    To list all installed packages:
    $ deck list

    To list all installed packages with the prefix "emacs":
    $ deck list emacs
"#;

#[derive(Debug, StructOpt)]
pub struct List {
    /// Profile to list packages from
    #[structopt(
        short = "p",
        long = "profile",
        empty_values = false,
        value_name = "PROFILE_NAME"
    )]
    profile: Option<String>,
    /// Regular expression for filtering package names
    #[structopt(value_name = "PATTERN")]
    pattern: Option<String>,
}

impl CliCommand for List {
    fn run(self, _flags: GlobalFlags) -> Result<(), String> {
        unimplemented!()
    }
}
