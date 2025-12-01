//! Derive macros for RGB ECS components.
//!
//! This crate provides `#[derive(Component)]` which enforces that components
//! are simple, flat data types suitable for an ECS with owned-value semantics.
//!
//! # Component Types
//!
//! ## Regular Components (POD)
//!
//! By default, components must be "plain old data" - no heap allocations,
//! no collections, no handles. This ensures:
//! - Cheap cloning (owned-value semantics)
//! - Easy persistence/serialization
//! - Predictable memory layout
//!
//! ```ignore
//! #[derive(Component, Clone)]
//! struct Position { x: f64, y: f64, z: f64 }
//! ```
//!
//! ## Opaque Components
//!
//! For runtime handles, channels, and other non-POD types, use `#[component(opaque)]`:
//!
//! ```ignore
//! #[derive(Component, Clone)]
//! #[component(opaque)]
//! struct NetworkIngress {
//!     rx: Receiver<Packet>,
//! }
//! ```
//!
//! Opaque components:
//! - Skip forbidden type validation
//! - Cannot be persisted to storage
//! - Should be used sparingly (prefer relations for per-entity data)
//!
//! # Forbidden Types (for non-opaque)
//!
//! - `Vec<T>` - Use relations: spawn child entities with `(Data, ChildOf(parent))`
//! - `VecDeque<T>` - Use relations with sequence field for ordering
//! - `HashMap<K, V>` / `HashSet<T>` - Use relations with pair queries
//! - `String` - Use fixed-size arrays or interned strings
//! - `Box<T>` / `Rc<T>` / `Arc<T>` - Use entity references
//! - `Mutex<T>` / `RwLock<T>` - ECS handles synchronization
//!
//! # Allowed Types (for non-opaque)
//!
//! - Primitives: `i8`..`i128`, `u8`..`u128`, `f32`, `f64`, `bool`, `char`
//! - Fixed arrays: `[T; N]` where T is allowed
//! - Tuples: `(A, B)` where all elements are allowed
//! - `Option<T>` where T is allowed
//! - Other `#[derive(Component)]` structs
//! - `Entity` (entity references)

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    Attribute, Data, DeriveInput, Fields, GenericArgument, Meta, Path, PathArguments, Type,
    spanned::Spanned,
};

/// Forbidden type patterns that indicate misuse of ECS components.
/// Each entry is (type_name, error_message).
const FORBIDDEN_TYPES: &[(&str, &str)] = &[
    (
        "Vec",
        "Vec<T> is not allowed in components. Use relations instead:\n\
         - Spawn each item as a separate entity with a relation to this entity\n\
         - Example: world.spawn((ItemData { ... }, Pair::<ChildOf>(parent_entity)))\n\
         - Query with: query.with::<ItemData>().pair::<ChildOf>(parent_entity)\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "VecDeque",
        "VecDeque<T> is not allowed in components. Use relations instead:\n\
         - Each queue item becomes an entity with a relation\n\
         - Add a sequence/priority field for ordering\n\
         - Example: world.spawn((Packet { seq: 1, data }, Pair::<PendingFor>(connection)))\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "HashMap",
        "HashMap<K, V> is not allowed in components. Use relations instead:\n\
         - The key becomes part of a pair relation\n\
         - Example: world.spawn((Value { ... }, Pair::<KeyedBy>(key_entity)))\n\
         - Or use named entities: world.lookup(key_bytes)\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "HashSet",
        "HashSet<T> is not allowed in components. Use tag relations instead:\n\
         - Each set member becomes a pair relation\n\
         - Example: world.insert(entity, Pair::<HasTag>(tag_entity))\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "BTreeMap",
        "BTreeMap<K, V> is not allowed in components. Use relations with order_by instead:\n\
         - Store items as entities with relations\n\
         - Use query.order_by::<Key>() for sorted iteration\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "BTreeSet",
        "BTreeSet<T> is not allowed in components. Use relations with order_by instead.\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "String",
        "String is not allowed in components. Alternatives:\n\
         - Use fixed-size arrays: [u8; 32] or a wrapper struct\n\
         - Use interned/hashed strings: StringId(u64)\n\
         - For names: use world.set_name(entity, \"name\") and world.lookup(\"name\")\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "Box",
        "Box<T> is not allowed in components. Use flat data or relations instead.\n\
         - If T is large, consider if it should be multiple smaller components\n\
         - If T is a trait object, use entity relations with marker components\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "Rc",
        "Rc<T> is not allowed in components. Reference counting breaks owned-value semantics.\n\
         - Use entity references (Entity) instead of Rc\n\
         - Store shared data on a separate entity and reference it\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "Arc",
        "Arc<T> is not allowed in components. Reference counting breaks owned-value semantics.\n\
         - Use entity references (Entity) instead of Arc\n\
         - Store shared data on a separate entity and reference it via relation\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "Mutex",
        "Mutex<T> is not allowed in components. ECS handles synchronization.\n\
         - Components are accessed via the World, which provides proper synchronization\n\
         - For shared mutable state, use the WORLD entity or relations\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "RwLock",
        "RwLock<T> is not allowed in components. ECS handles synchronization.\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "RefCell",
        "RefCell<T> is not allowed in components. ECS handles interior mutability.\n\
         - Use world.get() and world.update() for component access\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "Cell",
        "Cell<T> is not allowed in components. ECS handles interior mutability.\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "Sender",
        "Channel Sender is not allowed in components. Use events instead:\n\
         - world.send(entity, MyEvent { ... })\n\
         - Or spawn event entities with relations\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "Receiver",
        "Channel Receiver is not allowed in components. Use event queries instead.\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "LinkedList",
        "LinkedList<T> is not allowed in components. Use relations instead.\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
    (
        "BinaryHeap",
        "BinaryHeap<T> is not allowed in components. Use relations with priority field instead.\n\
         - Or mark this component as #[component(opaque)] if it's a runtime-only handle",
    ),
];

/// Check if the derive has `#[component(opaque)]` attribute
fn is_opaque(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path().is_ident("component") {
            if let Meta::List(meta_list) = &attr.meta {
                let tokens = meta_list.tokens.to_string();
                if tokens.trim() == "opaque" {
                    return true;
                }
            }
        }
    }
    false
}

/// Derive macro for ECS components.
///
/// By default, enforces that components contain only simple, flat data types.
/// Use `#[component(opaque)]` to skip validation for runtime handles.
///
/// # Examples
///
/// ```ignore
/// // Regular POD component
/// #[derive(Component, Clone)]
/// struct Position { x: f64, y: f64, z: f64 }
///
/// // Opaque component (runtime handle)
/// #[derive(Component, Clone)]
/// #[component(opaque)]
/// struct NetworkHandle { sender: Sender<Bytes> }
/// ```
#[proc_macro_derive(Component, attributes(component))]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Check for #[component(opaque)]
    let opaque = is_opaque(&input.attrs);

    // Collect all field types and check them (unless opaque)
    let mut errors = Vec::new();

    if !opaque {
        match &input.data {
            Data::Struct(data) => {
                check_fields(&data.fields, &mut errors);
            }
            Data::Enum(data) => {
                for variant in &data.variants {
                    check_fields(&variant.fields, &mut errors);
                }
            }
            Data::Union(_) => {
                errors.push(quote_spanned! {
                    input.span() =>
                    compile_error!("Unions cannot derive Component. Use a struct or enum instead.");
                });
            }
        }
    }

    // If there are errors, return them
    if !errors.is_empty() {
        let error_tokens = errors.into_iter().collect::<proc_macro2::TokenStream>();
        return TokenStream::from(error_tokens);
    }

    // Generate the impl
    // Since Component has a blanket impl for all Send + Sync + 'static types,
    // we just need to verify the type meets the constraints.
    // The derive is primarily for compile-time validation of field types.
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        // Static assertions to verify the type is suitable for ECS
        const _: () = {
            // Verify Clone is implemented
            fn _assert_clone<T: Clone>() {}
            // Verify Send + Sync + 'static
            fn _assert_component<T: Send + Sync + 'static>() {}

            fn _verify #impl_generics () #where_clause {
                _assert_clone::<#name #ty_generics>();
                _assert_component::<#name #ty_generics>();
            }
        };
    };

    TokenStream::from(expanded)
}

fn check_fields(fields: &Fields, errors: &mut Vec<proc_macro2::TokenStream>) {
    match fields {
        Fields::Named(named) => {
            for field in &named.named {
                check_type(&field.ty, errors);
            }
        }
        Fields::Unnamed(unnamed) => {
            for field in &unnamed.unnamed {
                check_type(&field.ty, errors);
            }
        }
        Fields::Unit => {}
    }
}

fn check_type(ty: &Type, errors: &mut Vec<proc_macro2::TokenStream>) {
    match ty {
        Type::Path(type_path) => {
            check_type_path(&type_path.path, ty.span(), errors);
        }
        Type::Array(array) => {
            // Arrays are fine, but check the element type
            check_type(&array.elem, errors);
        }
        Type::Tuple(tuple) => {
            // Tuples are fine, check all elements
            for elem in &tuple.elems {
                check_type(elem, errors);
            }
        }
        Type::Paren(paren) => {
            check_type(&paren.elem, errors);
        }
        Type::Group(group) => {
            check_type(&group.elem, errors);
        }
        Type::Reference(_) => {
            errors.push(quote_spanned! {
                ty.span() =>
                compile_error!("References are not allowed in components. Components use owned values.\n\
                               Use the actual type, not a reference to it.\n\
                               Or mark this component as #[component(opaque)] if it's a runtime-only handle.");
            });
        }
        Type::Ptr(_) => {
            errors.push(quote_spanned! {
                ty.span() =>
                compile_error!("Raw pointers are not allowed in components.\n\
                               Use entity references (Entity type) instead.\n\
                               Or mark this component as #[component(opaque)] if it's a runtime-only handle.");
            });
        }
        Type::TraitObject(_) => {
            errors.push(quote_spanned! {
                ty.span() =>
                compile_error!("Trait objects (dyn Trait) are not allowed in components.\n\
                               Use marker components and entity relations for polymorphism.\n\
                               Or mark this component as #[component(opaque)] if it's a runtime-only handle.");
            });
        }
        Type::ImplTrait(_) => {
            errors.push(quote_spanned! {
                ty.span() =>
                compile_error!("impl Trait is not allowed in component fields.\n\
                               Use concrete types.\n\
                               Or mark this component as #[component(opaque)] if it's a runtime-only handle.");
            });
        }
        // Other types (Never, Infer, etc.) are generally fine or will fail elsewhere
        _ => {}
    }
}

fn check_type_path(
    path: &Path,
    span: proc_macro2::Span,
    errors: &mut Vec<proc_macro2::TokenStream>,
) {
    // Get the last segment (the actual type name)
    if let Some(segment) = path.segments.last() {
        let type_name = segment.ident.to_string();

        // Check against forbidden types
        for (forbidden, message) in FORBIDDEN_TYPES {
            if type_name == *forbidden {
                let error_msg =
                    format!("Component field uses forbidden type `{type_name}`.\n\n{message}");
                errors.push(quote_spanned! {
                    span =>
                    compile_error!(#error_msg);
                });
                return;
            }
        }

        // Check generic arguments recursively
        if let PathArguments::AngleBracketed(args) = &segment.arguments {
            for arg in &args.args {
                if let GenericArgument::Type(inner_ty) = arg {
                    check_type(inner_ty, errors);
                }
            }
        }
    }
}
