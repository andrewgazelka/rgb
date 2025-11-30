# Expressions

Expressions are values that can be used in effects, conditions, and other expressions.

## Syntax

```skript
player                      # simple expression
player's health             # property expression
{_variable}                 # variable
health of player            # alternative property syntax
all players                 # collection expression
random number between 1 and 10
```

## Core Expressions (Phase 1-2)

### Event Values
| Expression | Type | Events |
|------------|------|--------|
| `player`, `event-player` | Player | Most player events |
| `block`, `event-block` | Block | Block events |
| `attacker` | Entity | Damage events |
| `victim` | Entity | Damage events |
| `message` | Text | Chat events |

### Literals
| Expression | Type | Example |
|------------|------|---------|
| Number | Number | `42`, `3.14` |
| String | Text | `"hello"`, `"Hello %player%"` |
| Boolean | Boolean | `true`, `false` |

### Player Properties
| Expression | Pattern | Type |
|------------|---------|------|
| Health | `health of %player%` | Number |
| Name | `name of %player%` | Text |
| UUID | `uuid of %player%` | Text |
| Location | `location of %entity%` | Location |
| World | `world of %entity%` | World |
| GameMode | `gamemode of %player%` | GameMode |
| Level | `level of %player%` | Number |
| Food | `food level of %player%` | Number |

### Collections
| Expression | Pattern | Type |
|------------|---------|------|
| All Players | `all players` | Players |
| Online Players | `online players` | Players |
| Players in World | `players in %world%` | Players |

### Math
| Expression | Pattern | Type |
|------------|---------|------|
| Random | `random (number|integer) between %n% and %n%` | Number |
| Abs | `abs(%number%)` | Number |
| Floor | `floor(%number%)` | Number |
| Ceil | `ceil(%number%)` | Number |
| Round | `round(%number%)` | Number |

### String Operations
| Expression | Pattern | Type |
|------------|---------|------|
| Join | `%texts% joined by %text%` | Text |
| Length | `length of %text%` | Number |
| Lowercase | `lowercase %text%` | Text |
| Uppercase | `uppercase %text%` | Text |

## Variables

```skript
{global_var}      # global variable
{_local_var}      # local to trigger
{list::*}         # list variable
{list::%player%}  # dynamic index
```

## Implementation

```rust
pub enum Value {
    None,
    Boolean(bool),
    Number(f64),
    String(String),
    Player(Entity),      // Flecs entity
    Entity(Entity),
    Block(BlockPos),
    Location(Location),
    List(Vec<Value>),
}

pub fn evaluate_expr(expr: &Expr, ctx: &ExecutionContext) -> Value {
    match expr {
        Expr::Literal(lit) => match &lit.kind {
            LiteralKind::Number(n) => Value::Number(*n),
            LiteralKind::String(s) => Value::String(s.to_string()),
            LiteralKind::Boolean(b) => Value::Boolean(*b),
        },

        Expr::Ident(name, _) => {
            // Event values
            match *name {
                "player" | "event-player" => ctx.event_player(),
                "block" | "event-block" => ctx.event_block(),
                _ => Value::None,
            }
        }

        Expr::Variable(var) => {
            ctx.get_variable(&var.name, var.local)
        }

        Expr::Property { object, property, .. } => {
            let obj = evaluate_expr(object, ctx);
            get_property(&obj, property)
        }

        Expr::Binary { left, op, right, .. } => {
            let l = evaluate_expr(left, ctx);
            let r = evaluate_expr(right, ctx);
            apply_binary_op(*op, l, r)
        }

        Expr::InterpolatedString { parts, .. } => {
            let mut result = String::new();
            for part in parts {
                match part {
                    StringPart::Literal(s) => result.push_str(s),
                    StringPart::Expr(e) => {
                        result.push_str(&evaluate_expr(e, ctx).to_string())
                    }
                }
            }
            Value::String(result)
        }

        // ...
    }
}
```

## Type Conversions

Skript is dynamically typed with implicit conversions:

| From | To | Conversion |
|------|----|------------|
| Number | Text | `"42"` |
| Boolean | Text | `"true"` / `"false"` |
| Player | Text | Player name |
| Player | Entity | Upcast |
| Entity | Location | Entity location |
