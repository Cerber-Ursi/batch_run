use lazy_static::lazy_static;
use termcolor::{Buffer, BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

lazy_static! {
    static ref TERM: BufferWriter = BufferWriter::stderr(ColorChoice::Auto);
}

pub fn buf() -> Buffer {
    TERM.buffer()
}

pub fn bold(buf: &mut Buffer) {
    let _ = buf.set_color(ColorSpec::new().set_bold(true));
}

pub fn color(buf: &mut Buffer, color: Color) {
    let _ = buf.set_color(ColorSpec::new().set_fg(Some(color)));
}

pub fn bold_color(buf: &mut Buffer, color: Color) {
    let _ = buf.set_color(ColorSpec::new().set_bold(true).set_fg(Some(color)));
}

pub fn reset(buf: &mut Buffer) {
    let _ = buf.reset();
}
