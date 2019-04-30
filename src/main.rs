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

use crate::error::Result;
use parse::eval_str;
use rustyline::{error::ReadlineError, Editor};
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

pub fn repl() -> Result<()> {
    println!("Blispr v0.0.1");
    println!("Press Ctrl-C or Ctrl-D or use exit() to exit prompt");
    debug!("Debug mode enabled");

    let mut rl = Editor::<()>::new();
    if rl.load_history("./.blispr-history.txt").is_err() {
        println!("No history found.");
    }

    loop {
        let input = rl.readline("blispr> ");

        match input {
            Ok(line) => {
                rl.add_history_entry(line.as_ref());
                eval_str(&line)?;
            }
            Err(ReadlineError::Interrupted) => {
                info!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                info!("CTRL-D");
                break;
            }
            Err(err) => {
                warn!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("./.blispr-history.txt")?;
    Ok(())
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
