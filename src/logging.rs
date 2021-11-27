use termcolor::{
    Buffer,
    Color::{self, *},
};
use termcolor_output::colored;

use crate::entry::{Entry, Expected};
use crate::normalize;
use crate::term;

use std::io;
use std::path::Path;

pub(crate) enum Level {
    Fail,
    Warn,
}

pub(crate) use self::Level::*;

// This function is called if there are no entries, so it will create the buffer itself,
// not expecting it from caller.
pub(crate) fn no_entries() -> io::Result<Buffer> {
    let mut out = term::buf();
    colored!(
        out,
        "{}{}No entries were provided to runner. Maybe the files are not created yet, or the glob path is wrong.\n{}",
        reset!(), fg!(Some(Yellow)), reset!()
    )?;
    Ok(out)
}

pub(crate) fn ok(buf: &mut Buffer) -> io::Result<()> {
    colored!(buf, "{}ok{}\n", fg!(Some(Green)), reset!())
}

pub(crate) fn log_entry_start(entry: &Entry, log: &mut Buffer) -> io::Result<()> {
    let display_name = entry
        .path()
        .file_name()
        .unwrap_or_else(|| entry.path().as_os_str())
        .to_string_lossy();

    let expected = match entry.expected() {
        Expected::RunMatch => " [should run and generate output]",
        Expected::CompileFail => " [should fail to compile]",
    };

    write_entry_header(log, &display_name, expected)
}

pub(crate) fn log_entry_fail_to_start(entry: &Entry, buf: &mut Buffer) -> io::Result<()> {
    write_entry_header(buf, &entry.path().as_os_str().to_string_lossy(), "")
}

fn write_entry_header(buf: &mut Buffer, name: &str, expected: &str) -> io::Result<()> {
    colored!(
        buf,
        "{}batch entry {}{}{}{} ... ",
        reset!(),
        bold!(true),
        name,
        bold!(false),
        expected
    )
}

pub(crate) fn log_wip_write(
    buf: &mut Buffer,
    wip_path: &Path,
    path: &Path,
    string: &str,
) -> io::Result<()> {
    let wip_path = wip_path.to_string_lossy();
    let path = path.to_string_lossy();

    colored!(
        buf,
        "{}{}wip\n\nNOTE{}: writing the following output to `{}`.\nMove this file to {} to accept it as correct.\n",
        reset!(),
        fg!(Some(Yellow)),
        reset!(),
        wip_path,
        path,
    )?;
    snippet(buf, Yellow, string)?;
    colored!(buf, "\n")
}

pub(crate) fn log_overwrite(buf: &mut Buffer, path: &Path, string: &str) -> io::Result<()> {
    let path = path.to_string_lossy();

    colored!(
        buf,
        "{}{}wip\n\nNOTE{}: writing the following output to {}.\n",
        reset!(),
        fg!(Some(Yellow)),
        reset!(),
        path
    )?;
    snippet(buf, Yellow, string)?;
    colored!(buf, "\n")
}

/* TODO - I'll probably want to implement it on `Mismatch` itself
pub(crate) fn mismatch(expected: &str, actual: &str) {
    term::bold_color(Red);
    println!("mismatch");
    term::reset();
    println!();
    term::bold_color(Blue);
    println!("EXPECTED:");
    snippet(Blue, expected);
    println!();
    term::bold_color(Red);
    println!("ACTUAL OUTPUT:");
    snippet(Red, actual);
    println!();
}
*/

pub(crate) fn build_status_mismatch(log: &mut Buffer) -> io::Result<()> {
    colored!(log, "{}{}{}error: {}", reset!(), bold!(true), fg!(Some(Red)), bold!(false))
}

pub(crate) fn unexpected_build_success(log: &mut Buffer) -> io::Result<()> {
    build_status_mismatch(log)?;
    colored!(log, "Expected test case to fail to compile, but it succeeded.{}\n", reset!())
}

pub(crate) fn unexpected_build_error(log: &mut Buffer, error: &[u8]) -> io::Result<()> {
    build_status_mismatch(log)?;
    colored!(log, "Entry failed to build; compiler output:{}\n", reset!())?;
    snippet(log, Red, &normalize::trim(error))
}

pub(crate) fn log_snapshot(log: &mut Buffer, level: Level, snapshot: &[u8]) -> io::Result<()> {
    let color = match level {
        Fail => Red,
        Warn => Yellow,
    };

    if !snapshot.is_empty() {
        colored!(log, "{}{}STDOUT:", bold!(true), fg!(Some(color)))?;
        snippet(log, color, &normalize::trim(snapshot))?;
        colored!(log, "\n")?;
    }
    Ok(())
}

fn snippet(log: &mut Buffer, color: Color, content: &str) -> io::Result<()> {
    let dotted_line = "┈".repeat(60);

    colored!(log, "{}{}{}\n", reset!(), fg!(Some(color)), dotted_line)?;

    // Color one line at a time because Travis does not preserve color setting
    // across output lines.
    for line in content.lines() {
        colored!(log, "{}{}\n", fg!(Some(color)), line)?;
    }

    colored!(log, "{}{}{}\n", fg!(Some(color)), dotted_line, reset!())
}
