use structopt::StructOpt;

use super::{CliCommand, GlobalFlags};

pub const AFTER_HELP: &str = r#"EXAMPLES:
    To search all repositories for a package containing a substring:
    $ deck search firefox

    To search a specific repository for a package:
    $ deck search --only-in stable firefox

    To search for a specific package with JSON output:
    $ deck search --json firefox | jq '.[].name'

This command will always search locally fetched repositories and never
reach out to the internet. To see the most up to date results, first run
`deck update`.
"#;

#[derive(Debug, StructOpt)]
pub struct Search {
    /// Output search results in JSON format
    #[structopt(long = "json", conflicts_with = "recutils")]
    json: bool,
    /// Output search results in GNU Recutils format
    #[structopt(long = "recutils")]
    recutils: bool,
    /// Limit search results to a specific set of repositories
    #[structopt(empty_values = false, long = "repo", value_name = "REPO")]
    repo: Option<String>,
    /// Regular expression(s)
    #[structopt(value_name = "REGEX", required = true)]
    keywords: Vec<String>,
}

impl CliCommand for Search {
    fn run(self, _flags: GlobalFlags) -> Result<(), String> {
        unimplemented!()
    }
}
