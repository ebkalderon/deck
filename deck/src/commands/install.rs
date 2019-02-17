use structopt::StructOpt;

use super::{CliCommand, GlobalFlags};

pub const AFTER_HELP: &str = r#"EXAMPLES:
    To install the latest version of a package:
    $ deck install firefox

    To install a specific version of a package:
    $ deck install firefox:67.0.0-alpha1

    To install multiple packages:
    $ deck install firefox emacs:25.1.0 ffmpeg

This command is a convenient shorthand for `deck package -i <PACKAGE>`.
Any package transaction can be atomically rolled back `deck revert`. See
`deck revert --help` for more details.
"#;

#[derive(Debug, StructOpt)]
pub struct Install {
    /// Package manifest specifiers
    #[structopt(empty_values = false, value_name = "PACKAGE", required = true)]
    packages: Vec<String>,
}

impl CliCommand for Install {
    fn run(self, _flags: GlobalFlags) -> Result<(), String> {
        unimplemented!()
    }
}
