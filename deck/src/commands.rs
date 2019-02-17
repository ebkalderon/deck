use std::path::PathBuf;

use structopt::StructOpt;

use self::build::Build;
use self::install::{Install, AFTER_HELP as INSTALL_AFTER_HELP};
use self::package::{Package, AFTER_HELP as PACKAGE_AFTER_HELP};
use self::remove::{Remove, AFTER_HELP as REMOVE_AFTER_HELP};
use self::search::{Search, AFTER_HELP as SEARCH_AFTER_HELP};
use self::upgrade::{Upgrade, AFTER_HELP as UPGRADE_AFTER_HELP};
use self::verify::{Verify, AFTER_HELP as VERIFY_AFTER_HELP};

mod build;
mod install;
mod package;
mod remove;
mod search;
mod upgrade;
mod verify;

/// Trait implemented by all subcommands.
trait CliCommand: StructOpt {
    /// Execute the command with the given global flags.
    fn run(self, flags: GlobalFlags) -> Result<(), String>;
}

/// Global command-line flags.
#[derive(Debug, StructOpt)]
pub struct GlobalFlags {
    /// Simulate an action without doing anything
    #[structopt(global = true, long = "dry-run")]
    dry_run: bool,
    /// No output printed to stdout
    #[structopt(
        global = true,
        short = "q",
        long = "quiet",
        conflicts_with = "verbosity"
    )]
    quiet: bool,
    #[structopt(
        global = true,
        long = "store-dir",
        env = "DECK_STORE_PATH",
        empty_values = false,
        value_name = "PATH",
        default_value = "/deck/store",
        hide_default_value = true,
        hide_env_values = true,
        parse(from_os_str)
    )]
    /// Path to the store directory
    store_path: PathBuf,
    /// Increase verbosity level of output
    #[structopt(global = true, short = "v", long = "verbose", parse(from_occurrences))]
    verbosity: u8,
}

/// Built-in Deck client subcommands.
#[derive(Debug, StructOpt)]
pub enum Subcommand {
    /// Compile a package from source
    #[structopt(name = "build")]
    Build(Build),
    /// Install new packages
    #[structopt(name = "install", raw(after_help = "INSTALL_AFTER_HELP"))]
    Install(Install),
    /// Perform a package transaction
    #[structopt(name = "package", raw(after_help = "PACKAGE_AFTER_HELP"))]
    Package(Package),
    /// Uninstall existing packages
    #[structopt(name = "remove", raw(after_help = "REMOVE_AFTER_HELP"))]
    Remove(Remove),
    /// Search repositories for packages
    #[structopt(name = "search", raw(after_help = "SEARCH_AFTER_HELP")]
    Search(Search),
    /// Synchronize updates from upstream repositories
    #[structopt(name = "update")]
    Update,
    /// Upgrade existing packages in your environment
    #[structopt(name = "upgrade", raw(after_help = "UPGRADE_AFTER_HELP"))]
    Upgrade(Upgrade),
    /// Verify the integrity of store contents
    #[structopt(name = "verify", raw(after_help = "VERIFY_AFTER_HELP"))]
    Verify(Verify),
}

impl Subcommand {
    /// Executes the active subcommand with the given arguments.
    pub fn run(self, flags: GlobalFlags) -> Result<(), String> {
        println!("{:?}", flags);
        println!("{:?}", self);
        match self {
            Subcommand::Build(cmd) => cmd.run(flags),
            _ => Ok(()),
        }
    }
}
