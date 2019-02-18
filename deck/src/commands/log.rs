use structopt::StructOpt;

use super::{CliCommand, GlobalFlags};

pub const AFTER_HELP: &str = r#"EXAMPLES:
    To get the build logs for a package:
    $ deck log firefox:67.0.0-alpha1@fc3j3vub6kodu4jtfoakfs5xhumqi62m"#;

#[derive(Debug, StructOpt)]
pub struct Log {
    /// Package manifest specifier
    #[structopt(value_name = "PACKAGE", empty_values = false)]
    manifest_id: String,
}

impl CliCommand for Log {
    fn run(self, _flags: GlobalFlags) -> Result<(), String> {
        unimplemented!()
    }
}
