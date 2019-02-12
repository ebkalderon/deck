#![forbid(unsafe_code)]

use std::process;

use structopt::StructOpt;

use commands::{CommonFlags, Subcommand};

mod commands;

/// Declarative, reproducible, and functional package manager
///
/// Deck is an ongoing work in progress.
#[derive(Debug, StructOpt)]
#[structopt(name = "deck")]
struct Opt {
    #[structopt(flatten)]
    common_flags: CommonFlags,
    #[structopt(subcommand)]
    command: Subcommand,
}

fn main() {
    let opt = Opt::from_args();

    if let Err(e) = opt.command.run(opt.common_flags) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
