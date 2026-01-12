//! Integration tests for the kwarg crate.

use kwarg::{kwarg, kwargs};

// Test free function with #[kwarg]
#[kwarg]
fn greet(name: &str, age: u32, greeting: &str) -> String {
    format!("{} {}, you are {}", greeting, name, age)
}

// Test struct with impl block
struct Point {
    x: i32,
    y: i32,
}

#[kwarg]
impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    fn from_tuple(coords: (i32, i32)) -> Self {
        Point {
            x: coords.0,
            y: coords.1,
        }
    }
}

// Test struct with multiple parameters
struct Config {
    host: String,
    port: u16,
    timeout: u64,
    verbose: bool,
}

#[kwarg]
impl Config {
    fn new(host: String, port: u16, timeout: u64, verbose: bool) -> Self {
        Config {
            host,
            port,
            timeout,
            verbose,
        }
    }
}

#[test]
fn test_free_function() {
    // Test with kwargs - different order
    let result = kwargs!(greet =>
        greeting: "Hello",
        name: "Alice",
        age: 30
    );
    assert_eq!(result, "Hello Alice, you are 30");

    // Test with kwargs - original order
    let result2 = kwargs!(greet =>
        name: "Bob",
        age: 25,
        greeting: "Hi"
    );
    assert_eq!(result2, "Hi Bob, you are 25");
}

#[test]
fn test_original_function_still_works() {
    // Original positional call should still work
    let result = greet("Charlie", 40, "Hey");
    assert_eq!(result, "Hey Charlie, you are 40");
}

#[test]
fn test_impl_block_associated_function() {
    // Test Point::new with kwargs
    let p = kwargs!(Point::new =>
        y: 20,
        x: 10
    );
    assert_eq!(p.x, 10);
    assert_eq!(p.y, 20);

    // Original still works
    let p2 = Point::new(5, 15);
    assert_eq!(p2.x, 5);
    assert_eq!(p2.y, 15);
}

#[test]
fn test_single_param() {
    let p = kwargs!(Point::from_tuple =>
        coords: (100, 200)
    );
    assert_eq!(p.x, 100);
    assert_eq!(p.y, 200);
}

#[test]
fn test_many_params() {
    let config = kwargs!(Config::new =>
        verbose: true,
        timeout: 5000,
        host: "localhost".to_string(),
        port: 8080
    );

    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 8080);
    assert_eq!(config.timeout, 5000);
    assert!(config.verbose);
}

#[test]
fn test_trailing_comma() {
    // Should accept trailing comma
    let result = kwargs!(greet =>
        name: "Test",
        age: 1,
        greeting: "Hi",
    );
    assert_eq!(result, "Hi Test, you are 1");
}
