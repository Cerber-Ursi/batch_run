use crate::{
    batch_result::{EntryError, EntryFailed, EntryResult, NoExpected},
    config::Update,
    mismatch::{CompileFailMismatch, RunMismatch},
    normalize::diagnostics,
};
use std::path::Path;
use std::{
    convert::Infallible,
    fs::{create_dir_all, read_to_string, write, File},
    io::{BufReader, BufWriter},
    process::Output,
};
use xml::{
    reader::{EventReader, XmlEvent as ReadEvent},
    writer::{EventWriter, XmlEvent as WriteEvent},
};

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
    // early exit if the entry has not compiled
    if !output.status.success() {
        Err(EntryFailed::ShouldCompile)?;
    }

    // In this case, the expected output is an XML representing the output - let's read it!
    let snapshot_path = path.with_extension("snapshot");

    // But first, check if it ever exists...
    if !snapshot_path.exists() {
        // message::fail_output(Warn, &build_stdout);

        let xml = output.to_xml();
        // both write_wip and write_overwrite are "always-fallible", and this is statically guaranteed
        // so we know, that this branch will always return early
        // with stabilization of "never" type, we can guarantee this here, too
        // but for now, just trust us
        // (joking... you can always check the signatures)
        match update_mode {
            Update::Wip => write_wip(&snapshot_path, &xml)?,
            Update::Overwrite => write_overwrite(&snapshot_path, &xml)?,
        };
    }

    // ok, well - the file does exist, but does it contain the same that we've got?
    let expected = read_to_string(&snapshot_path)
        .map_err(EntryError::ReadExpected)?
        .replace("\r\n", "\n")
        .to_output();

    if expected == output {
        // message::ok();
        return Ok(());
    }

    match update_mode {
        Update::Wip => {
            // message::mismatch(&expected, preferred);
            Err(EntryFailed::RunMismatch(RunMismatch::new(
                expected, output,
            )))?;
        }
        Update::Overwrite => {
            // note that we can't move this out of the block, due to the types mismatch
            write_overwrite(&snapshot_path, &output.to_xml())?;
        }
    };

    Ok(())
}

// helper traits
trait FromXml {
    fn to_output(&self) -> Output;
}
trait ToXml {
    fn to_xml(&self) -> String;
}
impl FromXml for String {
    fn to_output(&self) -> Output {
        unimplemented!();
    }
}
impl ToXml for Output {
    fn to_xml(&self) -> String {
        unimplemented!();
    }
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
