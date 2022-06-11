#![warn(clippy::pedantic)]

use clap::Parser;
use std::{path::PathBuf, process::exit};

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

#[derive(clap::Parser)]
pub struct Opt {
	/// debug mode
	#[clap(short, long)]
	debug: bool,
	/// input file
	#[clap(short, long)]
	input: Option<PathBuf>,
}

fn main() {
	if let Err(e) = run(Opt::parse()) {
		eprintln!("Error: {}", e);
		exit(1);
	}
}
