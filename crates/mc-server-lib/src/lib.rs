pub mod components;
pub mod modules;
pub mod packet_dispatch;
mod packets;
mod world_gen;

pub use components::*;
pub use flecs_ecs::prelude::*;
pub use modules::*;
pub use packet_dispatch::*;
pub use packets::*;

use crossbeam_channel::{Receiver, Sender};

/// Configuration for the Minecraft server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Port for the Flecs REST API explorer (default: 27750)
    pub rest_port: u16,
    /// Enable stats collection for the explorer
    pub enable_stats: bool,
    /// Target frames per second for the game loop
    pub target_fps: f32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            rest_port: 27750,
            enable_stats: true,
            target_fps: 20.0, // Minecraft runs at 20 TPS
        }
    }
}

/// Channels for network I/O between async Tokio runtime and sync Flecs world
pub struct NetworkChannels {
    /// Sender for incoming packets (async -> ECS)
    pub ingress_tx: Sender<IncomingPacket>,
    /// Receiver for incoming packets (async -> ECS)
    pub ingress_rx: Receiver<IncomingPacket>,
    /// Sender for outgoing packets (ECS -> async)
    pub egress_tx: Sender<OutgoingPacket>,
    /// Receiver for outgoing packets (ECS -> async)
    pub egress_rx: Receiver<OutgoingPacket>,
    /// Sender for disconnect events (async -> ECS)
    pub disconnect_tx: Sender<DisconnectEvent>,
    /// Receiver for disconnect events (async -> ECS)
    pub disconnect_rx: Receiver<DisconnectEvent>,
}

impl NetworkChannels {
    /// Create a new pair of network channels
    #[must_use]
    pub fn new() -> Self {
        let (ingress_tx, ingress_rx) = crossbeam_channel::unbounded();
        let (egress_tx, egress_rx) = crossbeam_channel::unbounded();
        let (disconnect_tx, disconnect_rx) = crossbeam_channel::unbounded();
        Self {
            ingress_tx,
            ingress_rx,
            egress_tx,
            egress_rx,
            disconnect_tx,
            disconnect_rx,
        }
    }
}

impl Default for NetworkChannels {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the Flecs world with all server modules
#[must_use]
pub fn create_world(channels: &NetworkChannels) -> World {
    let world = World::new();

    // IMPORTANT: Register singleton traits BEFORE importing modules that create systems.
    // Flecs requires traits to be set before any systems query the components.
    world
        .component::<WorldTime>()
        .add_trait::<flecs::Singleton>();
    world
        .component::<EntityIdCounter>()
        .add_trait::<flecs::Singleton>();
    world
        .component::<ChunkIndex>()
        .add_trait::<flecs::Singleton>();
    world
        .component::<ConnectionIdCounter>()
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
    world
        .component::<ConnectionIndex>()
        .add_trait::<flecs::Singleton>();
    world
        .component::<TpsTracker>()
        .add_trait::<flecs::Singleton>();

    // Set singleton values
    world.set(NetworkIngress {
        rx: channels.ingress_rx.clone(),
    });
    world.set(NetworkEgress {
        tx: channels.egress_tx.clone(),
    });
    world.set(DisconnectIngress {
        rx: channels.disconnect_rx.clone(),
    });
    world.set(WorldTime::default());
    world.set(EntityIdCounter::default());
    world.set(ChunkIndex::default());
    world.set(ConnectionIdCounter::default());
    world.set(ConnectionIndex::default());
    world.set(TpsTracker::default());

    // Import all modules (systems get created here, after singletons are registered)
    world.import::<NetworkModule>();
    world.import::<PacketDispatchModule>();
    world.import::<HandshakeModule>();
    world.import::<LoginModule>();
    world.import::<ConfigurationModule>();
    world.import::<ChunkModule>();
    world.import::<PlayModule>();
    world.import::<TimeModule>();

    // Generate spawn chunks
    generate_spawn_chunks(&world, 8);

    world
}

/// Run the Flecs world with REST API and stats enabled
///
/// This function blocks and runs the game loop.
pub fn run_with_explorer(world: &World, config: &ServerConfig) -> i32 {
    world
        .app()
        .enable_stats(config.enable_stats)
        .enable_rest(config.rest_port)
        .set_target_fps(config.target_fps)
        .run()
}

/// Run the Flecs world without REST API (for production)
///
/// This function blocks and runs the game loop.
pub fn run(world: &World, config: &ServerConfig) -> i32 {
    world.app().set_target_fps(config.target_fps).run()
}

/// Manually progress the world by one tick (for custom game loops)
pub fn tick(world: &World) -> bool {
    world.progress()
}
