# RGB Event System Implementation

## Phase 1: Core Event Infrastructure

- [ ] Create `Cargo.toml` with workspace dependencies
- [ ] Define `Event` trait with `GlobalEvent`, `SpatialEvent`, `TargetedEvent` markers
- [ ] Implement `EventId` and `EventMeta` for type-erased event storage
- [ ] Create `EventQueue` with RGB color buckets (red, green, blue, global)
- [ ] Implement `EventPtr` for safe event access

## Phase 2: Handler System

- [ ] Define `Handler` trait with `init`, `run`, `refresh_archetype` methods
- [ ] Implement `HandlerInfo` for handler metadata
- [ ] Create `HandlerParam` trait for handler function parameters
- [ ] Implement `Receiver<E>` - receives the event being handled
- [ ] Implement `Fetcher<Q>` - fetch components from entities
- [ ] Implement `Single<Q>` - access global/world components
- [ ] Implement `Sender<E>` - emit new events from handlers
- [ ] Create handler registration on World

## Phase 3: Function Handler Conversion

- [ ] Implement `IntoHandler` for `Fn` types
- [ ] Create macro or trait magic for extracting params from function signature
- [ ] Support handlers with 1-8 parameters

## Phase 4: RGB Execution

- [ ] Implement `cell_color(x, z) -> Color` function
- [ ] Add `flush_global_events()` method
- [ ] Add `flush_spatial_events(color)` method
- [ ] Implement full `tick()` with Global → R → G → B → Global phases
- [ ] Add audio profiling hooks per phase

## Phase 5: Integration

- [ ] Add rgb-event to workspace
- [ ] Integrate with rgb-ecs World
- [ ] Convert one system (e.g., movement) to event handler
- [ ] Test end-to-end with mc-server-runner

## Testing

- [ ] Unit test: EventQueue push/pop by color
- [ ] Unit test: Handler registration and lookup
- [ ] Unit test: cell_color computation for various positions
- [ ] Integration test: Simple event → handler → component update
- [ ] Integration test: Cross-color event forwarding (attack → damage)
