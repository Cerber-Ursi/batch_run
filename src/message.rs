#![allow(dead_code)]

use termcolor::{
    Buffer,
    Color::{self, *},
};
use termcolor_output::colored;

use crate::result::error::EntryFailed;
use crate::normalize;
use crate::term;
use crate::entry::{Entry, Expected};

use std::io;
use std::path::Path;

pub(crate) enum Level {
    Fail,
    Warn,
}

pub(crate) use self::Level::*;

pub(crate) fn entry_failed(buf: &mut Buffer, err: EntryFailed) -> io::Result<()> {
    colored!(
        buf,
        "{}{}error\n{}{}\n",
        bold!(true),
        fg!(Some(Red)),
        bold!(false),
        err
    )
}

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
    colored!(buf, "{}ok\n", fg!(Some(Green)))
}

pub(crate) fn begin_entry(entry: &Entry, buf: &mut Buffer, show_expected: bool) -> io::Result<()> {
    let display_name = if show_expected {
        entry
            .path()
            .file_name()
            .unwrap_or_else(|| entry.path().as_os_str())
            .to_string_lossy()
    } else {
        entry.path().as_os_str().to_string_lossy()
    }
    .to_string();
    let expected = if show_expected {
        match entry.expected() {
            Expected::RunMatch => " [should run and generate output]",
            Expected::CompileFail => " [should fail to compile]",
        }
    } else {
        ""
    };

    colored!(
        buf,
        "{}batch entry {}{}{}{}",
        reset!(),
        bold!(true),
        display_name,
        bold!(false),
        expected
    )
}

pub(crate) fn write_stderr_wip(
    buf: &mut Buffer,
    wip_path: &Path,
    stderr_path: &Path,
    stderr: &str,
) -> io::Result<()> {
    let wip_path = wip_path.to_string_lossy();
    let stderr_path = stderr_path.to_string_lossy();

    colored!(
        buf,
        "{}{}wip\n\nNOTE{}: writing the following output to `{}`.\nMove this file to {} to accept it as correct.\n",
        reset!(),
        fg!(Some(Yellow)),
        reset!(),
        wip_path,
        stderr_path,
    )?;
    snippet(buf, Yellow, stderr)?;
    colored!(buf, "\n")
}

pub(crate) fn overwrite_stderr(
    buf: &mut Buffer,
    stderr_path: &Path,
    stderr: &str,
) -> io::Result<()> {
    let stderr_path = stderr_path.to_string_lossy();

    colored!(
        buf,
        "{}{}wip\n\nNOTE{}: writing the following output to {}.\n",
        reset!(),
        fg!(Some(Yellow)),
        reset!(),
        stderr_path
    )?;
    snippet(buf, Yellow, stderr)?;
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

pub(crate) fn fail_output(buf: &mut Buffer, level: Level, stdout: &[u8]) -> io::Result<()> {
    let color = match level {
        Fail => Red,
        Warn => Yellow,
    };

    if !stdout.is_empty() {
        colored!(buf, "{}{}STDOUT:", bold!(true), fg!(Some(color)))?;
        snippet(buf, color, &normalize::trim(stdout))?;
        colored!(buf, "\n")?;
    }
    Ok(())
}

fn snippet(buf: &mut Buffer, color: Color, content: &str) -> io::Result<()> {
    let dotted_line = "┈".repeat(60);

    colored!(buf, "{}{}{}", reset!(), fg!(Some(color)), dotted_line)?;

    // Color one line at a time because Travis does not preserve color setting
    // across output lines.
    for line in content.lines() {
        colored!(buf, "{}{}\n", fg!(Some(color)), line)?;
    }

    colored!(
        buf,
        "{}{}{}{}",
        reset!(),
        fg!(Some(color)),
        dotted_line,
        reset!()
    )
}
