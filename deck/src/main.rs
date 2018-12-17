#![forbid(unsafe_code)]

extern crate deck_client;
#[macro_use]
extern crate structopt;

use std::process::Command;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "deck")]
pub struct Opt {
    // The number of occurences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,
    #[structopt(subcommand)]
    cmd: Subcommand,
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
    /// Builds a Deck package
    #[structopt(name = "build")]
    Build,
}

fn main() {
    let opt = Opt::from_args();
}
