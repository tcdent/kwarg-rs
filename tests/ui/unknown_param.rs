//! Test that unknown parameters produce a compile error.

use kwarg::{kwarg, kwargs};

#[kwarg]
fn greet(name: &str, age: u32) -> String {
    format!("Hello {}, you are {}", name, age)
}

fn main() {
    // 'unknown' is not a valid parameter - should fail
    let _ = kwargs!(greet =>
        name: "Alice",
        age: 30,
        unknown: "value"
    );
}
