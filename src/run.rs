use crate::{
    error::{BlisprResult, Result},
    lenv::Lenv,
    parse::eval_str,
    Opt,
};
use log::{debug, info, warn};
use rustyline::{error::ReadlineError, Editor};
use std::{
    env::set_var,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

fn print_eval_result(v: BlisprResult) {
    match v {
        Ok(res) => println!("{}", res),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn repl(e: &mut Lenv) -> Result<()> {
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
                print_eval_result(eval_str(e, &line));
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

fn file_contents(path: PathBuf) -> Result<String> {
    let file = File::open(path)?;
    let mut bfr = BufReader::new(file);
    let mut program = String::new();
    bfr.read_to_string(&mut program)?;
    Ok(program)
}

pub fn run(opt: Opt) -> Result<()> {
    // enable debug output if needed
    if opt.debug {
        set_var("RUST_LOG", "blispr=debug");
    }
    pretty_env_logger::init();

    // Initialize global environment
    // First arg is optional lookup map, second is optional parent env
    // The root env starts empty (except for builtins) and has no parent
    let global_env = &mut Lenv::new(None, None);

    if let Some(f) = opt.input {
        // if input file passed, eval its contents
        print_eval_result(eval_str(global_env, &file_contents(f)?));
    } else {
        repl(global_env)?
    }
    Ok(())
}
