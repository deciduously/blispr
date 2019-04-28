#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate log;

mod error;
mod eval;
mod lval;
mod parse;

use parse::repl;

fn main() {
    // set "-d" to enable debug! log level
    // first check if we have a flag at all
    let debug = {
        let args = &::std::env::args().collect::<Vec<String>>();

        if args.len() == 1 {
            false
        } else {
            args[1] == "-d"
        }
    };

    // enable debug output if needed
    if debug {
        ::std::env::set_var("RUST_LOG", "blispr=debug");
    }

    pretty_env_logger::init();

    if let Err(e) = repl() {
        eprintln!("Error: {}", e);
        ::std::process::exit(1)
    }
}
