#[test]
fn basic() {
    let t = batch_run::Batch::new();
    t.compile_fail("tests/basic/compile-fail.rs");
    t.run_match("tests/basic/run-*.rs");
    t.run_match("tests/basic/print-*.rs");
    t.run().unwrap().assert_all_ok();
}

#[test]
fn ui() {
    let t = batch_run::Batch::new();
    t.run_match("tests/ui-runner/main.rs");
    t.run().unwrap().assert_all_ok();
}
