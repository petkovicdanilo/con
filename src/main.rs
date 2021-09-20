mod commands;
mod image;
mod util;

use anyhow::Result;
use clap::{crate_version, Clap};
use commands::{pull, run};

/// con - simple program to ilustrate containers in Rust
#[derive(Clap, Debug)]
#[clap(version = crate_version!())]
enum Opt {
    Pull(pull::Pull),
    Run(run::Run),
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::parse();

    match opt {
        Opt::Pull(pull) => pull.exec().await,
        Opt::Run(run) => run.exec().await,
    }
}
