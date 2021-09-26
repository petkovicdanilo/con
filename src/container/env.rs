use std::str::FromStr;

use anyhow::Result;

#[derive(Debug)]
pub struct EnvVariable {
    pub key: String,
    pub value: String,
}

impl FromStr for EnvVariable {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once("=") {
            Some((key, value)) => Ok(Self {
                key: key.to_string(),
                value: value.to_string(),
            }),
            None => Err("Invalid volume syntax. Expected in format 'key=value'"),
        }
    }
}

pub fn set_variables<'a, I>(variables: I) -> Result<()>
where
    I: Iterator<Item = &'a EnvVariable>,
{
    for variable in variables {
        std::env::set_var(&variable.key, &variable.value);
    }

    Ok(())
}
