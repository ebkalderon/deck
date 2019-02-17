use structopt::StructOpt;

use super::{CliCommand, GlobalFlags};

pub const AFTER_HELP: &str = r#"EXAMPLES:
    To install the latest version of a package:
    $ deck package -i firefox

    To install a specific version of a package:
    $ deck package -i firefox:67.0.0-alpha1

    To remove a package:
    $ deck package -r firefox

    To upgrade a package to the latest version:
    $ deck package -u firefox

    To perform multiple operations in a single transaction:
    $ deck package -i emacs -r ffmpeg -u firefox

Any package transaction, whether if done here or with the convenience
commands `deck install`, `deck remove`, and `deck upgrade`, can be
atomically rolled back `deck revert`. See `deck revert --help` for more
details.
"#;

#[derive(Debug, StructOpt)]
pub struct Package {
    /// Install a package
    #[structopt(
        short = "i",
        long = "install",
        empty_values = false,
        value_name = "PACKAGE",
        raw(required_unless_one = "&[\"remove\", \"upgrade\"]")
    )]
    install: Vec<String>,
    /// Remove an existing package
    #[structopt(
        short = "r",
        long = "remove",
        empty_values = false,
        value_name = "PACKAGE",
        raw(required_unless_one = "&[\"install\", \"upgrade\"]")
    )]
    remove: Vec<String>,
    /// Upgrade a package to the latest version
    #[structopt(
        short = "u",
        long = "upgrade",
        empty_values = false,
        value_name = "PACKAGE",
        raw(required_unless_one = "&[\"install\", \"remove\"]")
    )]
    upgrade: Vec<String>,
}

impl CliCommand for Package {
    fn run(self, _flags: GlobalFlags) -> Result<(), String> {
        unimplemented!()
    }
}
