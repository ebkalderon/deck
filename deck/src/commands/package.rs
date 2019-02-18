use std::path::PathBuf;
use std::str::FromStr;

use structopt::StructOpt;

use super::{CliCommand, GlobalFlags};

pub const AFTER_HELP: &str = r#"EXAMPLES:
    To build a portable tarball with Plex and its runtime dependencies:
    $ deck package plex

    To build a portable tarball cross-compiled for another platform:
    $ deck package --target x86_64-unknown-linux plex

    To build a Docker slug with Plex and its runtime dependencies:
    $ deck package -f docker plex

    To symlink the `/usr/bin` directory to the `bin` subdirectory of Plex for convenience:
    $ deck package -S /usr/bin=bin -f docker plex
"#;

#[derive(Debug, StructOpt)]
pub struct Package {
    /// Format of this portable package
    #[structopt(
        short = "f",
        long = "format",
        value_name = "FORMAT",
        empty_values = false,
        default_value = "tarball",
        parse(try_from_str),
        raw(possible_values = "&[\"docker\", \"tarball\"]")
    )]
    format: Format,
    /// Output file name
    #[structopt(
        short = "o",
        long = "output",
        value_name = "OUTPUT_FILE",
        default_value = "package.tar.gz",
        empty_values = false,
        parse(from_os_str)
    )]
    output_file: PathBuf,
    /// Comma-separated list of symlinks to create (e.g. '/opt/bin=bin,/opt/etc=etc')
    #[structopt(
        short = "S",
        long = "symlink",
        value_name = "PATTERN",
        empty_values = false,
        require_delimiter = true
    )]
    symlinks: Vec<String>,
    /// Packages to export from the store
    #[structopt(value_name = "PACKAGE", required = true, empty_values = false)]
    manifest_ids: Vec<String>,
}

#[derive(Debug)]
enum Format {
    Tarball,
    Docker,
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tarball" => Ok(Format::Tarball),
            "docker" => Ok(Format::Docker),
            other @ _ => Err(format!("unrecognized format `{}`", other)),
        }
    }
}

impl CliCommand for Package {
    fn run(self, _flags: GlobalFlags) -> Result<(), String> {
        unimplemented!()
    }
}
