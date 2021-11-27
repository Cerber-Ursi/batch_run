Batch Run
=========

[![Latest Version](https://img.shields.io/crates/v/batch_run.svg)](https://crates.io/crates/batch_run)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/batch_run)

`batch_run` is a runner for a set of Rust source files, based on [dtolnay's `trybuild`](https://github.com/dtolnay/trybuild).
It can be useful when you have a bunch of Rust sources which are not complex enough to be
packed into dedicated crates, but which are (by their meaning) not just integration test cases.
It also checks for output correctness, either on compile-time (for `compile_fail` cases)
or at runtime (for `run_pass` cases).

```toml
[dependencies]
batch_run = "1.0"
```

*Compiler support: requires rustc 1.31+*

<br>

## Compile-fail cases

A minimal batch_run setup looks like this:

```rust
fn main() {
    let b = batch_run::Batch::new();
    b.compile_fail("batches/ui/*.rs");
    match b.run() {
        Ok(()) => {},
        Err(err) => println!("{:?}", err)
    };
}
```

This program will individually compile each of the
source files matching the glob pattern, expect them to fail to compile, and
assert that the compiler's error message matches an adjacently named _*.stderr_
file containing the expected output (same file name as the test except with a
different extension). If it doesn't match, the program will print the error message
with expected vs actual compiler output.

Dependencies listed under `[dependencies]` in the project's Cargo.toml are
accessible from within the batch.

A compile\_fail case that fails to fail to compile is also a failure.

<br>

## Run-pass cases

In the run_pass cases, we not only check that the code compiles, but also actually run it
and match the stdout/stderr output with the corresponding _*.stdout_/_*.stderr_ files.

You can mix compile_faiil and run_pass cases in one batch:

```rust
fn main() {
    let t = batch_run::Batch::new();
    t.run_pass("batches/01-parse-header.rs");
    t.run_pass("batches/02-parse-body.rs");
    t.compile_fail("batches/03-expand-four-errors.rs");
    t.run_pass("batches/04-paste-ident.rs");
    t.run_pass("batches/05-repeat-section.rs");
}
```

<br>

## Details

That's the entire API.

<br>

## Workflow

(TODO)

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
