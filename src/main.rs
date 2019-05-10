#[macro_use]
extern crate pest_derive;

mod error;
mod eval;
mod lenv;
mod lval;
mod parse;
mod run;

#[cfg(test)]
mod test;

use crate::run::run;
use std::{path::PathBuf, process::exit};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "blispr")]
pub struct Opt {
    /// debug mode
    #[structopt(short = "d", long = "debug")]
    debug: bool,
    /// input file
    #[structopt(short = "i", long = "input")]
    input: Option<PathBuf>,
}

fn main() {
    if let Err(e) = run(Opt::from_args()) {
        eprintln!("Error: {}", e);
        exit(1);
    }
}
