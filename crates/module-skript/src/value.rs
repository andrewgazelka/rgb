//! Runtime value types for Skript.

use flecs_ecs::core::Entity;

/// A runtime value in Skript.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Value {
    /// No value / unset.
    #[default]
    None,
    /// Boolean value.
    Boolean(bool),
    /// Numeric value (all numbers are f64 in Skript).
    Number(f64),
    /// Text/string value.
    Text(String),
    /// Player entity.
    Player(Entity),
    /// Generic entity.
    Entity(Entity),
    /// List of values.
    List(Vec<Value>),
}

impl Value {
    /// Convert to boolean (truthiness).
    #[must_use]
    pub fn as_boolean(&self) -> bool {
        match self {
            Self::None => false,
            Self::Boolean(b) => *b,
            Self::Number(n) => *n != 0.0,
            Self::Text(s) => !s.is_empty(),
            Self::List(l) => !l.is_empty(),
            Self::Player(_) | Self::Entity(_) => true,
        }
    }

    /// Convert to number.
    #[must_use]
    pub fn as_number(&self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Boolean(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Number(n) => *n,
            Self::Text(s) => s.parse().unwrap_or(0.0),
            Self::List(l) => l.len() as f64,
            Self::Player(_) | Self::Entity(_) => 0.0,
        }
    }

    /// Convert to text.
    #[must_use]
    pub fn as_text(&self) -> String {
        match self {
            Self::None => String::new(),
            Self::Boolean(b) => b.to_string(),
            Self::Number(n) => format_number(*n),
            Self::Text(s) => s.clone(),
            Self::Player(e) | Self::Entity(e) => format!("entity:{}", e.0),
            Self::List(l) => l.iter().map(Self::as_text).collect::<Vec<_>>().join(", "),
        }
    }

    /// Check equality with another value.
    #[must_use]
    pub fn equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Self::Text(a), Self::Text(b)) => a.eq_ignore_ascii_case(b),
            (Self::Player(a), Self::Player(b)) | (Self::Entity(a), Self::Entity(b)) => a == b,
            // Cross-type: try numeric comparison
            (Self::Number(_), Self::Text(_)) | (Self::Text(_), Self::Number(_)) => {
                (self.as_number() - other.as_number()).abs() < f64::EPSILON
            }
            _ => false,
        }
    }
}

/// Format a number for display, avoiding unnecessary decimals.
fn format_number(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_conversion() {
        assert!(!Value::None.as_boolean());
        assert!(!Value::Boolean(false).as_boolean());
        assert!(Value::Boolean(true).as_boolean());
        assert!(!Value::Number(0.0).as_boolean());
        assert!(Value::Number(1.0).as_boolean());
        assert!(!Value::Text(String::new()).as_boolean());
        assert!(Value::Text("hello".to_string()).as_boolean());
    }

    #[test]
    fn test_number_conversion() {
        assert!((Value::None.as_number() - 0.0).abs() < f64::EPSILON);
        assert!((Value::Boolean(true).as_number() - 1.0).abs() < f64::EPSILON);
        assert!((Value::Number(42.0).as_number() - 42.0).abs() < f64::EPSILON);
        assert!((Value::Text("3.15".to_string()).as_number() - 3.15).abs() < f64::EPSILON);
    }

    #[test]
    fn test_equality() {
        assert!(Value::None.equals(&Value::None));
        assert!(Value::Number(42.0).equals(&Value::Number(42.0)));
        assert!(Value::Text("Hello".to_string()).equals(&Value::Text("hello".to_string())));
        assert!(Value::Number(42.0).equals(&Value::Text("42".to_string())));
    }
}
