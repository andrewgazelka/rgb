use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureFormat, TextureDimension, TextureUsages};
use bevy::sprite::Anchor;

use rgb_core::{ChunkPos, World, CHUNK_SIZE};
use rgb_life::{LifeChunk, LifeSimulation};

const CELL_SIZE: f32 = 4.0;
const CHUNK_PIXEL_SIZE: f32 = CHUNK_SIZE as f32 * CELL_SIZE;

#[derive(Resource)]
struct GameWorld(World<LifeSimulation>);

#[derive(Resource)]
struct SimulationTimer(Timer);

#[derive(Resource)]
struct Paused(bool);

#[derive(Component)]
struct ChunkSprite(ChunkPos);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RGB Game of Life".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .insert_resource(GameWorld(create_initial_world()))
        .insert_resource(SimulationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .insert_resource(Paused(false))
        .add_systems(Startup, setup)
        .add_systems(Update, (simulation_step, sync_chunk_sprites, handle_input, camera_movement))
        .run();
}

fn create_initial_world() -> World<LifeSimulation> {
    let mut world = World::new(LifeSimulation);

    {
        let chunk = world.get_chunk_mut(ChunkPos::new(0, 0));
        chunk.set(5, 5, true);
        chunk.set(6, 5, true);
        chunk.set(7, 5, true);
        chunk.set(7, 4, true);
        chunk.set(6, 3, true);
    }

    {
        let chunk = world.get_chunk_mut(ChunkPos::new(0, 0));
        for i in 0..10 {
            chunk.set(2, i, true);
        }
    }

    {
        let chunk = world.get_chunk_mut(ChunkPos::new(1, 0));
        chunk.set(0, 0, true);
        chunk.set(1, 0, true);
        chunk.set(2, 0, true);
        chunk.set(2, 1, true);
        chunk.set(1, 2, true);
    }

    world
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn simulation_step(
    time: Res<Time>,
    mut timer: ResMut<SimulationTimer>,
    mut world: ResMut<GameWorld>,
    paused: Res<Paused>,
) {
    if paused.0 {
        return;
    }

    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        world.0.step();
    }
}

fn sync_chunk_sprites(
    mut commands: Commands,
    world: Res<GameWorld>,
    mut images: ResMut<Assets<Image>>,
    mut chunk_sprites: Query<(Entity, &ChunkSprite, &mut Sprite)>,
) {
    let mut existing: std::collections::HashSet<ChunkPos> = std::collections::HashSet::new();

    for (entity, chunk_sprite, mut sprite) in &mut chunk_sprites {
        let pos = chunk_sprite.0;

        if let Some(chunk) = world.0.get_chunk(pos) {
            existing.insert(pos);
            sprite.image = images.add(create_chunk_image(chunk));
        } else {
            commands.entity(entity).despawn();
        }
    }

    for (pos, chunk) in world.0.chunks() {
        if existing.contains(&pos) {
            continue;
        }

        let image = create_chunk_image(chunk);
        let handle = images.add(image);

        commands.spawn((
            Sprite {
                image: handle,
                anchor: Anchor::BottomLeft,
                custom_size: Some(Vec2::splat(CHUNK_PIXEL_SIZE)),
                ..default()
            },
            Transform::from_xyz(
                pos.x as f32 * CHUNK_PIXEL_SIZE,
                pos.y as f32 * CHUNK_PIXEL_SIZE,
                0.0,
            ),
            ChunkSprite(pos),
        ));
    }
}

fn create_chunk_image(chunk: &LifeChunk) -> Image {
    let size = CHUNK_SIZE;
    let mut data = vec![0u8; size * size * 4];

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            if chunk.get(x, y) {
                data[idx] = 255;
                data[idx + 1] = 255;
                data[idx + 2] = 255;
                data[idx + 3] = 255;
            } else {
                data[idx] = 20;
                data[idx + 1] = 20;
                data[idx + 2] = 20;
                data[idx + 3] = 255;
            }
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );
    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;

    image
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut paused: ResMut<Paused>,
    mut world: ResMut<GameWorld>,
) {
    if keys.just_pressed(KeyCode::Space) {
        paused.0 = !paused.0;
    }

    if keys.just_pressed(KeyCode::KeyS) && paused.0 {
        world.0.step();
    }

    if keys.just_pressed(KeyCode::KeyR) {
        world.0 = create_initial_world();
    }
}

fn camera_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2d>>,
) {
    let Some((mut transform, mut projection)) = camera.iter_mut().next() else {
        return;
    };

    let speed = 200.0 * projection.scale;
    let delta = time.delta_secs();

    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        transform.translation.y += speed * delta;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        transform.translation.x -= speed * delta;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        transform.translation.x += speed * delta;
    }
    if keys.pressed(KeyCode::KeyZ) || keys.pressed(KeyCode::ArrowDown) {
        transform.translation.y -= speed * delta;
    }

    if keys.pressed(KeyCode::KeyQ) {
        projection.scale *= 1.0 + delta;
    }
    if keys.pressed(KeyCode::KeyE) {
        projection.scale *= 1.0 - delta;
        projection.scale = projection.scale.max(0.1);
    }
}
