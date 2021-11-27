use crate::result::{error::BatchError, error::ConfigError, BatchResult};
use std::{env, rc::Rc};
use termcolor::{Buffer, ColorChoice, StandardStream, WriteColor};

#[derive(PartialEq, Debug, Copy, Clone)]
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
            _ => Err(BatchError::ConfigError(ConfigError::UpdateEnvVar(var))),
        }
    }
}

pub struct WriterBuilder<W: WriteColor>(Rc<dyn Fn() -> W>);
impl<W: WriteColor> Clone for WriterBuilder<W> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<W: WriteColor> WriterBuilder<W> {
    pub fn new(inner: Box<dyn Fn() -> W>) -> Self
    where
        W: 'static,
    {
        Self(Rc::new(inner))
    }
    pub(crate) fn build(&self) -> W {
        self.0()
    }
}
impl Default for WriterBuilder<StandardStream> {
    fn default() -> Self {
        Self(Rc::new(|| StandardStream::stderr(ColorChoice::Always)))
    }
}
impl WriterBuilder<Buffer> {
    pub fn buffer() -> Self {
        Self(Rc::new(crate::term::buf))
    }
}

pub struct Config<W: WriteColor> {
    update_mode: Update,
    writer: WriterBuilder<W>,
}

impl Default for Config<StandardStream> {
    fn default() -> Self {
        Self {
            update_mode: Default::default(),
            writer: Default::default(),
        }
    }
}

impl Config<StandardStream> {
    pub fn from_env() -> BatchResult<Self> {
        Ok(Self {
            update_mode: Update::env()?,
            writer: WriterBuilder::default(),
        })
    }
}
impl<W: WriteColor> Config<W> {
    pub fn with_update_mode(self, update_mode: Update) -> Self {
        Self {
            update_mode,
            ..self
        }
    }
    pub fn update_mode(&self) -> Update {
        self.update_mode
    }
    pub fn with_writer<W2: WriteColor>(self, writer: WriterBuilder<W2>) -> Config<W2> {
        Config {
            writer,
            update_mode: self.update_mode,
        }
    }
    pub fn with_buffer(self) -> Config<Buffer> {
        Config { update_mode: self.update_mode, writer: WriterBuilder::buffer() }
    }
    pub fn writer(&self) -> WriterBuilder<W> {
        self.writer.clone()
    }
}
