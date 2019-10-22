fn main() {
    let b = batch_run::Batch::new();
    b.compile_fail("tests/ui-cases/compile-*.rs");
    b.run_match("tests/ui-cases/run-*.rs");
    b.run().unwrap().assert_all_ok();
}
