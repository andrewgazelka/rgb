//! The Introspectable trait for components that can be serialized to JSON.

use crate::IntrospectError;

/// Trait for components that can be serialized to/from JSON for the dashboard.
///
/// This trait is typically derived using `#[derive(Introspectable)]` on types
/// that also derive `Serialize` and `Deserialize`.
///
/// # Example
///
/// ```ignore
/// use rgb_ecs::Component;
/// use rgb_ecs_introspect::Introspectable;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Component, Clone, Serialize, Deserialize, Introspectable)]
/// pub struct Position {
///     pub x: f64,
///     pub y: f64,
///     pub z: f64,
/// }
/// ```
///
/// # Opaque Components
///
/// Components marked with `#[component(opaque)]` get a default implementation
/// where `to_json()` returns `null` and `is_opaque()` returns `true`.
pub trait Introspectable: rgb_ecs::Component + Clone + Send + Sync + 'static {
    /// Serialize this component to a JSON value.
    ///
    /// Returns `serde_json::Value::Null` for opaque components.
    fn to_json(&self) -> serde_json::Value;

    /// Deserialize a component from a JSON value.
    ///
    /// Returns an error for opaque components.
    fn from_json(value: serde_json::Value) -> Result<Self, IntrospectError>
    where
        Self: Sized;

    /// Get the JSON schema for this component type.
    ///
    /// Used by the dashboard to generate appropriate editors.
    /// Returns `None` for opaque components or if schema generation is not supported.
    fn schema() -> Option<serde_json::Value> {
        None
    }

    /// Whether this is an opaque component (cannot be serialized).
    fn is_opaque() -> bool {
        false
    }

    /// Get a human-readable summary for opaque components.
    ///
    /// Override this to provide useful info like byte sizes, handle counts, etc.
    /// Returns `None` by default (no summary available).
    fn opaque_info(&self) -> Option<String> {
        None
    }

    /// Get the short type name (without module path).
    fn type_name() -> &'static str {
        let full = core::any::type_name::<Self>();
        full.rsplit("::").next().unwrap_or(full)
    }

    /// Get the full type name (with module path).
    fn full_type_name() -> &'static str {
        core::any::type_name::<Self>()
    }
}

/// Blanket implementation for opaque components.
///
/// This macro creates an Introspectable impl that returns null/errors.
/// Used by the derive macro for `#[component(opaque)]` types.
#[macro_export]
macro_rules! impl_opaque_introspectable {
    ($ty:ty) => {
        impl $crate::Introspectable for $ty {
            fn to_json(&self) -> serde_json::Value {
                serde_json::Value::Null
            }

            fn from_json(_value: serde_json::Value) -> Result<Self, $crate::IntrospectError> {
                Err($crate::IntrospectError::OpaqueComponent(
                    Self::type_name().to_string(),
                ))
            }

            fn is_opaque() -> bool {
                true
            }
        }
    };
}
