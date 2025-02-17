use crate::cargo_rustc;
use crate::result::BatchResult;
use lazy_static::lazy_static;
use rand::random;
use std::{
    env::var_os,
    ffi::OsString,
    fs::{create_dir, remove_dir, remove_file, write},
    ops::Not,
    path::{Path, PathBuf},
    process::Command,
};

lazy_static! {
    static ref BIN_DIR: PathBuf = [
        var_os("CARGO_MANIFEST_DIR").unwrap(),
        OsString::from("src"),
        OsString::from("bin")
    ]
    .iter()
    .collect();
    pub static ref BUILDER: BinaryBuilder = BinaryBuilder::new().unwrap();
}

fn new() -> BatchResult<(String, bool)> {
    let bin_needed = BIN_DIR.exists().not();
    let bin_created = bin_needed && create_dir(&*BIN_DIR).map(|_| true)?;
    let mut name = "batch_runner_check_".to_owned();
    loop {
        // it is _still_ possible to break this somehow... but let's assume it doesn't
        if BIN_DIR.join(&name).with_extension("rs").exists() {
            name.push_str(&format!("{:x}", random::<u8>()));
        } else {
            break;
        }
    }
    write(BIN_DIR.join(&name).with_extension("rs"), b"fn main() {}")?;
    Ok((name, bin_created))
}

fn into_builder(name: &str) -> BatchResult<BinaryBuilder> {
    let cmd = cargo_rustc::capture_build_command(name)?;

    let args = cmd
        .split_ascii_whitespace()
        .scan("", |prev, cur| {
            let out = if prev == &"-L" {
                vec!["-L", cur]
            } else if prev == &"--cfg" {
                vec!["--cfg", cur]
            } else if prev == &"--extern" {
                vec!["--extern", cur]
            } else if cur.starts_with("--edition") {
                vec![cur]
            } else {
                vec![]
            };
            *prev = cur;
            Some(out)
        })
        .flatten()
        .map(String::from)
        .collect();

    Ok(BinaryBuilder { args })
}
fn drop(name: &str, bin_created: bool) {
    remove_file(BIN_DIR.join(name).with_extension("rs")).unwrap_or_else(|_| {
        eprintln!(
            "Unable to remove temporary file {}, please check it and remove manually",
            name
        )
    });
    if bin_created {
        remove_dir(&*BIN_DIR).unwrap_or_else(|_| {
            eprintln!(
                "Unable to remove directory {}, please check it and remove manually",
                BIN_DIR
                    .to_str()
                    .unwrap_or("[ERROR BUILDING BIN_DIR PATH, PLEASE CONTACT US!]")
            )
        });
    }
}

pub struct BinaryBuilder {
    args: Vec<String>,
}

impl BinaryBuilder {
    pub fn new() -> BatchResult<Self> {
        let (name, bin_created) = new()?;
        let builder = into_builder(&name);
        drop(&name, bin_created);
        builder
    }
    pub fn args_to_command(&self, cmd: &mut Command, main: &Path) {
        cmd.args(&self.args).arg(main);
    }
}
