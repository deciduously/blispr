#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod error;
mod eval;
mod lenv;
mod lval;
mod parse;

use parse::{eval_str, repl};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::exit,
};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "blispr")]
struct Opt {
    /// debug mode
    #[structopt(short = "d", long = "debug")]
    debug: bool,
    /// input file
    #[structopt(short = "i", long = "input")]
    input: Option<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();
    // enable debug output if needed
    if opt.debug {
        ::std::env::set_var("RUST_LOG", "blispr=debug");
    }

    pretty_env_logger::init();

    if let Some(f) = opt.input {
        // if input file passed, eval that
        let file = File::open(f).unwrap();
        let bfr = BufReader::new(file);
        for line in bfr.lines() {
            if let Err(e) = eval_str(&line.unwrap()) {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
    } else if let Err(e) = repl() {
        // otherwise start a REPL
        eprintln!("Error: {}", e);
        exit(1);
    }
}
