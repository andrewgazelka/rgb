//! Compile-time tests for the Component derive macro.
//!
//! These tests verify that:
//! 1. Valid components compile successfully
//! 2. Invalid components (with forbidden types) fail with helpful errors

#[test]
fn test_compile_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail_*.rs");
    t.pass("tests/ui/pass_*.rs");
}
