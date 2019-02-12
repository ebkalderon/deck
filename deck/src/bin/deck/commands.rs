use structopt::StructOpt;

use self::build::Build;

mod build;

trait CliCommand: StructOpt {
    fn run(self, flags: CommonFlags) -> Result<(), String>;
}

#[derive(Debug, StructOpt)]
pub struct CommonFlags {
    /// No output printed to stdout
    #[structopt(
        short = "q",
        long = "quiet",
        raw(global = "true", conflicts_with = "\"verbose\"")
    )]
    quiet: bool,
    /// Increase verbosity level of output
    #[structopt(
        short = "v",
        long = "verbose",
        raw(global = "true"),
        parse(from_occurrences)
    )]
    verbosity: u8,
}

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
    pub fn run(self, flags: CommonFlags) -> Result<(), String> {
        println!("{:?}", flags);
        match self {
            Subcommand::Build(cmd) => cmd.run(flags),
            _ => Ok(()),
        }
    }
}
