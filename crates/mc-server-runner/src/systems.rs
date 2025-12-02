//! ECS Systems - all game logic
//!
//! Uses Flecs ECS with pipeline phases.

mod attack;
mod command;
mod config;
mod handshake;
pub mod history;
#[cfg(feature = "dashboard")]
mod introspect;
mod login;
mod network;
mod play;
mod time;

pub use command::send_commands_to_player;

use flecs_ecs::prelude::*;

use crate::components::*;

/// Server module - registers all systems
#[derive(Component)]
pub struct ServerModule;

impl Module for ServerModule {
    fn module(world: &World) {
        world.module::<ServerModule>("server");

        // Register all component types
        world.component::<NetworkIngress>();
        world.component::<NetworkEgress>();
        world.component::<DisconnectIngress>();
        world.component::<Connection>();
        world.component::<ConnectionId>();
        world.component::<ProtocolState>();
        world.component::<PacketBuffer>();
        world.component::<PendingPackets>();
        world.component::<ConnectionIndex>();
        world.component::<Player>();
        world.component::<Name>();
        world.component::<Uuid>();
        world.component::<EntityId>();
        world.component::<Position>();
        world.component::<Rotation>();
        world.component::<ChunkPosition>();
        world.component::<GameMode>();
        world.component::<NeedsSpawnChunks>();
        world.component::<InPlayState>();
        world.component::<EntityIdCounter>();
        world.component::<ChunkPos>();
        world.component::<ChunkData>();
        world.component::<ChunkLoaded>();
        world.component::<ServerConfig>();
        world.component::<WorldTime>();
        world.component::<TpsTracker>();
        world.component::<DeltaTime>();

        // Mark singletons
        world
            .component::<ServerConfig>()
            .add_trait::<flecs::Singleton>();
        world
            .component::<WorldTime>()
            .add_trait::<flecs::Singleton>();
        world
            .component::<TpsTracker>()
            .add_trait::<flecs::Singleton>();
        world
            .component::<DeltaTime>()
            .add_trait::<flecs::Singleton>();
        world
            .component::<EntityIdCounter>()
            .add_trait::<flecs::Singleton>();
        world
            .component::<PendingPackets>()
            .add_trait::<flecs::Singleton>();
        world
            .component::<ConnectionIndex>()
            .add_trait::<flecs::Singleton>();
        world
            .component::<NetworkIngress>()
            .add_trait::<flecs::Singleton>();
        world
            .component::<NetworkEgress>()
            .add_trait::<flecs::Singleton>();
        world
            .component::<DisconnectIngress>()
            .add_trait::<flecs::Singleton>();

        // ============================================================
        // NETWORK INGRESS - OnLoad phase (first)
        // ============================================================
        world
            .system_named::<(&NetworkIngress, &mut PendingPackets, &mut ConnectionIndex)>(
                "NetworkIngress",
            )
            .kind::<flecs::pipeline::OnLoad>()
            .run(|mut it| {
                while it.next() {
                    network::system_network_ingress(&it);
                }
            });

        world
            .system_named::<(&DisconnectIngress, &mut ConnectionIndex)>("HandleDisconnects")
            .kind::<flecs::pipeline::OnLoad>()
            .run(|mut it| {
                while it.next() {
                    network::system_handle_disconnects(&it);
                }
            });

        // ============================================================
        // PROTOCOL HANDLING - PreUpdate phase
        // ============================================================
        world
            .system_named::<(&mut PacketBuffer, &mut ProtocolState)>("HandleHandshake")
            .kind::<flecs::pipeline::PreUpdate>()
            .with::<Connection>()
            .each_entity(|entity, (buffer, state)| {
                handshake::handle_handshake(entity, buffer, state);
            });

        world
            .system_named::<(&mut PacketBuffer, &ProtocolState, &ServerConfig)>("HandleStatus")
            .kind::<flecs::pipeline::PreUpdate>()
            .with::<Connection>()
            .run(|mut it| {
                while it.next() {
                    handshake::system_handle_status(&it);
                }
            });

        world
            .system_named::<(&mut PacketBuffer, &mut ProtocolState, &EntityIdCounter)>(
                "HandleLogin",
            )
            .kind::<flecs::pipeline::PreUpdate>()
            .with::<Connection>()
            .run(|mut it| {
                while it.next() {
                    login::system_handle_login(&it);
                }
            });

        world
            .system_named::<(&mut PacketBuffer, &mut ProtocolState)>("HandleConfiguration")
            .kind::<flecs::pipeline::PreUpdate>()
            .with::<Connection>()
            .run(|mut it| {
                while it.next() {
                    config::system_handle_configuration(&it);
                }
            });

        // ============================================================
        // PLAY STATE - OnUpdate phase
        // ============================================================
        world
            .system_named::<(
                &mut PacketBuffer,
                &Position,
                &EntityId,
                &ServerConfig,
                &WorldTime,
            )>("SendSpawnData")
            .kind::<flecs::pipeline::OnUpdate>()
            .with::<NeedsSpawnChunks>()
            .run(|mut it| {
                while it.next() {
                    play::system_send_spawn_data(&it);
                }
            });

        world
            .system_named::<(&mut PacketBuffer, &mut Position, &mut Rotation)>("HandleMovement")
            .kind::<flecs::pipeline::OnUpdate>()
            .with::<InPlayState>()
            .each_entity(|_entity, (buffer, pos, rot)| {
                play::handle_movement(buffer, pos, rot);
            });

        world
            .system_named::<(&mut PacketBuffer, &WorldTime)>("SendKeepalive")
            .kind::<flecs::pipeline::OnUpdate>()
            .with::<InPlayState>()
            .run(|mut it| {
                while it.next() {
                    play::system_send_keepalive(&it);
                }
            });

        world
            .system_named::<(&mut PacketBuffer, &Position, &TpsTracker, &WorldTime)>(
                "SendPositionActionBar",
            )
            .kind::<flecs::pipeline::OnUpdate>()
            .with::<InPlayState>()
            .run(|mut it| {
                while it.next() {
                    play::system_send_position_action_bar(&it);
                }
            });

        world
            .system_named::<(&mut PacketBuffer, &InPlayState)>("HandleAttacks")
            .kind::<flecs::pipeline::OnUpdate>()
            .run(|mut it| {
                while it.next() {
                    attack::system_handle_attacks(&it);
                }
            });

        world
            .system_named::<(&mut PacketBuffer, &InPlayState)>("HandleCommands")
            .kind::<flecs::pipeline::OnUpdate>()
            .run(|mut it| {
                while it.next() {
                    command::system_handle_commands(&it);
                }
            });

        // ============================================================
        // TIME - PostUpdate phase
        // ============================================================
        world
            .system_named::<&mut WorldTime>("TickWorldTime")
            .kind::<flecs::pipeline::PostUpdate>()
            .each(|time| {
                time.tick();
            });

        world
            .system_named::<(&mut TpsTracker, &DeltaTime)>("UpdateTps")
            .kind::<flecs::pipeline::PostUpdate>()
            .each(|(tps, delta)| {
                tps.update(delta.0);
            });

        // ============================================================
        // NETWORK EGRESS - OnStore phase (last)
        // ============================================================
        world
            .system_named::<(&mut PacketBuffer, &ConnectionId, &NetworkEgress)>("NetworkEgress")
            .kind::<flecs::pipeline::OnStore>()
            .with::<Connection>()
            .each(|(buffer, conn_id, egress)| {
                network::handle_egress(buffer, conn_id, egress);
            });
    }
}
