#[test]
fn test() {
    let t = batch_run::Batch::new();
    // t.run_match("tests/ui/run-pass-0.rs");
    // t.run_match("tests/ui/print-stdout.rs");
    // t.run_match("tests/ui/run-pass-1.rs");
    // t.run_match("tests/ui/print-stderr.rs");
    // t.run_match("tests/ui/run-pass-2.rs");
    // t.run_match("tests/ui/print-both.rs");
    // t.run_match("tests/ui/run-pass-4.rs");
    // t.run_match("tests/ui/run-pass-3.rs");
    // t.run_match("tests/ui/run-pass-5.rs");
    // t.compile_fail("tests/ui/compile-fail-0.rs");
    // t.run_match("tests/ui/run-pass-6.rs");
    // t.run_match("tests/ui/run-pass-7.rs");
    // t.run_match("tests/ui/run-pass-8.rs");
    // t.compile_fail("tests/ui/compile-fail-1.rs");
    // //    t.run_match("tests/ui/run-fail.rs"); - run-fail?
    // t.run_match("tests/ui/run-pass-9.rs");
    t.compile_fail("tests/ui/compile-fail-2.rs");
    t.run().unwrap().assert_all_ok();
}
