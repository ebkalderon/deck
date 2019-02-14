#![forbid(unsafe_code)]

use std::process;

use structopt::StructOpt;

use commands::{GlobalFlags, Subcommand};

mod commands;

/// Declarative, reproducible, and functional package manager
#[derive(Debug, StructOpt)]
#[structopt(name = "deck", after_help = "Foo")]
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
