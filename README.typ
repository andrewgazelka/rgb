#set document(title: "RGB: Parallel Game of Life with Flecs ECS", author: "Andrew Gazelka")
#set page(margin: 2cm)
#set text(font: "New Computer Modern", size: 11pt)
#set heading(numbering: "1.")

#align(center)[
  #text(size: 18pt, weight: "bold")[RGB: Parallel Game of Life with Flecs ECS]
  #v(0.5em)
  #text(size: 12pt)[A chunked, infinite-world cellular automaton using entity relationships]
  #v(1em)
]

= Overview

RGB is a Conway's Game of Life implementation designed for parallel simulation of an infinite world. It uses the *Flecs* Entity Component System for data management and *wgpu* for GPU-accelerated rendering.

The key insight is the *9-color region scheme*: by partitioning the world into 4×4 chunk regions colored like a 3×3 checkerboard pattern, chunks of the same color never share edges. This enables embarrassingly parallel simulation within each color.

= Architecture

== Crate Structure

```
rgb/
├── rgb-core/      # ECS components, chunk management
├── rgb-life/      # Game of Life simulation rules
├── rgb-app/       # wgpu renderer, winit window
├── mc-protocol/   # Protocol encoding/decoding traits
└── mc-packets/    # Generated Minecraft packet structs
```

== Minecraft Protocol Generation

The `mc-packets` crate contains auto-generated Minecraft protocol packet structs. Run:
```bash
nix run .#mc-gen
```
This uses the unobfuscated Minecraft client (25w46a+) to extract packet field info via reflection, then generates Rust structs with proper types.

== Core Components

*Chunks* are 16×16 grids of cells stored as `CellData`:
```rust
#[derive(Component)]
pub struct CellData {
    pub cells: [[bool; 16]; 16],
}
```

*Double buffering* via `NextCellData` enables safe parallel updates---read from current, write to next, then swap.

*Neighbor relationships* use 8 Flecs relationship components:
```rust
#[derive(Component)] pub struct NeighborN;  // North
#[derive(Component)] pub struct NeighborS;  // South
// ... NE, NW, SE, SW, E, W
```

A chunk accesses its neighbor via:
```rust
entity.target(NeighborN, 0)  // O(1) lookup
```

= The 9-Color Scheme

Regions are 4×4 chunks. Each region is assigned a color (0--8) based on:
```rust
color = (region.y % 3) * 3 + (region.x % 3)
```

This creates a repeating 3×3 pattern where no two adjacent regions share a color:
```
0 1 2 0 1 2
3 4 5 3 4 5
6 7 8 6 7 8
```

*Property*: Chunks with the same `SimColor` never share edges, so they can be simulated in parallel without data races.

= Simulation Pipeline

Each simulation step:

1. *Expand*: Check edge activity, spawn empty neighbor chunks where needed
2. *Compute*: For each active chunk, count neighbors and apply Conway's rules to `NextCellData`
3. *Swap*: Copy `NextCellData` → `CellData`
4. *Update*: Mark chunks as Active/Inactive based on live cell count

Conway's rules:
- Live cell with 2--3 neighbors survives
- Dead cell with exactly 3 neighbors becomes alive
- All other cells die

= Rendering

== Texture Atlas

All chunks share a single 4096×4096 texture atlas (256×256 slots). Each chunk occupies a 16×16 pixel region.

Benefits:
- Single draw call for all chunks
- Dynamic allocation as world expands
- Efficient GPU memory usage

== Region Borders

The fragment shader draws colored borders on region boundaries:
```wgsl
if is_region_edge && near_border {
    return vec4(region_color, 1.0);
}
```

Each of the 9 colors gets a distinct RGB value, visualizing the parallel processing structure.

== Instanced Rendering

Each chunk is a single quad instance with:
- World position
- Atlas UV coordinates
- Region color (for border)
- Position within region (for edge detection)

= Controls

#table(
  columns: 2,
  [*Key*], [*Action*],
  [WASD / Arrows], [Move camera],
  [Q / E], [Zoom out / in],
  [Space], [Pause / Resume],
  [N], [Single step (when paused)],
  [R], [Reset simulation],
)

= Performance Considerations

- *Lazy chunk creation*: Only chunks with edge activity spawn neighbors
- *Active tag*: Empty chunks skip simulation
- *Dirty tag*: Only upload textures for modified chunks
- *Relationship-based lookup*: O(1) neighbor access via Flecs

= Dependencies

- `flecs_ecs` --- Entity Component System
- `wgpu` --- Cross-platform GPU API
- `winit` --- Window management
- `bytemuck` --- Safe casting for GPU buffers

#v(1em)
#line(length: 100%)
#align(center)[
  #text(size: 9pt, fill: gray)[
    Built with Flecs-Rust and wgpu. MIT License.
  ]
]
