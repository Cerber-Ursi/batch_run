use crate::batch_result::{Error, Result};
use std::env;

#[derive(PartialEq, Debug)]
pub enum Update {
    Wip,
    Overwrite,
}

impl Default for Update {
    fn default() -> Self {
        Update::Wip
    }
}

impl Update {
    fn env() -> Result<Self> {
        let var = match env::var_os("BATCH_RUN") {
            Some(var) => var,
            None => return Ok(Update::default()),
        };

        match var.as_os_str().to_str() {
            Some("wip") => Ok(Update::Wip),
            Some("overwrite") => Ok(Update::Overwrite),
            _ => Err(Error::UpdateVar(var))?,
        }
    }
}

pub struct Config {
    update_mode: Update,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            update_mode: Update::env()?,
        })
    }
}
