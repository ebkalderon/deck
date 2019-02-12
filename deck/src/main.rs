#![forbid(unsafe_code)]

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "deck")]
struct Opt {
    // The number of occurences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,
    #[structopt(subcommand)]
    cmd: Subcommand,
}

#[derive(Debug, StructOpt)]
enum Subcommand {
    /// Builds a Deck package
    #[structopt(name = "build")]
    Build,
}

fn main() {
    let _opt = Opt::from_args();
}
