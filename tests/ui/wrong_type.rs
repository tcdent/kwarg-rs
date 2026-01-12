//! Test that wrong parameter types produce a compile error.

use kwarg::{kwarg, kwargs};

#[kwarg]
fn greet(name: &str, age: u32) -> String {
    format!("Hello {}, you are {}", name, age)
}

fn main() {
    // 'age' expects u32, not &str - should fail with type error
    let _ = kwargs!(greet =>
        name: "Alice",
        age: "not a number"
    );
}
