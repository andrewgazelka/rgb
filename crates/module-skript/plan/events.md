# Events

Events are triggers that execute script blocks when something happens in the game.

## Syntax

```skript
on <event pattern>:
    <statements>
```

## Priority Levels

| Priority | Order |
|----------|-------|
| lowest | First |
| low | |
| normal | Default |
| high | |
| highest | |
| monitor | Last (read-only) |

## Core Events (Phase 1)

### Player Events
| Event | Pattern | Bukkit Event |
|-------|---------|--------------|
| Join | `on join`, `on player join` | `PlayerJoinEvent` |
| Quit | `on quit`, `on disconnect` | `PlayerQuitEvent` |
| Chat | `on chat` | `AsyncPlayerChatEvent` |
| Death | `on death`, `on player death` | `PlayerDeathEvent` |
| Respawn | `on respawn` | `PlayerRespawnEvent` |

### Block Events
| Event | Pattern | Bukkit Event |
|-------|---------|--------------|
| Break | `on break`, `on mine` | `BlockBreakEvent` |
| Place | `on place` | `BlockPlaceEvent` |

### Entity Events
| Event | Pattern | Bukkit Event |
|-------|---------|--------------|
| Damage | `on damage` | `EntityDamageEvent` |
| Click | `on click`, `on right click` | `PlayerInteractEvent` |

## Event Values

Each event provides expressions:
```skript
on join:
    send "Welcome %player%!" to player
    # player, event-player available
```

| Event | Available Expressions |
|-------|----------------------|
| Join | `player`, `join message` |
| Break | `player`, `block`, `event-block` |
| Damage | `attacker`, `victim`, `damage` |

## Implementation

```rust
pub trait SkriptEvent: Send + Sync {
    /// Pattern that matches this event
    fn patterns() -> &'static [&'static str];

    /// Check if this event matches the given AST event name
    fn matches(event_name: &str) -> bool;

    /// Get available event-values for this event
    fn event_values() -> &'static [EventValue];
}

pub struct EventValue {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub value_type: ValueType,
}
```

## Flecs Integration

Events dispatch through Flecs observers:

```rust
world.observer::<PlayerJoinEvent>()
    .each(|event| {
        // Find matching script triggers
        // Execute with event context
    });
```
