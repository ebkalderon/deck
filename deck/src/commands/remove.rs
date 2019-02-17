use structopt::StructOpt;

use super::{CliCommand, GlobalFlags};

pub const AFTER_HELP: &str = r#"EXAMPLES:
    To remove the latest version of an installed package:
    $ deck remove firefox

    To remove a specific version of an installed package:
    $ deck remove firefox:67.0.0-alpha1

    To remove multiple packages:
    $ deck remove firefox emacs:25.1.0 ffmpeg

This command is a convenient shorthand for `deck package -r <PACKAGE>`.
Any package transaction can be atomically rolled back `deck revert`. See
`deck revert --help` for more details.
"#;

#[derive(Debug, StructOpt)]
pub struct Remove {
    /// Package manifest specifiers
    #[structopt(empty_values = false, value_name = "PACKAGE", required = true)]
    packages: Vec<String>,
}

impl CliCommand for Remove {
    fn run(self, _flags: GlobalFlags) -> Result<(), String> {
        unimplemented!()
    }
}
