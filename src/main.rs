mod app;
mod backend;
mod filter;
mod keymap;
mod ui;
mod utils;

use std::{error::Error, time::Duration};

use backend::run;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, default_value_t = 250)]
    tick_rate: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let tick_rate = Duration::from_millis(args.tick_rate);
    run(tick_rate)?;
    Ok(())
}
