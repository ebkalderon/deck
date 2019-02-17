use structopt::StructOpt;

use super::{CliCommand, GlobalFlags};

pub const AFTER_HELP: &str =
    r#"By default, this will verify the integrity of all elements present in the store.
Specifying `--manifests`, `--outputs`, or `--sources` will restrict verification
to only specific types of elements.

EXAMPLES:
    To verify the entire Deck store:
    $ deck verify

    To verify a specific object:
    $ deck verify firefox:67.0.0-alpha1@123456789abcdef

    To verify a custom Deck store:
    $ deck verify --store-dir ./my-local-store

"#;

#[derive(Debug, StructOpt)]
pub struct Verify {
    /// Verify package manifests
    #[structopt(long = "manifests")]
    manifests: bool,
    /// Verify installed outputs
    #[structopt(long = "outputs")]
    outputs: bool,
    /// Verify downloaded sources
    #[structopt(long = "sources")]
    sources: bool,
    /// Package outputs, sources, and/or manifests to validate
    specifiers: Vec<String>,
}

impl CliCommand for Verify {
    fn run(self, _flags: GlobalFlags) -> Result<(), String> {
        unimplemented!()
    }
}
