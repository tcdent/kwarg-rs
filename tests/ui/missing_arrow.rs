//! Test that missing => separator produces a compile error.

use kwarg::{kwarg, kwargs};

#[kwarg]
fn greet(name: &str, age: u32) -> String {
    format!("Hello {}, you are {}", name, age)
}

fn main() {
    // Missing '=>' separator - should fail
    let _ = kwargs!(greet
        name: "Alice",
        age: 30
    );
}
