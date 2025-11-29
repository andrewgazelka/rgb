# rgb

A Minecraft server written in Rust, developed with Claude Opus 4.5.

Fast. Modular. AI-native. Just works.

**Currently targeting:** Minecraft snapshot (development builds). We develop against snapshots to stay ahead—when a new Minecraft version drops, we're ready.

## Why rgb?

Building Minecraft servers is hard. We make it easy. With Opus 4.5 driving development, rgb delivers a clean, performant, and hackable server that stays current with Minecraft's latest snapshots.

## Why AI?

Opus 4.5 is genuinely good at writing Rust. Like, *really* good. We're building rgb to take full advantage of that.

The key insight: **AI needs to test autonomously.** That means both a client and a server, written in Rust, that can spin up and talk to each other without human intervention. No launching a Java client manually. No clicking through menus. Just `cargo test` and everything works.

This enables:
- **Rapid iteration** — Opus writes code, runs tests, fixes issues, repeats
- **Full coverage** — Every packet, every edge case, tested automatically
- **Confident refactoring** — Break something? The tests catch it immediately

**The LLM is the abstraction, not the code.**

Traditional codebases add layers of abstraction to help humans manage complexity. But LLMs don't need that—they can hold the entire context in memory and reason about low-level details directly. So we keep the code flat and explicit. Fewer abstractions means fewer indirections for the AI to trace through, and more direct control over what actually happens.

This isn't about dumbing things down. It's about giving Opus full power over the system. When the AI can see exactly what every line does, it can make better decisions, catch more bugs, and write more efficient code.

## Deobfuscated Minecraft

Starting with snapshot **25w46a**, Minecraft removed obfuscation from their source code. This is huge for us—we can now decompile the official client/server and read actual class and method names instead of `a.b.c()`.

This makes implementing the protocol straightforward: just look at what Mojang does and do the same thing in Rust.

## Goals

<img src=".github/assets/ai-first.svg" width="20" height="20" align="top"/> **AI-First** — Built with Opus 4.5. Easy to test, extend, and understand. Designed for AI-assisted development from day one.

<img src=".github/assets/performant.svg" width="20" height="20" align="top"/> **Performant & Succinct** — Uses the RGB region coloring algorithm. High performance without sacrificing code clarity.

<img src=".github/assets/modular.svg" width="20" height="20" align="top"/> **Modular** — Import what you need, skip what you don't. Build your own custom server from composable parts.

<img src=".github/assets/self-contained.svg" width="20" height="20" align="top"/> **Self-Contained** — No waiting on upstream maintainers. We own the full stack.

<img src=".github/assets/latest.svg" width="20" height="20" align="top"/> **Always Latest** — Targets the newest Minecraft snapshot. No waiting for updates.

<img src=".github/assets/rust.svg" width="20" height="20" align="top"/> **Pure Rust** — No mixed-language dependencies. One toolchain, zero friction.

<img src=".github/assets/vanilla.svg" width="20" height="20" align="top"/> **Full Vanilla** — Complete vanilla implementation. Every feature, every mechanic—modular but complete.

<img src=".github/assets/cross-platform.svg" width="20" height="20" align="top"/> **Cross-Platform** — Runs on Linux, macOS, and Windows.

<img src=".github/assets/monorepo.svg" width="20" height="20" align="top"/> **Monorepo** — Everything in one place. Clone once, build everything.
