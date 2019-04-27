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
    pretty_env_logger::init();
    // set "-p" to also print out the parsed blipsr, pre-eval
    // first check if we have a flag at all
    let print_parsed = {
        let args = &::std::env::args().collect::<Vec<String>>();

        if args.len() == 1 {
            false
        } else {
            args[1] == "-p"
        }
    };

    if let Err(e) = repl(print_parsed) {
        eprintln!("Error: {}", e);
        ::std::process::exit(1)
    }
}
