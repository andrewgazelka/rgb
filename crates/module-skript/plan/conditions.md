# Conditions

Conditions are boolean expressions that gate execution.

## Syntax

```skript
if player is alive:
    send "You live!"
else:
    send "You died!"

# As statement (guard)
player has permission "admin"
send "Admin only message"  # only runs if condition passed
```

## Core Conditions (Phase 1-2)

### Comparison
| Condition | Pattern | Description |
|-----------|---------|-------------|
| Is | `%objects% is %objects%` | Equality check |
| IsNot | `%objects% is not %objects%` | Inequality check |
| Contains | `%objects% contains %objects%` | Collection membership |
| IsIn | `%objects% is in %objects%` | Reverse contains |

### Numeric
| Condition | Pattern | Description |
|-----------|---------|-------------|
| Greater | `%number% > %number%` | Greater than |
| Less | `%number% < %number%` | Less than |
| GreaterEq | `%number% >= %number%` | Greater or equal |
| LessEq | `%number% <= %number%` | Less or equal |

### Entity State
| Condition | Pattern | Description |
|-----------|---------|-------------|
| IsAlive | `%entity% is alive` | Not dead |
| IsDead | `%entity% is dead` | Dead |
| IsBurning | `%entity% is burning` | On fire |
| IsSneaking | `%player% is sneaking` | Sneaking |
| IsFlying | `%player% is flying` | Flying |

### Permissions
| Condition | Pattern | Description |
|-----------|---------|-------------|
| HasPermission | `%player% has permission %text%` | Permission check |
| IsOp | `%player% is op` | Operator check |

### Type Checking
| Condition | Pattern | Description |
|-----------|---------|-------------|
| IsPlayer | `%entity% is a player` | Entity type check |
| IsBlock | `%object% is a block` | Block check |

### Variables
| Condition | Pattern | Description |
|-----------|---------|-------------|
| IsSet | `%variable% is set` | Variable exists |
| IsEmpty | `%objects% is empty` | Empty collection |

## Logical Operators

```skript
if player is alive and player has permission "vip":
if health < 5 or player is burning:
if not player is op:
```

## Implementation

```rust
pub trait SkriptCondition: Send + Sync {
    /// Evaluate this condition
    fn check(&self, ctx: &ExecutionContext) -> bool;
}

// From AST
pub fn evaluate_condition(cond: &Condition, ctx: &ExecutionContext) -> bool {
    let result = match &cond.kind {
        ConditionKind::Is(left, right) => {
            let l = evaluate_expr(left, ctx);
            let r = evaluate_expr(right, ctx);
            l.equals(&r)
        }
        ConditionKind::IsNot(left, right) => {
            let l = evaluate_expr(left, ctx);
            let r = evaluate_expr(right, ctx);
            !l.equals(&r)
        }
        ConditionKind::Compare { left, op, right } => {
            let l = evaluate_expr(left, ctx).as_number();
            let r = evaluate_expr(right, ctx).as_number();
            match op {
                CompareOp::Lt => l < r,
                CompareOp::Gt => l > r,
                CompareOp::LtEq => l <= r,
                CompareOp::GtEq => l >= r,
                _ => false,
            }
        }
        // ...
    };

    if cond.negated { !result } else { result }
}
```

## Guard Conditions

Conditions as statements act as guards:
```skript
on chat:
    player is op          # guard - stops if false
    broadcast "OP spoke!"  # only runs if guard passed
```

Implementation:
```rust
match stmt {
    Stmt::Condition(cond) => {
        if !evaluate_condition(cond, ctx) {
            return EffectResult::Stop;
        }
        EffectResult::Continue
    }
}
```
