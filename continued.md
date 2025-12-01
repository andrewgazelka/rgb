is there anything that is better that u can do? look at evenio and flecs architecture? if there is add and repeat

remember... ALWAYS clean up; always improve. If something is not great or could be improved even if major architectural overhaul you are ok to do this


# TODO

- ~~we should have the notion of Modules in our ECS similar to bevy/flecs btw so we can have many separte cates that we all include; do we have this... can we?~~ **DONE** - Added `Plugin` trait
- ~~add AttackModule~~ **DONE** - Added `systems/attack.rs` with Interact packet parsing (ATTACK action type)
- ~~add CommandMoudle (that allows improting clap as a command in Minecraft make sure Minecraft will send proper command tree based on this / etc)~~ **DONE** - Added `systems/command.rs` with:
  - Minecraft command tree packet generation (ClientboundCommandsPacket)
  - Commands: `/tps`, `/pos`, `/entities`, `/inspect`, `/history`
  - Sent to client on play state entry for tab completion
- ~~add command that allows for reading the state of a given entity and prints (minecraft command) like all the components~~ **DONE** - `/inspect @s` or `/inspect <entity_id>`
- ~~add command that allows for reading the HISTORY state of a given (entity, componentId) similar to flecs query~~ **DONE** - Added `systems/history.rs` with:
  - `ComponentHistory` using BTreeMap for O(log n) tick-based lookups
  - `HistoryRegistry` for tracking which entities have history enabled
  - Note: For persistent history, use `rgb_storage::VersionedWorld` which stores every mutation in Nebari's B+tree
- add in-depth miri tests
- add in-depth integration tests
- compare how we are doing regions,grouping with how flecs does group-by and how others might and determine the most efficient way to do the RGB stuff given that players / etc might move between regions often
- do a bit if tango benchmarking

# Things added

- **Query API** (`world.query::<T>()`) - Iterate entities with specific components using archetype-based iteration
- **Plugin trait** (`rgb_ecs::Plugin`) - Modular ECS setup like Bevy/Flecs (`world.add_plugin(MyPlugin)`)
- **EventPlugin** (`rgb_event::EventPlugin`) - Event system as a proper plugin
- **Refactored mc-server-runner** - Now uses queries instead of manual ConnectionIndex iteration:
  - All systems now use `world.query::<ComponentType>()` instead of iterating `conn_index.map.values()`
  - Cleaner, more idiomatic ECS patterns
  - Better separation of concerns
- **Attack System** (`systems/attack.rs`) - Handles player attacks via ServerboundInteractPacket:
  - Parses Interact packet with ATTACK action type
  - Logs attacks with attacker/target names
  - Ready for Health component and damage events
- **Command System** (`systems/command.rs`) - Full Minecraft command integration:
  - Generates ClientboundCommandsPacket for tab completion
  - Commands: `/tps` (server stats), `/pos` (position), `/entities` (list), `/inspect` (components), `/history` (tracking)
  - Parses ServerboundChatCommandPacket
  - Returns colored chat responses
- **History Tracking** (`systems/history.rs`) - Component change tracking:
  - `ComponentHistory` with BTreeMap for O(log n) tick lookups
  - `HistoryRegistry` to enable/disable per-entity tracking
  - Query by tick, tick range, or last N entries
  - Integrates with `rgb_storage::VersionedWorld` for persistent time-travel
- **Query DSL** (`crates/query-dsl`) - Flecs-like query language parser:
  - Syntax: `Position, Velocity` (AND), `!Dead` (NOT), `?Health` (OPTIONAL), `A || B` (OR)
  - Pairs: `(ChildOf, Player)` for relationships
  - Wildcard: `*` to match all entities
  - Used by `/entities` command: `/entities Player, Position` or `/entities *`
