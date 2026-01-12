//! Compile-fail tests using trybuild.
//!
//! These tests verify that the kwarg macros produce helpful error messages
//! when used incorrectly.

#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
