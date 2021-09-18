mod commands;

use anyhow::Result;
use clap::{crate_version, Clap};
use commands::run;

/// con - simple program to ilustrate containers in Rust
#[derive(Clap, Debug)]
#[clap(version = crate_version!())]
struct Opt {
    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    Run(run::Run),
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    match opt.subcommand {
        SubCommand::Run(run) => run.exec(),
    }
}
