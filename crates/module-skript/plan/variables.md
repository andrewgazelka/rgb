# Variables

Variables store values that persist across statements and triggers.

## Syntax

```skript
{global}           # global variable, persists across scripts
{_local}           # local variable, scoped to current trigger
{list::*}          # list variable (all elements)
{list::1}          # list index access
{map::%player%}    # dynamic key (player's UUID)
{nested::a::b}     # nested keys
```

## Variable Types

| Prefix | Scope | Persistence |
|--------|-------|-------------|
| `{name}` | Global | File/database |
| `{_name}` | Local | Current trigger only |
| `{-name}` | Local | Current event only |

## List Variables

```skript
set {list::*} to 1, 2, and 3
add 4 to {list::*}
remove 2 from {list::*}
loop {list::*}:
    send "%loop-value%"
```

Operations:
- `{list::*}` - all elements
- `{list::1}` - first element (1-indexed)
- `{list::%expr%}` - dynamic index
- `size of {list::*}` - element count

## Implementation

```rust
pub struct VariableStorage {
    /// Global variables (persisted)
    globals: HashMap<String, Value>,
    /// Local variables (per-trigger)
    locals: HashMap<String, Value>,
}

impl VariableStorage {
    pub fn get(&self, name: &str, local: bool) -> Value {
        if local {
            self.locals.get(name).cloned().unwrap_or(Value::None)
        } else {
            self.globals.get(name).cloned().unwrap_or(Value::None)
        }
    }

    pub fn set(&mut self, name: &str, local: bool, value: Value) {
        if local {
            self.locals.insert(name.to_string(), value);
        } else {
            self.globals.insert(name.to_string(), value);
        }
    }

    pub fn delete(&mut self, name: &str, local: bool) {
        if local {
            self.locals.remove(name);
        } else {
            self.globals.remove(name);
        }
    }

    /// Clear local variables (call at trigger end)
    pub fn clear_locals(&mut self) {
        self.locals.clear();
    }
}
```

## List Variable Implementation

```rust
pub fn get_list_variable(storage: &VariableStorage, base: &str, indices: &[Value]) -> Value {
    if indices.is_empty() {
        // {list::*} - return all with matching prefix
        let prefix = format!("{}::", base);
        let items: Vec<Value> = storage.globals
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(_, v)| v.clone())
            .collect();
        return Value::List(items);
    }

    // Build full key
    let key = build_variable_key(base, indices);
    storage.get(&key, false)
}

fn build_variable_key(base: &str, indices: &[Value]) -> String {
    let mut key = base.to_string();
    for idx in indices {
        key.push_str("::");
        key.push_str(&idx.as_text());
    }
    key
}
```

## Persistence

### Phase 1: In-Memory Only
All variables lost on restart.

### Phase 5: File Persistence

```rust
pub trait VariablePersistence: Send + Sync {
    fn load(&self) -> HashMap<String, Value>;
    fn save(&self, variables: &HashMap<String, Value>);
}

pub struct JsonPersistence {
    path: PathBuf,
}

impl VariablePersistence for JsonPersistence {
    fn load(&self) -> HashMap<String, Value> {
        let content = std::fs::read_to_string(&self.path).ok()?;
        serde_json::from_str(&content).ok()?
    }

    fn save(&self, variables: &HashMap<String, Value>) {
        let content = serde_json::to_string_pretty(variables).unwrap();
        std::fs::write(&self.path, content).unwrap();
    }
}
```

## Thread Safety

For multi-threaded execution:

```rust
pub struct ConcurrentVariableStorage {
    globals: DashMap<String, Value>,
    locals: ThreadLocal<RefCell<HashMap<String, Value>>>,
}
```

Or use Flecs components for variable storage per-script entity.
