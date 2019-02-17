use structopt::StructOpt;

use super::{CliCommand, GlobalFlags};

pub const AFTER_HELP: &str = r#"EXAMPLES:
    To upgrade all packages in your environment:
    $ deck upgrade

    To upgrade a specific package to the latest version:
    $ deck upgrade firefox

    To upgrade to a specific version of a package:
    $ deck upgrade firefox:67.0.0-alpha1

    To upgrade a specific set of packages:
    $ deck upgrade firefox:67.0.0-alpha1 emacs:25.1.0 ffmpeg:4.1.0

This command is a convenient shorthand for `deck package -u <PACKAGE>`.
Any package transaction can be atomically rolled back `deck revert`. See
`deck revert --help` for more details.
"#;

#[derive(Debug, StructOpt)]
pub struct Upgrade {
    /// Package manifest specifiers
    #[structopt(empty_values = false, value_name = "PACKAGE", required = true)]
    packages: Vec<String>,
}

impl CliCommand for Upgrade {
    fn run(self, _flags: GlobalFlags) -> Result<(), String> {
        unimplemented!()
    }
}
