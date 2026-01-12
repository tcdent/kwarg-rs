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

// =============================================================================
// GENERIC TYPE PARAMETER TESTS
// =============================================================================

#[kwarg]
fn identity<T>(value: T) -> T {
    value
}

#[kwarg]
fn swap<T, U>(first: T, second: U) -> (U, T) {
    (second, first)
}

#[kwarg]
fn generic_with_bound<T: Clone>(item: T, count: usize) -> Vec<T> {
    vec![item; count]
}

#[test]
fn test_generic_single_type_param() {
    let result = kwargs!(identity =>
        value: 42
    );
    assert_eq!(result, 42);

    let result_str = kwargs!(identity =>
        value: "hello"
    );
    assert_eq!(result_str, "hello");
}

#[test]
fn test_generic_multiple_type_params() {
    let result = kwargs!(swap =>
        second: "world",
        first: 123
    );
    assert_eq!(result, ("world", 123));
}

#[test]
fn test_generic_with_trait_bound() {
    let result = kwargs!(generic_with_bound =>
        count: 3,
        item: "x"
    );
    assert_eq!(result, vec!["x", "x", "x"]);
}

// =============================================================================
// LIFETIME PARAMETER TESTS
// =============================================================================

#[kwarg]
fn slice_from<'a>(data: &'a [u8], start: usize) -> &'a [u8] {
    &data[start..]
}

#[kwarg]
fn longer<'a>(first: &'a str, second: &'a str) -> &'a str {
    if first.len() >= second.len() {
        first
    } else {
        second
    }
}

#[kwarg]
fn lifetime_with_generic<'a, T>(slice: &'a [T], index: usize) -> &'a T {
    &slice[index]
}

#[test]
fn test_lifetime_single() {
    let data = vec![1u8, 2, 3, 4, 5];
    let result = kwargs!(slice_from =>
        start: 2,
        data: &data
    );
    assert_eq!(result, &[3, 4, 5]);
}

#[test]
fn test_lifetime_multiple_refs() {
    let result = kwargs!(longer =>
        second: "hi",
        first: "hello"
    );
    assert_eq!(result, "hello");
}

#[test]
fn test_lifetime_with_generics() {
    let nums = vec![10, 20, 30];
    let result = kwargs!(lifetime_with_generic =>
        index: 1,
        slice: &nums
    );
    assert_eq!(*result, 20);
}

// =============================================================================
// REFERENCE AND MUTABLE REFERENCE TESTS
// =============================================================================

#[kwarg]
fn ref_params(first: &i32, second: &str) -> String {
    format!("{}: {}", second, first)
}

#[kwarg]
fn mut_ref_increment(value: &mut i32, amount: i32) {
    *value += amount;
}

#[kwarg]
fn mixed_refs(shared: &str, exclusive: &mut String, count: usize) {
    for _ in 0..count {
        exclusive.push_str(shared);
    }
}

#[test]
fn test_immutable_references() {
    let num = 42;
    let result = kwargs!(ref_params =>
        second: "Value",
        first: &num
    );
    assert_eq!(result, "Value: 42");
}

#[test]
fn test_mutable_reference() {
    let mut value = 10;
    kwargs!(mut_ref_increment =>
        amount: 5,
        value: &mut value
    );
    assert_eq!(value, 15);
}

#[test]
fn test_mixed_references() {
    let shared = "abc";
    let mut exclusive = String::new();
    kwargs!(mixed_refs =>
        count: 3,
        exclusive: &mut exclusive,
        shared: shared
    );
    assert_eq!(exclusive, "abcabcabc");
}

// =============================================================================
// GENERIC IMPL BLOCK TESTS
// =============================================================================

struct Container<T> {
    items: Vec<T>,
}

#[kwarg]
impl<T> Container<T> {
    fn new(capacity: usize) -> Self {
        Container {
            items: Vec::with_capacity(capacity),
        }
    }

    fn with_items(items: Vec<T>) -> Self {
        Container { items }
    }
}

struct Pair<T, U> {
    first: T,
    second: U,
}

#[kwarg]
impl<T, U> Pair<T, U> {
    fn new(first: T, second: U) -> Self {
        Pair { first, second }
    }
}

#[test]
fn test_generic_impl_single_param() {
    let container: Container<i32> = kwargs!(Container::new =>
        capacity: 10
    );
    assert_eq!(container.items.capacity(), 10);
}

#[test]
fn test_generic_impl_with_items() {
    let container = kwargs!(Container::with_items =>
        items: vec![1, 2, 3]
    );
    assert_eq!(container.items, vec![1, 2, 3]);
}

#[test]
fn test_generic_impl_multiple_params() {
    let pair = kwargs!(Pair::new =>
        second: "hello",
        first: 42
    );
    assert_eq!(pair.first, 42);
    assert_eq!(pair.second, "hello");
}

// =============================================================================
// COMPLEX PARAMETER TYPE TESTS
// =============================================================================

use std::collections::HashMap;

#[kwarg]
fn process_vec(items: Vec<i32>, multiplier: i32) -> Vec<i32> {
    items.into_iter().map(|x| x * multiplier).collect()
}

#[kwarg]
fn process_option(value: Option<i32>, default: i32) -> i32 {
    value.unwrap_or(default)
}

#[kwarg]
fn process_hashmap(map: HashMap<String, i32>, key: String) -> Option<i32> {
    map.get(&key).copied()
}

#[kwarg]
fn nested_generics(data: Vec<Option<i32>>, filter_none: bool) -> Vec<i32> {
    if filter_none {
        data.into_iter().flatten().collect()
    } else {
        data.into_iter().map(|x| x.unwrap_or(0)).collect()
    }
}

#[test]
fn test_vec_parameter() {
    let result = kwargs!(process_vec =>
        multiplier: 2,
        items: vec![1, 2, 3]
    );
    assert_eq!(result, vec![2, 4, 6]);
}

#[test]
fn test_option_parameter() {
    let result_some = kwargs!(process_option =>
        default: 0,
        value: Some(42)
    );
    assert_eq!(result_some, 42);

    let result_none = kwargs!(process_option =>
        default: 99,
        value: None
    );
    assert_eq!(result_none, 99);
}

#[test]
fn test_hashmap_parameter() {
    let mut map = HashMap::new();
    map.insert("key1".to_string(), 100);
    map.insert("key2".to_string(), 200);

    let result = kwargs!(process_hashmap =>
        key: "key1".to_string(),
        map: map
    );
    assert_eq!(result, Some(100));
}

#[test]
fn test_nested_generic_types() {
    let data = vec![Some(1), None, Some(3), None, Some(5)];

    let filtered = kwargs!(nested_generics =>
        filter_none: true,
        data: data.clone()
    );
    assert_eq!(filtered, vec![1, 3, 5]);

    let with_zeros = kwargs!(nested_generics =>
        filter_none: false,
        data: data
    );
    assert_eq!(with_zeros, vec![1, 0, 3, 0, 5]);
}

// =============================================================================
// RETURN TYPE VARIATION TESTS
// =============================================================================

#[kwarg]
fn returns_option(value: i32, threshold: i32) -> Option<i32> {
    if value > threshold {
        Some(value)
    } else {
        None
    }
}

#[kwarg]
fn returns_result(numerator: i32, denominator: i32) -> Result<i32, String> {
    if denominator == 0 {
        Err("division by zero".to_string())
    } else {
        Ok(numerator / denominator)
    }
}

#[kwarg]
fn returns_tuple(a: i32, b: i32, c: i32) -> (i32, i32, i32) {
    (c, b, a)
}

#[kwarg]
fn returns_unit(message: &str, _unused: i32) {
    let _ = message;
}

#[test]
fn test_returns_option() {
    let some_result = kwargs!(returns_option =>
        threshold: 5,
        value: 10
    );
    assert_eq!(some_result, Some(10));

    let none_result = kwargs!(returns_option =>
        threshold: 10,
        value: 5
    );
    assert_eq!(none_result, None);
}

#[test]
fn test_returns_result() {
    let ok_result = kwargs!(returns_result =>
        denominator: 2,
        numerator: 10
    );
    assert_eq!(ok_result, Ok(5));

    let err_result = kwargs!(returns_result =>
        denominator: 0,
        numerator: 10
    );
    assert!(err_result.is_err());
}

#[test]
fn test_returns_tuple() {
    let result = kwargs!(returns_tuple =>
        c: 3,
        a: 1,
        b: 2
    );
    assert_eq!(result, (3, 2, 1));
}

#[test]
fn test_returns_unit() {
    // Should compile and run without issues
    kwargs!(returns_unit =>
        _unused: 0,
        message: "test"
    );
}

// =============================================================================
// ZERO-PARAMETER FUNCTION TESTS
// =============================================================================

#[kwarg]
fn no_params() -> i32 {
    42
}

struct NoParamStruct;

#[kwarg]
impl NoParamStruct {
    fn create() -> Self {
        NoParamStruct
    }

    fn get_value() -> i32 {
        100
    }
}

#[test]
fn test_zero_param_function() {
    let result = kwargs!(no_params =>);
    assert_eq!(result, 42);
}

#[test]
fn test_zero_param_impl_methods() {
    let _instance = kwargs!(NoParamStruct::create =>);
    let value = kwargs!(NoParamStruct::get_value =>);
    assert_eq!(value, 100);
}

// =============================================================================
// MULTIPLE IMPL BLOCKS TESTS
// =============================================================================

struct MultiImpl {
    value: i32,
}

#[kwarg]
impl MultiImpl {
    fn new(value: i32) -> Self {
        MultiImpl { value }
    }
}

#[kwarg]
impl MultiImpl {
    fn from_pair(a: i32, b: i32) -> Self {
        MultiImpl { value: a + b }
    }

    fn from_triple(x: i32, y: i32, z: i32) -> Self {
        MultiImpl { value: x + y + z }
    }
}

#[test]
fn test_multiple_impl_blocks() {
    let m1 = kwargs!(MultiImpl::new =>
        value: 10
    );
    assert_eq!(m1.value, 10);

    let m2 = kwargs!(MultiImpl::from_pair =>
        b: 20,
        a: 10
    );
    assert_eq!(m2.value, 30);

    let m3 = kwargs!(MultiImpl::from_triple =>
        z: 3,
        x: 1,
        y: 2
    );
    assert_eq!(m3.value, 6);
}

// =============================================================================
// VISIBILITY MODIFIER TESTS
// =============================================================================

mod inner {
    use kwarg::{kwarg, kwargs};

    #[kwarg]
    pub fn public_fn(x: i32, y: i32) -> i32 {
        x + y
    }

    pub struct PubStruct {
        pub value: i32,
    }

    #[kwarg]
    impl PubStruct {
        pub fn new(value: i32) -> Self {
            PubStruct { value }
        }

        pub fn create_with_offset(base: i32, offset: i32) -> Self {
            PubStruct {
                value: base + offset,
            }
        }
    }

    #[test]
    fn test_module_internal() {
        let result = kwargs!(public_fn =>
            y: 2,
            x: 1
        );
        assert_eq!(result, 3);
    }
}

#[test]
fn test_public_function_from_module() {
    // Call the public function from parent module
    let result = inner::public_fn(5, 10);
    assert_eq!(result, 15);
}

#[test]
fn test_public_impl_from_module() {
    let s = inner::PubStruct::new(42);
    assert_eq!(s.value, 42);

    let s2 = inner::PubStruct::create_with_offset(10, 5);
    assert_eq!(s2.value, 15);
}

// =============================================================================
// COMPLEX EXPRESSION TESTS
// =============================================================================

#[kwarg]
fn compute(a: i32, b: i32, c: i32) -> i32 {
    a + b * c
}

fn helper() -> i32 {
    10
}

#[test]
fn test_literal_expressions() {
    let result = kwargs!(compute =>
        c: 2 + 3,
        b: 4 * 2,
        a: 1
    );
    // a + b * c = 1 + 8 * 5 = 1 + 40 = 41
    assert_eq!(result, 41);
}

#[test]
fn test_function_call_in_args() {
    let result = kwargs!(compute =>
        a: helper(),
        b: helper() + 5,
        c: 2
    );
    // a + b * c = 10 + 15 * 2 = 10 + 30 = 40
    assert_eq!(result, 40);
}

#[test]
fn test_closure_in_args() {
    let closure = || 7;
    let result = kwargs!(compute =>
        a: closure(),
        b: (|x| x * 2)(3),
        c: 1
    );
    // a + b * c = 7 + 6 * 1 = 13
    assert_eq!(result, 13);
}

#[test]
fn test_method_call_in_args() {
    let s = "hello";
    let result = kwargs!(compute =>
        a: s.len() as i32,
        b: "world".len() as i32,
        c: 2
    );
    // a + b * c = 5 + 5 * 2 = 5 + 10 = 15
    assert_eq!(result, 15);
}

#[test]
fn test_block_expression_in_args() {
    let result = kwargs!(compute =>
        a: { let x = 1; let y = 2; x + y },
        b: { 4 },
        c: { if true { 5 } else { 0 } }
    );
    // a + b * c = 3 + 4 * 5 = 3 + 20 = 23
    assert_eq!(result, 23);
}

// =============================================================================
// ATTRIBUTE PRESERVATION TESTS
// =============================================================================

#[kwarg]
#[inline]
fn inlined_fn(x: i32, y: i32) -> i32 {
    x + y
}

#[kwarg]
#[allow(dead_code)]
fn allowed_dead_code(a: i32, b: i32) -> i32 {
    a - b
}

#[test]
fn test_inline_attribute() {
    let result = kwargs!(inlined_fn =>
        y: 5,
        x: 10
    );
    assert_eq!(result, 15);
}

// =============================================================================
// WHERE CLAUSE TESTS
// =============================================================================

#[kwarg]
fn with_where_clause<T, U>(first: T, second: U) -> String
where
    T: std::fmt::Display,
    U: std::fmt::Debug,
{
    format!("{} - {:?}", first, second)
}

struct WhereStruct<T> {
    value: T,
}

#[kwarg]
impl<T> WhereStruct<T>
where
    T: Default,
{
    fn new_default(scale: i32) -> Self
    where
        T: Clone,
    {
        let _ = scale;
        WhereStruct {
            value: T::default(),
        }
    }
}

#[test]
fn test_function_where_clause() {
    let result = kwargs!(with_where_clause =>
        second: vec![1, 2, 3],
        first: "hello"
    );
    assert_eq!(result, "hello - [1, 2, 3]");
}

#[test]
fn test_impl_where_clause() {
    let s: WhereStruct<i32> = kwargs!(WhereStruct::new_default =>
        scale: 1
    );
    assert_eq!(s.value, 0);
}

// =============================================================================
// BOX AND SMART POINTER TESTS
// =============================================================================

#[kwarg]
fn process_box(data: Box<i32>, multiplier: i32) -> i32 {
    *data * multiplier
}

#[kwarg]
fn process_rc(data: std::rc::Rc<String>, suffix: &str) -> String {
    format!("{}{}", data, suffix)
}

#[test]
fn test_box_parameter() {
    let result = kwargs!(process_box =>
        multiplier: 3,
        data: Box::new(10)
    );
    assert_eq!(result, 30);
}

#[test]
fn test_rc_parameter() {
    let result = kwargs!(process_rc =>
        suffix: "!",
        data: std::rc::Rc::new("hello".to_string())
    );
    assert_eq!(result, "hello!");
}

// =============================================================================
// ARRAY AND SLICE TESTS
// =============================================================================

#[kwarg]
fn process_array(arr: [i32; 3], index: usize) -> i32 {
    arr[index]
}

#[kwarg]
fn sum_slice(slice: &[i32], initial: i32) -> i32 {
    slice.iter().sum::<i32>() + initial
}

#[test]
fn test_array_parameter() {
    let result = kwargs!(process_array =>
        index: 1,
        arr: [10, 20, 30]
    );
    assert_eq!(result, 20);
}

#[test]
fn test_slice_parameter() {
    let data = vec![1, 2, 3, 4, 5];
    let result = kwargs!(sum_slice =>
        initial: 100,
        slice: &data
    );
    assert_eq!(result, 115);
}
