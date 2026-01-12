//! Basic example demonstrating kwarg usage.

use kwarg::{kwarg, kwargs};

// A simple greeting function
#[kwarg]
fn greet(name: &str, age: u32, greeting: &str) {
    println!("{} {}, you are {} years old!", greeting, name, age);
}

// A configuration struct
struct ServerConfig {
    host: String,
    port: u16,
    max_connections: usize,
    enable_tls: bool,
}

#[kwarg]
impl ServerConfig {
    fn new(host: String, port: u16, max_connections: usize, enable_tls: bool) -> Self {
        ServerConfig {
            host,
            port,
            max_connections,
            enable_tls,
        }
    }
}

impl std::fmt::Debug for ServerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("max_connections", &self.max_connections)
            .field("enable_tls", &self.enable_tls)
            .finish()
    }
}

fn main() {
    println!("=== Free Function Example ===");

    // Using kwargs - order doesn't matter!
    kwargs!(greet =>
        greeting: "Hello",
        name: "Alice",
        age: 30
    );

    // Different order, same result
    kwargs!(greet =>
        age: 25,
        name: "Bob",
        greeting: "Hi"
    );

    // Original positional call still works
    greet("Charlie", 40, "Hey");

    println!("\n=== Impl Block Example ===");

    // Create a config with kwargs
    let config = kwargs!(ServerConfig::new =>
        enable_tls: true,
        max_connections: 100,
        host: "localhost".to_string(),
        port: 8080
    );

    println!("Created config: {:?}", config);

    // Original constructor still works
    let config2 = ServerConfig::new("0.0.0.0".to_string(), 443, 50, false);
    println!("Created config2: {:?}", config2);
}
