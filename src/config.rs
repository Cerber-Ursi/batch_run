use crate::batch_result::{BatchError, BatchResult};
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
    fn env() -> BatchResult<Self> {
        let var = match env::var_os("BATCH_RUN") {
            Some(var) => var,
            None => return Ok(Update::default()),
        };

        match var.as_os_str().to_str() {
            Some("wip") => Ok(Update::Wip),
            Some("overwrite") => Ok(Update::Overwrite),
            _ => Err(BatchError::UpdateVar(var))?,
        }
    }
}

#[derive(Default)]
pub struct Config {
    update_mode: Update,
}

impl Config {
    pub fn from_env() -> BatchResult<Self> {
        Ok(Self {
            update_mode: Update::env()?,
        })
    }
    #[allow(dead_code)]
    pub fn with_update_mode(update_mode: Update) -> Self {
        Self { update_mode }
    }
}
