# Effects

Effects are actions that execute unconditionally when reached.

## Syntax

Effects are statements that perform actions:
```skript
send "Hello" to player
broadcast "Server message"
cancel event
```

## Core Effects (Phase 1-2)

### Communication
| Effect | Pattern | Description |
|--------|---------|-------------|
| Send | `send %texts% [to %players%]` | Send message to player(s) |
| Broadcast | `broadcast %texts%` | Send to all players |
| ActionBar | `send action bar %text% to %players%` | Action bar message |
| Title | `send title %text% [with subtitle %text%] to %players%` | Title screen |

### Event Control
| Effect | Pattern | Description |
|--------|---------|-------------|
| Cancel | `cancel [the] event` | Cancel current event |

### Entity Manipulation
| Effect | Pattern | Description |
|--------|---------|-------------|
| Teleport | `teleport %entities% to %location%` | Move entity |
| Kill | `kill %entities%` | Kill entity |
| Damage | `damage %entities% by %number%` | Deal damage |
| Heal | `heal %entities% [by %number%]` | Restore health |

### Inventory
| Effect | Pattern | Description |
|--------|---------|-------------|
| Give | `give %items% to %players%` | Give items |
| Remove | `remove %items% from %players%` | Take items |
| Clear | `clear [inventory of] %players%` | Clear inventory |

### Control Flow
| Effect | Pattern | Description |
|--------|---------|-------------|
| Stop | `stop` | Exit current trigger |
| Continue | `continue` | Next loop iteration |
| Return | `return [%value%]` | Return from function |

## Implementation

```rust
pub trait SkriptEffect: Send + Sync {
    /// Execute this effect
    fn execute(&self, ctx: &mut ExecutionContext) -> EffectResult;
}

pub enum EffectResult {
    Continue,
    Stop,
    Return(Value),
}

// Example: Send effect
pub struct SendEffect {
    pub message: Expr,
    pub target: Option<Expr>,
}

impl SkriptEffect for SendEffect {
    fn execute(&self, ctx: &mut ExecutionContext) -> EffectResult {
        let message = self.message.evaluate(ctx)?;
        let targets = self.target
            .as_ref()
            .map(|t| t.evaluate(ctx))
            .unwrap_or_else(|| ctx.event_player());

        for player in targets.as_players() {
            player.send_message(&message.as_string());
        }

        EffectResult::Continue
    }
}
```

## AST Mapping

From `skript-lang` AST:
```rust
match stmt {
    Stmt::Effect(Effect { kind, .. }) => match kind {
        EffectKind::Send { message, target } => {
            SendEffect { message, target }.execute(ctx)
        }
        EffectKind::Broadcast { message } => {
            BroadcastEffect { message }.execute(ctx)
        }
        EffectKind::Cancel => {
            ctx.cancel_event();
            EffectResult::Continue
        }
        // ...
    }
}
```
