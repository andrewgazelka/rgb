//! ECS Systems - all game logic
//!
//! Unlike Flecs, we don't have a scheduler. Instead, we manually call
//! each system function in order during each tick.

mod config;
mod handshake;
mod login;
mod network;
mod play;
mod time;

use rgb_ecs::World;

/// Run all systems for one tick
pub fn tick(world: &mut World, delta_time: f32) {
    // Phase 1: Network ingress (receive packets from network thread)
    network::system_network_ingress(world);
    network::system_handle_disconnects(world);

    // Phase 2: Protocol handling (process packets by connection state)
    handshake::system_handle_handshake(world);
    handshake::system_handle_status(world);
    login::system_handle_login(world);
    config::system_handle_configuration(world);

    // Phase 3: Play state systems
    play::system_send_spawn_data(world);
    play::system_handle_movement(world);
    play::system_send_keepalive(world);
    play::system_send_position_action_bar(world);

    // Phase 4: Time systems
    time::system_tick_world_time(world);
    time::system_update_tps(world, delta_time);

    // Phase 5: Network egress (send packets to network thread)
    network::system_network_egress(world);
}
