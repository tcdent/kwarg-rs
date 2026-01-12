# kwarg-rs

> Bringing keyword arguments to Rust, because sometimes you just want to troll.

## The Problem

Rust has beautiful named field syntax for structs:

```rust
let foo = Foo {
    x: 42,
    y: "hello".to_string(),
    z: vec![1, 2, 3],
};
```

But constructors and methods are stuck in the positional dark ages:

```rust
let foo = Foo::new(42, "hello".to_string(), vec![1, 2, 3]);
// Wait, which parameter is which again?
```

This library lets you write:

```rust
let foo = kwargs!(Foo::new =>
    x: 42,
    y: "hello".to_string(),
    z: vec![1, 2, 3]
);
```

## User Interface

### 1. Annotate Your Functions

Add `#[kwarg]` to any function or method you want to support keyword arguments:

```rust
use kwarg::kwarg;

impl Foo {
    #[kwarg]
    fn new(x: i32, y: String, z: Vec<i32>) -> Self {
        Foo { x, y, z }
    }
    
    #[kwarg]
    fn configure(&mut self, timeout: u64, retries: u32, verbose: bool) {
        // ...
    }
}

#[kwarg]
fn greet(name: &str, age: u32, greeting: &str) {
    println!("{} {}, you are {}", greeting, name, age);
}
```

### 2. Call With Keyword Arguments

Use the `kwargs!` macro at the call site:

```rust
use kwarg::kwargs;

// For methods
let foo = kwargs!(Foo::new =>
    y: "hello".to_string(),
    z: vec![1, 2, 3],
    x: 42  // Order doesn't matter!
);

// For functions
kwargs!(greet =>
    greeting: "Hello",
    name: "Alice",
    age: 30
);

// Still works for method calls
kwargs!(foo.configure =>
    verbose: true,
    timeout: 5000,
    retries: 3
);
```

### 3. Original Functions Still Work!

The annotated functions remain unchanged and can be called normally:

```rust
// Positional arguments still work
let foo = Foo::new(42, "hello".to_string(), vec![1, 2, 3]);
greet("Alice", 30, "Hello");

// You choose when to use kwargs
```

## Features

- ✅ **Order-independent arguments** - specify parameters in any order
- ✅ **Compile-time checking** - all the safety of Rust, no runtime overhead
- ✅ **Zero-cost abstraction** - expands to normal function calls
- ✅ **Works with methods** - `Foo::new`, `self.method()`, everything
- ✅ **Non-invasive** - original functions unchanged, opt-in at call sites
- ✅ **Type inference preserved** - works with Rust's type system
- ✅ **Clear error messages** - missing or unknown parameters caught at compile time

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
kwarg = "0.1.0"
```

## Examples

### Constructor with Many Parameters

```rust
#[kwarg]
impl Config {
    fn new(
        host: String,
        port: u16,
        timeout: Duration,
        max_connections: usize,
        enable_tls: bool,
        retry_policy: RetryPolicy,
    ) -> Self {
        // ...
    }
}

let config = kwargs!(Config::new =>
    host: "localhost".to_string(),
    port: 8080,
    enable_tls: true,
    timeout: Duration::from_secs(30),
    retry_policy: RetryPolicy::Exponential,
    max_connections: 100
);
```

### Builder Pattern Alternative

Instead of this:

```rust
let request = HttpRequest::builder()
    .method("POST")
    .url("https://api.example.com")
    .header("Content-Type", "application/json")
    .body(body)
    .timeout(Duration::from_secs(30))
    .build()?;
```

Write this:

```rust
#[kwarg]
fn make_request(
    method: &str,
    url: &str,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    timeout: Duration,
) -> Result<Response> {
    // ...
}

let response = kwargs!(make_request =>
    method: "POST",
    url: "https://api.example.com",
    headers: headers,
    body: body,
    timeout: Duration::from_secs(30)
)?;
```

### Works With References and Lifetimes

```rust
#[kwarg]
fn process<'a>(data: &'a [u8], offset: usize, length: usize) -> &'a [u8] {
    &data[offset..offset + length]
}

let result = kwargs!(process =>
    data: &buffer,
    length: 100,
    offset: 50
);
```

## Limitations

- Parameters must have unique names (no overloading based on type)
- All parameters are required (no optional/default values yet)
- Cannot use with variadic functions
- Macro names must be unique across your crate (standard Rust macro limitation)

## Why This Exists

This is partly a proof-of-concept to explore what's possible with Rust's macro system, and partly a genuine ergonomics improvement for functions with many parameters. It's inspired by:

- Python's keyword arguments
- Rust's struct literal syntax
- A desire to troll Rust developers who insist builder patterns are the only way

If you find this useful (or hilarious), great! If you think it's an abomination, that's also valid. Rust's macro system is powerful enough to do this, so why not?

---

# Technical Implementation

## Architecture Overview

The library consists of two proc macros that coordinate via a naming convention:

1. **`#[kwarg]` attribute macro** - Processes function definitions
2. **`kwargs!()` function-like macro** - Processes call sites

Both macros independently compute the same mangled name for a "hidden" declarative macro, allowing them to coordinate without any global registry or shared state.

## Design Decisions

### Decision 1: Two-Macro System

**Why not just one macro?**

We need to process both the function definition (to extract parameter names and order) and the call site (to reorder arguments). This requires two separate macros:

- `#[kwarg]` runs at the function definition
- `kwargs!()` runs at the call site

### Decision 2: Convention-Based Coordination

**The core insight:** Both macros compute the same mangled name from the function path.

```
Foo::new → __kwarg_Foo_new
std::fs::File::open → __kwarg_std_fs_File_open
```

**Why this works:**
- No global registry needed (macros can't access external state)
- No complex coordination required
- Simple, deterministic naming scheme
- Scales to any function path

**Name mangling algorithm:**
```
1. Take the full path: Foo::new
2. Replace :: with _: Foo_new
3. Prepend __kwarg_: __kwarg_Foo_new
4. This becomes the hidden macro name
```

### Decision 3: Preserve Original Functions

**Critical requirement:** The original function must remain unchanged.

```rust
#[kwarg]
fn foo(x: i32, y: String) -> Bar { ... }

// Must still be callable normally:
foo(42, "hello".to_string());
```

**Why:** This makes the library non-invasive. You can:
- Gradually adopt kwargs without changing existing code
- Choose per-call-site whether to use kwargs
- Maintain backwards compatibility
- Keep function signatures clean for documentation

**Implementation:** The `#[kwarg]` macro generates a *separate* hidden macro but leaves the original function untouched.

### Decision 4: Hidden Macro Generation

**What `#[kwarg]` actually does:**

```rust
// Input:
#[kwarg]
fn greet(name: &str, age: u32, greeting: &str) {
    println!("{} {}, you are {}", greeting, name, age);
}

// Output:
fn greet(name: &str, age: u32, greeting: &str) {  // Original unchanged!
    println!("{} {}, you are {}", greeting, name, age);
}

#[doc(hidden)]
macro_rules! __kwarg_greet {
    ($($key:ident: $value:expr),* $(,)?) => {{
        // Collect arguments by name
        let mut __name = None;
        let mut __age = None;
        let mut __greeting = None;
        
        $(
            match stringify!($key) {
                "name" => __name = Some($value),
                "age" => __age = Some($value),
                "greeting" => __greeting = Some($value),
                _ => compile_error!(concat!("Unknown parameter: ", stringify!($key))),
            }
        )*
        
        // Call with correct order
        greet(
            __name.expect("Missing required parameter: name"),
            __age.expect("Missing required parameter: age"),
            __greeting.expect("Missing required parameter: greeting"),
        )
    }};
}
```

**Design choices in the generated macro:**

1. **Use `Option<T>` for collection:** Allows detecting missing/duplicate parameters
2. **`stringify!($key)`:** Convert parameter names to strings for matching
3. **`compile_error!`:** Catch unknown parameters at compile time
4. **`.expect()`:** Clear runtime error if parameter is missing (shouldn't happen if used correctly)
5. **`#[doc(hidden)]`:** Hide from documentation, these are implementation details

### Decision 5: Call Site Macro Expansion

**What `kwargs!()` does:**

```rust
// Input:
kwargs!(Foo::new =>
    y: "hello".to_string(),
    x: 42
)

// Step 1: Parse the path
// Path: Foo::new
// Mangled: __kwarg_Foo_new

// Step 2: Expand to hidden macro call
__kwarg_Foo_new!(
    y: "hello".to_string(),
    x: 42
)

// Step 3: That macro reorders and calls
Foo::new(42, "hello".to_string())
```

**Key implementation details:**

1. **Parse the path as `syn::Path`:** Handle arbitrary module paths
2. **Support both `::` paths and `.` method calls**
3. **Forward all tokens after `=>` to the hidden macro**
4. **Preserve spans for error messages**

### Decision 6: Method Call Support

**Problem:** We don't know the type of `obj` at macro expansion time!

**Solution:** Only support static paths:
```rust
// Supported
kwargs!(Foo::method => ...)

// Not yet supported, use workaround:
let result = kwargs!(Foo::method => obj: &self, x: 42);
```
