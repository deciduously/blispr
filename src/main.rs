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

use parse::repl;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "blispr")]
struct Opt {
    /// debug mode
    #[structopt(short = "d", long = "debug")]
    debug: bool,
}

fn main() {
    let opt = Opt::from_args();
    // enable debug output if needed
    if opt.debug {
        ::std::env::set_var("RUST_LOG", "blispr=debug");
    }

    pretty_env_logger::init();

    if let Err(e) = repl() {
        eprintln!("Error: {}", e);
        ::std::process::exit(1)
    }
}
