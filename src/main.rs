use anyhow::Result;
use clap::Parser;
use con::commands::{pull, run};
use std::str;

#[derive(Parser, Debug)]
#[clap(author, about, version)]
enum Opt {
    Pull(pull::Pull),
    Run(run::Run),
}

fn main() -> Result<()> {
    let opt = Opt::parse();

    match opt {
        Opt::Pull(pull) => pull.exec(),
        Opt::Run(run) => run.exec(),
    }
}
