extern crate deck_client;
#[macro_use]
extern crate structopt;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "deck build")]
pub struct Opt {
    // The number of occurences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,
}

fn main() {
    let _opt = Opt::from_args();
    println!("Build");
}
