# Skript Implementation Plan

This document outlines the implementation plan for a Rust-native Skript runtime for Minecraft servers.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        module-skript                            │
│  (Flecs module - runtime, execution, Minecraft integration)    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        skript-lang                              │
│           (Parser, Lexer, AST - pure language frontend)         │
└─────────────────────────────────────────────────────────────────┘
```

## Components

| Component | Status | Plan |
|-----------|--------|------|
| Parser/Lexer | ✅ Done | [skript-lang](../../skript-lang/) |
| Type System | 🔲 Todo | [types.md](./types.md) |
| Events | 🔲 Todo | [events.md](./events.md) |
| Effects | 🔲 Todo | [effects.md](./effects.md) |
| Conditions | 🔲 Todo | [conditions.md](./conditions.md) |
| Expressions | 🔲 Todo | [expressions.md](./expressions.md) |
| Variables | 🔲 Todo | [variables.md](./variables.md) |
| Functions | 🔲 Todo | [functions.md](./functions.md) |

## Implementation Phases

### Phase 1: Core Runtime
- [ ] Script loading and compilation
- [ ] Event dispatch system
- [ ] Basic expression evaluation
- [ ] Variable storage (in-memory)

### Phase 2: Basic Effects & Conditions
- [ ] `send` / `broadcast`
- [ ] `cancel event`
- [ ] `is` / `is not` conditions
- [ ] `player` expression

### Phase 3: Control Flow
- [ ] `if` / `else`
- [ ] `loop` (times, collection iteration)
- [ ] `while`
- [ ] `stop` / `continue`

### Phase 4: Extended Syntax
- [ ] `set` / `add` / `remove` changers
- [ ] Property expressions (`player's health`)
- [ ] More effects (teleport, give, kill, spawn)
- [ ] More conditions (has permission, contains)

### Phase 5: Advanced Features
- [ ] Functions with parameters and return types
- [ ] Commands (`command /foo:`)
- [ ] Persistent variables (file/database)
- [ ] Delays (`wait 1 second`)

### Phase 6: Compatibility & Polish
- [ ] Error messages with source locations
- [ ] Script hot-reloading
- [ ] Addon/extension API
- [ ] Documentation generation

## Design Decisions

### Pattern Matching vs Direct AST

Original Skript uses runtime pattern matching against registered syntax strings.
We use a typed AST from `skript-lang` instead:

| Approach | Pros | Cons |
|----------|------|------|
| Pattern matching | Flexible, addon-friendly | Slow, complex, runtime errors |
| Typed AST | Fast, compile-time safety | Less dynamic, harder to extend |

**Decision**: Start with typed AST, consider pattern-based addon layer later.

### Flecs Integration

The runtime integrates as a Flecs module:
- Events map to Flecs observers/systems
- Variables can be ECS components
- Script state stored in entities

### Type System

Skript has a dynamic type system with conversions. We'll use:
- Rust enums for value types
- Trait-based converters
- Compile-time type checking where possible

## Reference

- Original Skript: https://github.com/SkriptLang/Skript
- Skript docs: https://docs.skriptlang.org/
- skript-lang crate: `crates/skript-lang/`
