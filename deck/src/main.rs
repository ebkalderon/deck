#![forbid(unsafe_code)]

use std::process;

use structopt::StructOpt;

use commands::{GlobalFlags, Subcommand};

mod commands;

const AFTER_HELP: &str = r#"Deck is a declarative system package manager which uses hermetic builds
and content-addressability to ensure packages are reproducible and easily
verified.

Deck supports the installation of multiple incompatible versions of
packages with ease and supports transactional upgrades and rollbacks.
Packages can be easily copied to and from other systems running Deck,
allowing for environments to be easily replicated between machines.

Deck can also be used as a meta build system on top of existing language
package managers and build systems, such as Cargo, npm, or make.

Note: This is an ongoing work in progress and may eat your laundry.
Stability is not guaranteed and plenty of functionality is missing.

EXAMPLES:
    To install the latest version of a package:
    $ deck install firefox

    To uninstall an existing package:
    $ deck remove firefox

    To search for packages in all repositories:
    $ deck search firefox

    To fetch upstream updates:
    $ deck update

    To upgrade all packages to the latest versions:
    $ deck upgrade

    To roll back the most recent transaction:
    $ deck revert
"#;

/// Declarative package manager
#[derive(Debug, StructOpt)]
#[structopt(name = "deck", raw(after_help = "AFTER_HELP"))]
struct Opt {
    #[structopt(flatten)]
    flags: GlobalFlags,
    #[structopt(subcommand)]
    command: Subcommand,
}

fn main() {
    let opt = Opt::from_args();

    if let Err(e) = opt.command.run(opt.flags) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
