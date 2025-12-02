//! ECS Systems - all game logic
//!
//! Uses RGB phase execution model where each color phase executes
//! independently. Audio markers help profile lag per phase.

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

use std::time::Instant;

use rgb_ecs::World;
use rgb_spatial::Color;

// use crate::audio;

/// Run all systems for one tick with RGB phase profiling.
pub fn tick(world: &mut World, delta_time: f32) {
    // Global phase: Network ingress (must happen before RGB phases)
    network::system_network_ingress(world);
    network::system_handle_disconnects(world);

    // Process dashboard introspection requests
    #[cfg(feature = "dashboard")]
    introspect::system_process_introspect(world);

    // RGB Phase execution with audio profiling
    for color in Color::ALL {
        let _phase_start = Instant::now();
        // audio::tick_start(color);

        run_phase(world, color, delta_time);

        let _phase_duration = _phase_start.elapsed();
        // audio::beep_color(color, phase_duration);
    }

    // Global phase: Network egress (must happen after RGB phases)
    network::system_network_egress(world);
}

/// Run systems for a specific RGB phase.
///
/// In a full implementation, each color phase would only process
/// entities in cells of that color, enabling parallel execution.
/// For now, we run all systems but mark the phase for profiling.
fn run_phase(world: &mut World, color: Color, delta_time: f32) {
    // TODO: Filter entities by cell color for true parallel execution
    // For now, distribute systems across phases for demonstration

    match color {
        Color::Red => {
            // Protocol handling phase
            handshake::system_handle_handshake(world);
            handshake::system_handle_status(world);
            login::system_handle_login(world);
            config::system_handle_configuration(world);
        }
        Color::Green => {
            // Play state systems phase
            play::system_send_spawn_data(world);
            play::system_handle_movement(world);
            play::system_send_keepalive(world);
            play::system_send_position_action_bar(world);

            // Combat and command systems
            attack::system_handle_attacks(world);
            command::system_handle_commands(world);
        }
        Color::Blue => {
            // Time systems phase
            time::system_tick_world_time(world);
            time::system_update_tps(world, delta_time);
        }
    }
}
