//! Test that #[kwarg] on invalid items produces a compile error.

use kwarg::kwarg;

// #[kwarg] can only be applied to functions or impl blocks
#[kwarg]
struct InvalidStruct {
    field: i32,
}

fn main() {}
