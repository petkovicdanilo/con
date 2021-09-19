use anyhow::Result;
use clap::Clap;

#[derive(Clap, Debug)]
pub struct Pull {
    image: String,
}

impl Pull {
    pub fn exec(&self) -> Result<()> {
        Ok(())
    }
}
