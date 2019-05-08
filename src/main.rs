#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate log;

mod error;
mod eval;
mod lenv;
mod lval;
mod parse;

use crate::{error::Result, lenv::Lenv, parse::eval_str};
use rustyline::{error::ReadlineError, Editor};
use std::{
    env::set_var,
    fs::File,
    io::{BufReader, Read},
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

pub fn repl(e: &mut Lenv) -> Result<()> {
    println!("Blispr v0.0.1");
    println!("Use exit(), Ctrl-C, or Ctrl-D to exit prompt");
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
                // if eval_str is an error, we want to catch it here, inside the loop, but still show the next prompt
                // just using ? would bubble it up to main()
                if let Err(err) = eval_str(e, &line) {
                    eprintln!("{}", err);
                }
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

fn run(opt: Opt) -> Result<()> {
    // enable debug output if needed
    if opt.debug {
        set_var("RUST_LOG", "blispr=debug");
    }

    pretty_env_logger::init();
    // First arg is optional lookup map, second is optional parent env
    // The root env starts empty (except for builtins) and has no parent
    let global_env = &mut Lenv::new(None, None);

    if let Some(f) = opt.input {
        // if input file passed, eval its contents
        let file = File::open(f)?;
        let mut bfr = BufReader::new(file);
        let mut program = String::new();
        bfr.read_to_string(&mut program)?;
        eval_str(global_env, &program)?
    } else {
        repl(global_env)?
    }
    Ok(())
}

fn main() {
    if let Err(e) = run(Opt::from_args()) {
        eprintln!("Error: {}", e);
        exit(1);
    }
}
