use crate::{
    config::Update,
    logging,
    mismatch::{CompileFailMismatch, LocalOutput, RunMismatch, match_with_backslashes},
    normalize::diagnostics,
    result::{
        error::NoExpected,
        error::{EntryError, EntryFailed},
        EntryResult,
    },
};
use ron::{
    de::from_str,
    ser::{to_string_pretty, PrettyConfig},
};
use std::path::Path;
use std::{
    convert::{Infallible, TryInto},
    fs::{create_dir_all, read_to_string, write},
    process::Output,
};
use termcolor::WriteColor;

pub fn check_compile_fail(
    path: &Path,
    output: Output,
    update_mode: Update,
    log: &mut impl WriteColor,
) -> EntryResult<()> {
    // early exit if the entry has indeed compiled
    if output.status.success() {
        logging::unexpected_build_success(log)?;
        return Err(EntryFailed::ShouldNotCompile);
    }

    let variations = diagnostics(&output.stderr);
    let preferred = variations.preferred();
    // In this case, the expected output is simply a string - let's read it!
    let stderr_path = path.with_extension("stderr");

    // But first, check if it ever exists...
    if !stderr_path.exists() {
        // logging::fail_output(log, Warn, &build_stdout);

        // both write_wip and write_overwrite are "always-fallible", and this is statically guaranteed
        // so we know, that this branch will always return early
        // with stabilization of "never" type, we can guarantee this here, too
        // but for now, just trust us
        // (joking... you can always check the signatures)
        match update_mode {
            Update::Wip => write_wip(&stderr_path, preferred, log)?,
            Update::Overwrite => write_overwrite(&stderr_path, preferred, log)?,
        };
    }

    // ok, well - the file does exist, but does it contain the same that we've got?
    let expected = read_to_string(&stderr_path)
        .map_err(EntryError::ReadExpected)?
        .replace("\r\n", "\n");

    if variations.any(|stderr| match_with_backslashes(&expected, &stderr)) {
        return Ok(());
    }

    match update_mode {
        Update::Wip => {
            logging::mismatch(log, &expected, &preferred)?;
            Err(EntryFailed::CompileFailMismatch(CompileFailMismatch::new(
                expected, preferred,
            )))
        }
        Update::Overwrite => write_overwrite(&stderr_path, preferred, log).map(|_| ()),
    }
}

pub fn check_run_match(
    path: &Path,
    output: Output,
    update_mode: Update,
    log: &mut impl WriteColor,
) -> EntryResult<()> {
    // TODO propagate error
    let output: LocalOutput = output.try_into().expect("No status code");

    // In this case, the expected output is the file representing the output - let's read it!
    let snapshot_path = path.with_extension("snapshot");

    // But first, check if it ever exists...
    if !snapshot_path.exists() {
        // logging::fail_output(log, Warn, &build_stdout);

        let data =
            to_string_pretty(&output, PrettyConfig::default()).expect("Serialization failed");
        // both write_wip and write_overwrite are "always-fallible", and this is statically guaranteed
        // so we know, that this branch will always return early
        // with stabilization of "never" type, we can guarantee this here, too
        // but for now, just trust us
        // (joking... you can always check the signatures)
        match update_mode {
            Update::Wip => write_wip(&snapshot_path, &data, log)?,
            Update::Overwrite => write_overwrite(&snapshot_path, &data, log)?,
        };
    }

    // ok, well - the file does exist, but does it contain the same that we've got?
    let string = &read_to_string(&snapshot_path)
        .map_err(EntryError::ReadExpected)?
        .replace("\r\n", "\n");
    let expected = from_str(string).expect("Deserialization failed");

    if output.matches(&expected) {
        return Ok(());
    }

    let data = to_string_pretty(&output, PrettyConfig::default()).expect("Serialization failed");
    match update_mode {
        Update::Wip => {
            logging::mismatch(log, &string, &data)?;
            Err(EntryFailed::RunMismatch(RunMismatch::new(expected, output)))
        }
        Update::Overwrite => {
            // TODO propagate the serialization-deserialization errors
            write_overwrite(&snapshot_path, &data, log).map(|_| ())
        }
    }
}

fn write_wip(path: &Path, content: &str, log: &mut impl WriteColor) -> EntryResult<Infallible> {
    let wip_dir = Path::new("wip");
    create_dir_all(wip_dir)?;

    let gitignore_path = wip_dir.join(".gitignore");
    write(gitignore_path, "*\n")?;

    let stderr_name = path
        .file_name()
        .expect("Failed to write expected content to WIP folder - corrupt path");
    let wip_path = wip_dir.join(stderr_name);
    logging::log_wip_write(log, &wip_path, &path, content)?;

    write(wip_path, content).map_err(EntryError::WriteExpected)?;

    Err(EntryFailed::ExpectedNotExist(NoExpected::ToWip(
        content.to_owned(),
    )))
}

fn write_overwrite(
    path: &Path,
    content: &str,
    log: &mut impl WriteColor,
) -> EntryResult<Infallible> {
    logging::log_overwrite(log, &path, content)?;

    write(path, content).map_err(EntryError::WriteExpected)?;

    Err(EntryFailed::ExpectedNotExist(NoExpected::Direct(
        content.to_owned(),
    )))
}
