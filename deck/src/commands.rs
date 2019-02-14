use std::path::PathBuf;

use structopt::StructOpt;

use self::build::Build;

mod build;

/// Trait implemented by all subcommands.
trait CliCommand: StructOpt {
    /// Execute the command with the given global flags.
    fn run(self, flags: GlobalFlags) -> Result<(), String>;
}

/// Global command-line flags.
#[derive(Debug, StructOpt)]
pub struct GlobalFlags {
    /// No output printed to stdout
    #[structopt(
        global = true,
        short = "q",
        long = "quiet",
        conflicts_with = "\"verbose\""
    )]
    quiet: bool,
    #[structopt(
        global = true,
        long = "store-dir",
        env = "DECK_STORE_PATH",
        default_value = "/deck/store",
        parse(from_os_str)
    )]
    /// Path to the store directory
    store_path: PathBuf,
    /// Increase verbosity level of output
    #[structopt(
        global = true,
        short = "v",
        long = "verbose",
        env = "DECK_VERBOSE",
        parse(from_occurrences)
    )]
    verbosity: u8,
}

/// Built-in Deck client subcommands.
#[derive(Debug, StructOpt)]
pub enum Subcommand {
    /// Compile a package from source
    #[structopt(name = "build")]
    Build(Build),
    /// Install new packages
    ///
    /// This command is a convenient shorthand for `package --install <specifier>...`.
    #[structopt(name = "install")]
    Install,
    /// Perform a package transaction
    #[structopt(name = "package")]
    Package,
    /// Search repositories for packages
    #[structopt(name = "search")]
    Search,
    /// Uninstall existing packages
    ///
    /// This command is a convenient shorthand for `package --uninstall <specifier>...`.
    #[structopt(name = "uninstall")]
    Uninstall,
    /// Synchronize updates from upstream repositories
    #[structopt(name = "update")]
    Update,
    /// Upgrades existing packages in your environment
    ///
    /// This command is a convenient shorthand for `package --upgrade [<specifier>...]`.
    #[structopt(name = "upgrade")]
    Upgrade,
}

impl Subcommand {
    /// Executes the active subcommand with the given arguments.
    pub fn run(self, flags: GlobalFlags) -> Result<(), String> {
        println!("{:?}", flags);
        match self {
            Subcommand::Build(cmd) => cmd.run(flags),
            _ => Ok(()),
        }
    }
}
