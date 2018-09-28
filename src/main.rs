extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate rustyline;

use pest::Parser;
use rustyline::error::ReadlineError;
use rustyline::Editor;

// First get the calculator working

#[cfg(debug_assertions)]
const _GRAMMAR: &str = include_str!("grammar.pest");

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct BlisprParser;

// PRINT

//fn print_ast()

fn main() {
    println!("Blispr v0.0.1");
    println!("Press Ctrl-C or Ctrl-D to exit");

    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No history found.");
    }

    loop {
        let input = rl.readline("blispr> ");
        match input {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                let ast = BlisprParser::parse(Rule::blispr, &line)
                    .expect("Gibberish!  Try some real blispr next time")
                    .next()
                    .unwrap();
                // match type - for now im leaving out the recursive ones
                println!("{:?}", ast);
                break;
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
