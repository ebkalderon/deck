use std::path::PathBuf;

use structopt::StructOpt;

use super::{CliCommand, CommonFlags};

#[derive(Debug, StructOpt)]
pub struct Build {
    #[structopt(parse(from_os_str))]
    manifest: PathBuf,
}

impl CliCommand for Build {
    fn run(self, _flags: CommonFlags) -> Result<(), String> {
        unimplemented!()
    }
}
