//! ECS Systems - all game logic
//!
//! Uses Flecs ECS with pipeline phases.

mod attack;
mod command;
mod config;
#[cfg(feature = "dashboard")]
pub mod dashboard;
mod handshake;
pub mod history;
mod login;
mod network;
mod play;
mod time;

pub use command::send_commands_to_player;

use flecs_ecs::prelude::*;

use crate::components::*;

/// Initialize all systems for the server
pub fn init_systems(world: &World) {
    // ============================================================
    // NETWORK INGRESS - OnLoad phase (first)
    // ============================================================
    world
        .system::<()>()
        .kind(id::<flecs::pipeline::OnLoad>())
        .each_iter(|it, _i, _| {
            network::system_network_ingress(&it.world());
        });

    world
        .system::<()>()
        .kind(id::<flecs::pipeline::OnLoad>())
        .each_iter(|it, _i, _| {
            network::system_handle_disconnects(&it.world());
        });

    // ============================================================
    // PROTOCOL HANDLING - PreUpdate phase
    // ============================================================
    world
        .system::<(&mut PacketBuffer, &mut ProtocolState)>()
        .with(Connection)
        .kind(id::<flecs::pipeline::PreUpdate>())
        .each_entity(|entity, (buffer, state)| {
            handshake::handle_handshake(entity, buffer, state);
        });

    world
        .system::<(&mut PacketBuffer, &ProtocolState)>()
        .with(Connection)
        .kind(id::<flecs::pipeline::PreUpdate>())
        .each_iter(|it, _i, (buffer, state)| {
            let world = it.world();
            let config = world.get::<&ServerConfig>(|c| c.clone());
            handshake::handle_status(buffer, state, &config);
        });

    world
        .system::<(&mut PacketBuffer, &mut ProtocolState)>()
        .with(Connection)
        .kind(id::<flecs::pipeline::PreUpdate>())
        .each_iter(|it, i, (buffer, state)| {
            let world = it.world();
            let entity_counter = world.get::<&EntityIdCounter>(|c| EntityIdCounter(c.0.clone()));
            let entity = it.entity(i);
            login::handle_login(entity, buffer, state, &entity_counter);
        });

    world
        .system::<(&mut PacketBuffer, &mut ProtocolState)>()
        .with(Connection)
        .kind(id::<flecs::pipeline::PreUpdate>())
        .each_entity(|entity, (buffer, state)| {
            config::handle_configuration(entity, buffer, state);
        });

    // ============================================================
    // PLAY STATE - OnUpdate phase
    // ============================================================
    world
        .system::<(&mut PacketBuffer, &Position, &EntityId)>()
        .with(NeedsSpawnChunks)
        .kind(id::<flecs::pipeline::OnUpdate>())
        .each_iter(|it, i, (buffer, pos, entity_id)| {
            let world = it.world();
            let entity = it.entity(i);
            play::send_spawn_data(&world, entity, buffer, pos, entity_id);
        });

    world
        .system::<(&mut PacketBuffer, &mut Position, &mut Rotation)>()
        .with(InPlayState)
        .kind(id::<flecs::pipeline::OnUpdate>())
        .each(|(buffer, pos, rot)| {
            play::handle_movement(buffer, pos, rot);
        });

    world
        .system::<&mut PacketBuffer>()
        .with(InPlayState)
        .kind(id::<flecs::pipeline::OnUpdate>())
        .each_iter(|it, _i, buffer| {
            let world = it.world();
            let world_time = world.get::<&WorldTime>(|t| *t);
            play::send_keepalive(buffer, &world_time);
        });

    world
        .system::<(&mut PacketBuffer, &Position)>()
        .with(InPlayState)
        .kind(id::<flecs::pipeline::OnUpdate>())
        .each_iter(|it, _i, (buffer, pos)| {
            let world = it.world();
            let world_time = world.get::<&WorldTime>(|t| *t);
            let tps = world.get::<&TpsTracker>(|t| *t);
            play::send_position_action_bar(buffer, pos, &world_time, &tps);
        });

    world
        .system::<&mut PacketBuffer>()
        .with(InPlayState)
        .kind(id::<flecs::pipeline::OnUpdate>())
        .each_iter(|it, i, buffer| {
            let world = it.world();
            let entity = it.entity(i);
            attack::handle_attacks(&world, entity, buffer);
        });

    world
        .system::<&mut PacketBuffer>()
        .with(InPlayState)
        .kind(id::<flecs::pipeline::OnUpdate>())
        .each_iter(|it, i, buffer| {
            let world = it.world();
            let entity = it.entity(i);
            command::handle_commands(&world, entity, buffer);
        });

    // ============================================================
    // TIME - PostUpdate phase
    // ============================================================
    world
        .system::<&mut WorldTime>()
        .kind(id::<flecs::pipeline::PostUpdate>())
        .each(|time| {
            time.tick();
        });

    world
        .system::<&mut TpsTracker>()
        .kind(id::<flecs::pipeline::PostUpdate>())
        .each_iter(|it, _i, tps| {
            let world = it.world();
            let delta = world.get::<&DeltaTime>(|d| d.0);
            tps.update(delta);
        });

    // ============================================================
    // NETWORK EGRESS - OnStore phase (last)
    // ============================================================
    world
        .system::<(&mut PacketBuffer, &ConnectionId)>()
        .with(Connection)
        .kind(id::<flecs::pipeline::OnStore>())
        .each_iter(|it, _i, (buffer, conn_id)| {
            let world = it.world();
            world.get::<&NetworkEgress>(|egress| {
                network::handle_egress(buffer, conn_id, egress);
            });
        });
}
