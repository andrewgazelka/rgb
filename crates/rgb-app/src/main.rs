mod renderer;

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use flecs_ecs::prelude::*;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use renderer::{atlas_uv, chunk_in_region, chunk_world_pos, region_color_rgb, ChunkInstance, Renderer};
use rgb_core::{
    link_chunk_neighbors, spawn_chunk, CellData, ChunkIndex, ChunkPos, Color, Dirty, CHUNK_SIZE,
};
use rgb_life::{expand_world, register_life_systems};

struct Camera {
    position: [f32; 2],
    zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            zoom: 1.0,
        }
    }
}

struct GameState {
    world: World,
    chunk_index: ChunkIndex,
    camera: Camera,
    paused: bool,
    last_step: Instant,
    step_interval: Duration,
    keys_pressed: HashSet<KeyCode>,
}

impl GameState {
    fn new() -> Self {
        let world = World::new();

        // Register life systems
        register_life_systems(&world);

        let mut chunk_index = ChunkIndex::default();

        // Create initial pattern
        create_initial_pattern(&world, &mut chunk_index);

        Self {
            world,
            chunk_index,
            camera: Camera::default(),
            paused: false,
            last_step: Instant::now(),
            step_interval: Duration::from_millis(100),
            keys_pressed: HashSet::new(),
        }
    }

    fn step(&mut self) {
        // First expand the world to create new chunks where needed
        expand_world(&self.world, &mut self.chunk_index);
        // Then run the simulation
        self.world.progress();
    }

    fn update(&mut self, dt: f32) {
        // Camera movement
        let speed = 200.0 * self.camera.zoom;

        if self.keys_pressed.contains(&KeyCode::KeyW)
            || self.keys_pressed.contains(&KeyCode::ArrowUp)
        {
            self.camera.position[1] += speed * dt;
        }
        if self.keys_pressed.contains(&KeyCode::KeyS)
            || self.keys_pressed.contains(&KeyCode::ArrowDown)
        {
            self.camera.position[1] -= speed * dt;
        }
        if self.keys_pressed.contains(&KeyCode::KeyA)
            || self.keys_pressed.contains(&KeyCode::ArrowLeft)
        {
            self.camera.position[0] -= speed * dt;
        }
        if self.keys_pressed.contains(&KeyCode::KeyD)
            || self.keys_pressed.contains(&KeyCode::ArrowRight)
        {
            self.camera.position[0] += speed * dt;
        }

        // Zoom
        if self.keys_pressed.contains(&KeyCode::KeyQ) {
            self.camera.zoom *= 1.0 + dt;
        }
        if self.keys_pressed.contains(&KeyCode::KeyE) {
            self.camera.zoom *= 1.0 - dt;
            self.camera.zoom = self.camera.zoom.max(0.1);
        }

        // Simulation step
        if !self.paused && self.last_step.elapsed() >= self.step_interval {
            self.step();
            self.last_step = Instant::now();
        }
    }

    fn handle_key(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            self.keys_pressed.insert(key);

            match key {
                KeyCode::Space => {
                    self.paused = !self.paused;
                    log::info!("Paused: {}", self.paused);
                }
                KeyCode::KeyN if self.paused => {
                    self.step();
                    log::info!("Manual step");
                }
                KeyCode::KeyR => {
                    // Reset
                    self.chunk_index.map.clear();

                    // Delete all chunk entities
                    self.world.each_entity::<&ChunkPos>(|e, _| {
                        e.destruct();
                    });

                    // Create new pattern
                    create_initial_pattern(&self.world, &mut self.chunk_index);
                    log::info!("Reset");
                }
                _ => {}
            }
        } else {
            self.keys_pressed.remove(&key);
        }
    }
}

/// Helper to add a chunk with cells
fn add_chunk(world: &World, index: &mut ChunkIndex, pos: ChunkPos, cells: CellData) {
    let chunk = spawn_chunk(world, pos, cells);
    index.map.insert(pos, chunk.id());
    link_chunk_neighbors(world, chunk.id(), pos, index);
}

/// Create a glider pattern at the given position within a chunk
fn glider_cells(offset_x: usize, offset_y: usize) -> CellData {
    let mut cells = CellData::default();
    // Glider shape:
    //   X
    // X X
    // XX
    if offset_x + 2 < CHUNK_SIZE && offset_y + 2 < CHUNK_SIZE {
        cells.set(offset_x + 1, offset_y, true);
        cells.set(offset_x + 2, offset_y + 1, true);
        cells.set(offset_x, offset_y + 2, true);
        cells.set(offset_x + 1, offset_y + 2, true);
        cells.set(offset_x + 2, offset_y + 2, true);
    }
    cells
}

/// Create an R-pentomino pattern (chaotic long-lived pattern)
fn r_pentomino_cells(offset_x: usize, offset_y: usize) -> CellData {
    let mut cells = CellData::default();
    //  XX
    // XX
    //  X
    if offset_x + 2 < CHUNK_SIZE && offset_y + 2 < CHUNK_SIZE {
        cells.set(offset_x + 1, offset_y, true);
        cells.set(offset_x + 2, offset_y, true);
        cells.set(offset_x, offset_y + 1, true);
        cells.set(offset_x + 1, offset_y + 1, true);
        cells.set(offset_x + 1, offset_y + 2, true);
    }
    cells
}

/// Create a Gosper glider gun (shoots gliders!)
fn gosper_gun_cells() -> CellData {
    let mut cells = CellData::default();
    // The glider gun is 36x9, but we only have 16x16 - let's use part of it
    // or create a simpler pattern. Let's create a lightweight spaceship instead.

    // Lightweight spaceship (LWSS)
    // .X..X
    // X....
    // X...X
    // XXXX.
    cells.set(1, 0, true);
    cells.set(4, 0, true);
    cells.set(0, 1, true);
    cells.set(0, 2, true);
    cells.set(4, 2, true);
    cells.set(0, 3, true);
    cells.set(1, 3, true);
    cells.set(2, 3, true);
    cells.set(3, 3, true);
    cells
}

/// Acorn pattern - small but grows to ~633 cells over 5206 generations
fn acorn_cells(offset_x: usize, offset_y: usize) -> CellData {
    let mut cells = CellData::default();
    // .X.....
    // ...X...
    // XX..XXX
    if offset_x + 6 < CHUNK_SIZE && offset_y + 2 < CHUNK_SIZE {
        cells.set(offset_x + 1, offset_y, true);
        cells.set(offset_x + 3, offset_y + 1, true);
        cells.set(offset_x, offset_y + 2, true);
        cells.set(offset_x + 1, offset_y + 2, true);
        cells.set(offset_x + 4, offset_y + 2, true);
        cells.set(offset_x + 5, offset_y + 2, true);
        cells.set(offset_x + 6, offset_y + 2, true);
    }
    cells
}

fn create_initial_pattern(world: &World, index: &mut ChunkIndex) {
    // Create multiple interesting patterns across several regions

    // Region (0,0): R-pentomino in center - creates long-lived chaos
    add_chunk(world, index, ChunkPos::new(1, 1), r_pentomino_cells(5, 5));

    // Region (1,0): Some gliders
    add_chunk(world, index, ChunkPos::new(4, 0), glider_cells(2, 2));
    add_chunk(world, index, ChunkPos::new(5, 1), glider_cells(8, 8));
    add_chunk(world, index, ChunkPos::new(6, 2), glider_cells(4, 4));

    // Region (0,1): Lightweight spaceship
    add_chunk(world, index, ChunkPos::new(0, 4), gosper_gun_cells());

    // Region (-1,-1): Acorn pattern
    add_chunk(world, index, ChunkPos::new(-2, -2), acorn_cells(3, 3));

    // Region (1,1): More gliders going different directions
    add_chunk(world, index, ChunkPos::new(5, 5), glider_cells(1, 1));
    add_chunk(world, index, ChunkPos::new(6, 6), glider_cells(10, 10));

    // Add some random-ish patterns
    let mut random_cells = CellData::default();
    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            if (x + y) % 7 == 0 || (x * y) % 11 == 3 {
                random_cells.set(x, y, true);
            }
        }
    }
    add_chunk(world, index, ChunkPos::new(-5, 3), random_cells.clone());

    // Create a simple oscillator (blinker)
    let mut blinker = CellData::default();
    blinker.set(7, 7, true);
    blinker.set(8, 7, true);
    blinker.set(9, 7, true);
    add_chunk(world, index, ChunkPos::new(3, -3), blinker);

    // Create a block (stable pattern)
    let mut block = CellData::default();
    block.set(5, 5, true);
    block.set(6, 5, true);
    block.set(5, 6, true);
    block.set(6, 6, true);
    add_chunk(world, index, ChunkPos::new(-4, -4), block);

    // Beehive (stable)
    let mut beehive = CellData::default();
    beehive.set(6, 5, true);
    beehive.set(7, 5, true);
    beehive.set(5, 6, true);
    beehive.set(8, 6, true);
    beehive.set(6, 7, true);
    beehive.set(7, 7, true);
    add_chunk(world, index, ChunkPos::new(8, 8), beehive);
}

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    game: GameState,
    last_frame: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            game: GameState::new(),
            last_frame: Instant::now(),
        }
    }

    fn sync_and_render(&mut self) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        // Update camera
        renderer.update_camera(self.game.camera.position, self.game.camera.zoom);

        // Sync dirty chunks to atlas and collect instances
        let mut instances = Vec::new();
        let mut dirty_entities = Vec::new();

        self.game
            .world
            .each_entity::<(&ChunkPos, &CellData)>(|entity, (pos, cells)| {
                // Allocate or get atlas slot
                let slot = renderer.atlas.allocate(*pos);

                // If dirty, upload texture
                if entity.has(Dirty) {
                    renderer.upload_chunk_texture(slot, &cells.cells);
                    dirty_entities.push(entity.id());
                }

                // Get region color for this chunk
                let region = pos.containing_region();
                let color = Color::from_region(region);

                // Add instance for rendering
                instances.push(ChunkInstance {
                    world_pos: chunk_world_pos(*pos),
                    atlas_uv: atlas_uv(slot),
                    region_color: region_color_rgb(color),
                    chunk_in_region: chunk_in_region(*pos),
                    _padding: 0.0,
                });
            });

        // Remove dirty tags outside of iteration
        for entity_id in dirty_entities {
            self.game.world.entity_from_id(entity_id).remove(Dirty);
        }

        renderer.update_instances(&instances);

        // Render
        match renderer.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => {
                let size = self.window.as_ref().unwrap().inner_size();
                renderer.resize(size.width, size.height);
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                log::error!("Out of memory");
            }
            Err(e) => log::warn!("Render error: {:?}", e),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes().with_title("RGB Game of Life - Flecs + wgpu"),
                    )
                    .unwrap(),
            );

            self.renderer = Some(pollster::block_on(Renderer::new(window.clone())));
            self.window = Some(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => {
                self.game.handle_key(key, state == ElementState::Pressed);
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now.duration_since(self.last_frame).as_secs_f32();
                self.last_frame = now;

                self.game.update(dt);
                self.sync_and_render();

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
