#[macro_use]
extern crate pest_derive;

mod eval;
mod lval;
mod parse;

use parse::repl;

fn main() {
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

    repl(print_parsed);
}
