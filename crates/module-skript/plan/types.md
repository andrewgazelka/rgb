# Type System

Skript uses a dynamic type system with runtime conversions.

## Core Types

| Type | Rust Representation | Examples |
|------|---------------------|----------|
| None | `()` | unset variable |
| Boolean | `bool` | `true`, `false` |
| Number | `f64` | `42`, `3.14`, `-5` |
| Text | `String` | `"hello"` |
| Player | `Entity` (Flecs) | `player` |
| Entity | `Entity` (Flecs) | `attacker`, `victim` |
| Block | `BlockState + BlockPos` | `event-block` |
| Location | `Location` struct | `location of player` |
| World | `Entity` (Flecs) | `world of player` |
| Item | `ItemStack` | `diamond sword` |
| List | `Vec<Value>` | `all players` |

## Value Enum

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    None,
    Boolean(bool),
    Number(f64),
    Text(String),
    Player(Entity),
    Entity(Entity),
    Block { state: BlockState, pos: BlockPos },
    Location(Location),
    World(Entity),
    Item(ItemStack),
    List(Vec<Value>),
}
```

## Conversions

### Implicit Conversions

```rust
impl Value {
    pub fn as_boolean(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::None => false,
            Value::Number(n) => *n != 0.0,
            Value::Text(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            _ => true,
        }
    }

    pub fn as_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            Value::Boolean(b) => if *b { 1.0 } else { 0.0 },
            Value::Text(s) => s.parse().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    pub fn as_text(&self) -> String {
        match self {
            Value::None => "".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Number(n) => format_number(*n),
            Value::Text(s) => s.clone(),
            Value::Player(e) => get_player_name(*e),
            Value::Entity(e) => get_entity_name(*e),
            Value::Location(l) => format!("{}, {}, {}", l.x, l.y, l.z),
            Value::List(l) => l.iter()
                .map(|v| v.as_text())
                .collect::<Vec<_>>()
                .join(", "),
            _ => "<unknown>".to_string(),
        }
    }

    pub fn as_players(&self) -> Vec<Entity> {
        match self {
            Value::Player(e) => vec![*e],
            Value::Entity(e) if is_player(*e) => vec![*e],
            Value::List(l) => l.iter()
                .filter_map(|v| v.as_player())
                .collect(),
            _ => vec![],
        }
    }
}
```

## Comparison

```rust
impl Value {
    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::None, Value::None) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Text(a), Value::Text(b)) => a.eq_ignore_ascii_case(b),
            (Value::Player(a), Value::Player(b)) => a == b,
            (Value::Entity(a), Value::Entity(b)) => a == b,
            // Cross-type comparisons via conversion
            (Value::Number(_), Value::Text(_)) => self.as_number() == other.as_number(),
            (Value::Text(_), Value::Number(_)) => self.as_number() == other.as_number(),
            _ => false,
        }
    }

    pub fn compare(&self, other: &Value) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::Text(a), Value::Text(b)) => Some(a.cmp(b)),
            _ => self.as_number().partial_cmp(&other.as_number()),
        }
    }
}
```

## Changers

Skript supports modifying values with changers:

| Changer | Syntax | Description |
|---------|--------|-------------|
| Set | `set %expr% to %value%` | Replace value |
| Add | `add %value% to %expr%` | Add to number/list |
| Remove | `remove %value% from %expr%` | Remove from number/list |
| Delete | `delete %expr%` | Clear/unset |
| Reset | `reset %expr%` | Reset to default |

```rust
pub enum ChangeMode {
    Set,
    Add,
    Remove,
    Delete,
    Reset,
}

pub trait Changeable {
    fn change(&mut self, mode: ChangeMode, value: &Value) -> Result<(), ChangeError>;
    fn accepts(&self, mode: ChangeMode) -> bool;
}
```

## Type Registry

For future addon support:

```rust
pub struct TypeInfo {
    pub name: &'static str,
    pub patterns: &'static [&'static str],
    pub from_string: fn(&str) -> Option<Value>,
    pub to_string: fn(&Value) -> String,
}

pub struct TypeRegistry {
    types: HashMap<&'static str, TypeInfo>,
}
```
