use structopt::StructOpt;

use super::{CliCommand, GlobalFlags};

pub const AFTER_HELP: &str = r#"EXAMPLES:
    To fetch updates for all repositories:
    $ deck update

    To fetch updates for a specific repository:
    $ deck update --repo stable
"#;

#[derive(Debug, StructOpt)]
pub struct Update {
    /// Specific repository to update
    #[structopt(long = "repo", empty_values = false, value_name = "REPO")]
    repo: Option<String>,
}

impl CliCommand for Update {
    fn run(self, _flags: GlobalFlags) -> Result<(), String> {
        unimplemented!()
    }
}
