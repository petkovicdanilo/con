use std::{path::PathBuf, str::FromStr};

use anyhow::Result;

#[derive(Debug)]
pub struct Volume {
    pub source: PathBuf,
    pub destination: PathBuf,
}

impl FromStr for Volume {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(":") {
            Some((source, destination)) => {
                let source = match PathBuf::from_str(source) {
                    Ok(source) => source,
                    Err(_) => return Err("Source is not a path"),
                };

                let destination = match PathBuf::from_str(destination) {
                    Ok(destination) => destination,
                    Err(_) => return Err("Destination is not a path"),
                };

                Ok(Self {
                    source,
                    destination,
                })
            }
            None => Err("Invalid volume syntax. Expected in format 'source:destination'"),
        }
    }
}
