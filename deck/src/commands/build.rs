use std::path::PathBuf;

use structopt::StructOpt;

use super::{CliCommand, GlobalFlags};

#[derive(Debug, StructOpt)]
pub struct Build {
    #[structopt(parse(from_os_str))]
    manifest: PathBuf,
}

impl CliCommand for Build {
    fn run(self, _flags: GlobalFlags) -> Result<(), String> {
        unimplemented!()
    }
}
