use crate::{
    batch_result::{EntryError, EntryFailed, EntryResult, NoExpected},
    config::Update,
    mismatch::{CompileFailMismatch, RunMismatch, LocalOutput},
    normalize::diagnostics,
};
use std::path::Path;
use std::{
    convert::{Infallible, TryInto},
    fs::{create_dir_all, read_to_string, write},
    process::Output,
};
use ron::{ser::{to_string_pretty, PrettyConfig}, de::from_str};

pub fn check_compile_fail(path: &Path, output: Output, update_mode: Update) -> EntryResult<()> {
    // early exit if the entry has indeed compiled
    if output.status.success() {
        Err(EntryFailed::ShouldNotCompile)?;
    }

    let variations = diagnostics(&output.stderr);
    let preferred = variations.preferred();
    // In this case, the expected output is simply a string - let's read it!
    let stderr_path = path.with_extension("stderr");

    // But first, check if it ever exists...
    if !stderr_path.exists() {
        // message::fail_output(Warn, &build_stdout);

        // both write_wip and write_overwrite are "always-fallible", and this is statically guaranteed
        // so we know, that this branch will always return early
        // with stabilization of "never" type, we can guarantee this here, too
        // but for now, just trust us
        // (joking... you can always check the signatures)
        match update_mode {
            Update::Wip => write_wip(&stderr_path, preferred)?,
            Update::Overwrite => write_overwrite(&stderr_path, preferred)?,
        };
    }

    // ok, well - the file does exist, but does it contain the same that we've got?
    let expected = read_to_string(&stderr_path)
        .map_err(EntryError::ReadExpected)?
        .replace("\r\n", "\n");

    if variations.any(|stderr| expected == stderr) {
        // message::ok();
        return Ok(());
    }

    match update_mode {
        Update::Wip => {
            // message::mismatch(&expected, preferred);
            Err(EntryFailed::CompileFailMismatch(CompileFailMismatch::new(
                expected, preferred,
            )))
        }
        Update::Overwrite => {
            // note that we can't move this out of the block, due to the types mismatch
            write_overwrite(&stderr_path, preferred).map(|_| ())
        }
    }
}

pub fn check_run_match(path: &Path, output: Output, update_mode: Update) -> EntryResult<()> {
    // TODO propagate error
    let output: LocalOutput = output.try_into().expect("No status code");

    // In this case, the expected output is the file representing the output - let's read it!
    let snapshot_path = path.with_extension("snapshot");

    // But first, check if it ever exists...
    if !snapshot_path.exists() {
        // message::fail_output(Warn, &build_stdout);

        let data = to_string_pretty(&output, PrettyConfig::default()).expect("Serialization failed");
        // both write_wip and write_overwrite are "always-fallible", and this is statically guaranteed
        // so we know, that this branch will always return early
        // with stabilization of "never" type, we can guarantee this here, too
        // but for now, just trust us
        // (joking... you can always check the signatures)
        match update_mode {
            Update::Wip => write_wip(&snapshot_path, &data)?,
            Update::Overwrite => write_overwrite(&snapshot_path, &data)?,
        };
    }

    // ok, well - the file does exist, but does it contain the same that we've got?
    let expected = from_str(&read_to_string(&snapshot_path)
        .map_err(EntryError::ReadExpected)?
        .replace("\r\n", "\n")).expect("Deserializing failed");

    if expected == output {
        // message::ok();
        return Ok(());
    }

    match update_mode {
        Update::Wip => {
            // message::mismatch(&expected, preferred);
            Err(EntryFailed::RunMismatch(RunMismatch::new(expected, output)))?;
        }
        Update::Overwrite => {
            // note that we can't move this out of the block, due to the types mismatch
            write_overwrite(&snapshot_path, &to_string_pretty(&output, PrettyConfig::default()).expect("Serialization failed"))?;
        }
    };

    Ok(())
}

fn write_wip(path: &Path, content: &str) -> EntryResult<Infallible> {
    let wip_dir = Path::new("wip");
    create_dir_all(wip_dir)?;
    let gitignore_path = wip_dir.join(".gitignore");
    write(gitignore_path, "*\n")?;
    let stderr_name = path
        .file_name()
        .expect("Failed to write expected content WIP folder - corrupt path");
    let wip_path = wip_dir.join(stderr_name);
    // message::write_stderr_wip(&wip_path, &stderr_path, preferred);
    write(wip_path, content).map_err(EntryError::WriteExpected)?;
    Err(EntryFailed::ExpectedNotExist(NoExpected::ToWip(
        content.to_owned(),
    )))
}

fn write_overwrite(path: &Path, content: &str) -> EntryResult<Infallible> {
    // message::overwrite_stderr(&stderr_path, preferred);
    write(path, content).map_err(EntryError::WriteExpected)?;
    Err(EntryFailed::ExpectedNotExist(NoExpected::Direct(
        content.to_owned(),
    )))?
}
