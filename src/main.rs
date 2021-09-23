use anyhow::Result;
use clap::Clap;
use con::commands::{pull, run};

/// con - simple program to ilustrate containers in Rust
#[derive(Clap, Debug)]
#[clap(author, about, version)]
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
