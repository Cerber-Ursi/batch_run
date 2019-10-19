// use std::sync::{PoisonError, RwLock, RwLockWriteGuard};
use crate::result::error::PrintError;

use lazy_static::lazy_static;
use termcolor::{
    Buffer,
    BufferWriter,
    ColorChoice,
    // StandardStream
};

lazy_static! {
    static ref WRITER: BufferWriter = BufferWriter::stdout(ColorChoice::Auto);
    // static ref TERM: RwLock<StandardStream> =
    //     RwLock::new(StandardStream::stdout(ColorChoice::Auto));
}

pub fn buf() -> Buffer {
    WRITER.buffer()
}

pub fn print(buf: Option<Buffer>) -> Result<(), PrintError> {
    buf.ok_or(PrintError::AlreadyPrinted)
        .and_then(|buf| WRITER.print(&buf).map_err(PrintError::Io))
}

// pub fn direct<'a>() -> RwLockWriteGuard<'a, StandardStream> {
//     TERM.write().unwrap_or_else(PoisonError::into_inner)
// }
