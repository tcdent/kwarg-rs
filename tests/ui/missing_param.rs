//! Test that missing required parameters produce a compile error.

use kwarg::{kwarg, kwargs};

#[kwarg]
fn greet(name: &str, age: u32) -> String {
    format!("Hello {}, you are {}", name, age)
}

fn main() {
    // Missing the 'age' parameter - should fail
    let _ = kwargs!(greet =>
        name: "Alice"
    );
}
