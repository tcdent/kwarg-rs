//! # kwarg
//!
//! Keyword arguments for Rust functions via proc macros.
//!
//! This crate provides two macros:
//! - `#[kwarg]` - Annotate functions or impl blocks to enable keyword argument support
//! - `kwargs!()` - Call functions using keyword arguments
//!
//! # Example
//!
//! ```ignore
//! use kwarg::{kwarg, kwargs};
//!
//! #[kwarg]
//! fn greet(name: &str, age: u32, greeting: &str) {
//!     println!("{} {}, you are {}", greeting, name, age);
//! }
//!
//! kwargs!(greet =>
//!     greeting: "Hello",
//!     name: "Alice",
//!     age: 30
//! );
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, FnArg, Ident, ImplItem, ItemFn, ItemImpl, Pat, Path, Token,
};

/// Mangle a function path into a hidden macro name.
///
/// # Examples
/// - `greet` → `__kwarg_greet`
/// - `Foo::new` → `__kwarg_Foo_new`
fn mangle_path(segments: &[String]) -> Ident {
    let mangled = segments.join("_");
    format_ident!("__kwarg_{}", mangled)
}

/// Extract parameter names from a function signature.
fn extract_param_names(inputs: &Punctuated<FnArg, Token![,]>) -> Vec<Ident> {
    inputs
        .iter()
        .filter_map(|arg| {
            match arg {
                FnArg::Typed(pat_type) => {
                    // Extract the identifier from the pattern
                    if let Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                        Some(pat_ident.ident.clone())
                    } else {
                        None
                    }
                }
                FnArg::Receiver(_) => None, // Skip self parameters
            }
        })
        .collect()
}

/// Generate the hidden macro_rules! for a function using tt-munching pattern.
///
/// This generates a macro that uses compile-time pattern matching to route
/// each named argument to its correct position, avoiding type inference issues.
fn generate_hidden_macro(
    macro_name: &Ident,
    fn_name: &Ident,
    param_names: &[Ident],
    type_prefix: Option<&Ident>,
) -> TokenStream2 {
    // Build the function call with slot variable references (using $)
    let slot_vars: Vec<_> = param_names
        .iter()
        .map(|name| format_ident!("__{}", name))
        .collect();

    // For the @call arm pattern: need $__name:expr, $__age:expr, etc.
    let call_patterns: Vec<_> = slot_vars
        .iter()
        .map(|var| {
            quote! { $#var:expr }
        })
        .collect();

    // For the function call: need $__name, $__age, etc.
    let call_args: Vec<_> = slot_vars
        .iter()
        .map(|var| {
            quote! { $#var }
        })
        .collect();

    let fn_call = if let Some(type_name) = type_prefix {
        quote! { #type_name::#fn_name(#(#call_args),*) }
    } else {
        quote! { #fn_name(#(#call_args),*) }
    };

    // Generate the @call arm that makes the final function call
    let call_arm = quote! {
        (@call [#(#call_patterns),*]) => {
            #fn_call
        };
    };

    // Generate collector arms for each parameter
    // Each arm matches one specific parameter and routes it to the correct slot
    let collector_arms: Vec<_> = param_names
        .iter()
        .enumerate()
        .map(|(idx, param)| {
            // Build the slot pattern with a placeholder for this param's slot
            let slot_patterns: Vec<_> = param_names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let var = format_ident!("__{}", name);
                    if i == idx {
                        // This slot gets replaced with the new value
                        quote! { $__old:expr }
                    } else {
                        // Other slots pass through
                        quote! { $#var:expr }
                    }
                })
                .collect();

            // Build the recursive call with the new value in the correct slot
            let slot_args: Vec<_> = param_names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let var = format_ident!("__{}", name);
                    if i == idx {
                        quote! { $__val }
                    } else {
                        quote! { $#var }
                    }
                })
                .collect();

            quote! {
                (@collect [#param : $__val:expr, $($rest:tt)*] [#(#slot_patterns),*]) => {
                    #macro_name!(@collect [$($rest)*] [#(#slot_args),*])
                };
                (@collect [#param : $__val:expr] [#(#slot_patterns),*]) => {
                    #macro_name!(@collect [] [#(#slot_args),*])
                };
            }
        })
        .collect();

    // Generate the terminal arm (empty input -> make the call)
    let terminal_arm = quote! {
        (@collect [] [$($slots:expr),*]) => {
            #macro_name!(@call [$($slots),*])
        };
    };

    // Generate default slot values (compile_error! for missing params)
    let default_slots: Vec<_> = param_names
        .iter()
        .map(|name| {
            let error_msg = format!("kwarg: missing required parameter `{}`", name);
            quote! { ::core::compile_error!(#error_msg) }
        })
        .collect();

    // Entry point arm
    let entry_arm = quote! {
        ($($args:tt)*) => {
            #macro_name!(@collect [$($args)*] [#(#default_slots),*])
        };
    };

    quote! {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! #macro_name {
            #call_arm

            #(#collector_arms)*

            #terminal_arm

            #entry_arm
        }
    }
}

/// Attribute macro to enable keyword arguments for a function or impl block.
///
/// # On Functions
///
/// ```ignore
/// #[kwarg]
/// fn greet(name: &str, age: u32) {
///     println!("Hello {}, you are {}", name, age);
/// }
/// ```
///
/// # On Impl Blocks
///
/// ```ignore
/// #[kwarg]
/// impl Foo {
///     fn new(x: i32, y: String) -> Self {
///         Foo { x, y }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn kwarg(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Try to parse as a function first
    if let Ok(func) = syn::parse::<ItemFn>(item.clone()) {
        return process_function(func).into();
    }

    // Try to parse as an impl block
    if let Ok(impl_block) = syn::parse::<ItemImpl>(item.clone()) {
        return process_impl_block(impl_block).into();
    }

    // Neither worked, emit an error
    let item2: TokenStream2 = item.into();
    let expanded = quote! {
        compile_error!("#[kwarg] can only be applied to functions or impl blocks");
        #item2
    };
    expanded.into()
}

/// Process a standalone function with #[kwarg].
fn process_function(func: ItemFn) -> TokenStream2 {
    let fn_name = &func.sig.ident;
    let param_names = extract_param_names(&func.sig.inputs);

    // Mangle just the function name for free functions
    let macro_name = mangle_path(&[fn_name.to_string()]);

    let hidden_macro = generate_hidden_macro(&macro_name, fn_name, &param_names, None);

    quote! {
        #func

        #hidden_macro
    }
}

/// Process an impl block with #[kwarg].
fn process_impl_block(mut impl_block: ItemImpl) -> TokenStream2 {
    // Extract the type name
    let type_name = match impl_block.self_ty.as_ref() {
        syn::Type::Path(type_path) => {
            type_path.path.segments.last().map(|seg| seg.ident.clone())
        }
        _ => None,
    };

    let type_name = match type_name {
        Some(name) => name,
        None => {
            return quote! {
                compile_error!("#[kwarg] on impl blocks requires a simple type path");
                #impl_block
            };
        }
    };

    let mut hidden_macros = Vec::new();

    // Process each method in the impl block
    for item in &mut impl_block.items {
        if let ImplItem::Fn(method) = item {
            // Skip methods with self receiver for now (instance methods)
            let has_self = method.sig.inputs.iter().any(|arg| {
                matches!(arg, FnArg::Receiver(_))
            });

            if has_self {
                // For instance methods, we'd need a different approach
                // Skip for alpha version
                continue;
            }

            let fn_name = &method.sig.ident;
            let param_names = extract_param_names(&method.sig.inputs);

            // Mangle as Type_method
            let macro_name = mangle_path(&[type_name.to_string(), fn_name.to_string()]);

            let hidden_macro =
                generate_hidden_macro(&macro_name, fn_name, &param_names, Some(&type_name));
            hidden_macros.push(hidden_macro);
        }
    }

    quote! {
        #impl_block

        #(#hidden_macros)*
    }
}

/// Input for the kwargs! macro: `path => key: value, key: value, ...`
struct KwargsInput {
    path: Path,
    args: Punctuated<KwargArg, Token![,]>,
}

/// A single keyword argument: `key: value`
struct KwargArg {
    key: Ident,
    value: Expr,
}

impl Parse for KwargsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: Path = input.parse()?;
        input.parse::<Token![=>]>()?;

        let args = Punctuated::parse_terminated(input)?;

        Ok(KwargsInput { path, args })
    }
}

impl Parse for KwargArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let value: Expr = input.parse()?;

        Ok(KwargArg { key, value })
    }
}

/// Call a function using keyword arguments.
///
/// # Example
///
/// ```ignore
/// kwargs!(greet =>
///     name: "Alice",
///     age: 30,
///     greeting: "Hello"
/// );
///
/// kwargs!(Foo::new =>
///     x: 42,
///     y: "hello".to_string()
/// );
/// ```
#[proc_macro]
pub fn kwargs(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as KwargsInput);

    // Mangle the path to get the hidden macro name
    let segments: Vec<String> = input
        .path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect();

    let macro_name = mangle_path(&segments);

    // Collect the keyword arguments
    let args: Vec<_> = input
        .args
        .iter()
        .map(|arg| {
            let key = &arg.key;
            let value = &arg.value;
            quote! { #key: #value }
        })
        .collect();

    // Generate the macro invocation
    let expanded = quote! {
        #macro_name!(#(#args),*)
    };

    expanded.into()
}
