//! Flecs component persistence system.
//!
//! This crate provides automatic persistence for Flecs components using LMDB (heed).
//!
//! # Usage
//!
//! 1. Register a component for persistence:
//! ```ignore
//! use persist::PersistExt;
//!
//! world.component::<Position>().persist::<Uuid>();
//! ```
//!
//! 2. Initialize the persistence system:
//! ```ignore
//! persist::init::<Uuid>(world, "data/persist");
//! ```
//!
//! 3. When `Uuid` is set on an entity, all persisted components are automatically loaded.
//! 4. When a persisted component is set on an entity with `Uuid`, it's automatically saved.

mod db;

use std::sync::Arc;

use flecs_ecs::prelude::*;

pub use db::PersistDb;

/// Tag component added to component entities to mark them as persistent.
#[derive(Component, Default)]
pub struct Persist;

/// Stores type-erased load/save functions on a component entity.
#[derive(Component, Clone)]
pub struct PersistLoader {
    /// Deserialize bytes and set component on entity.
    /// fn(bytes, entity)
    pub load: fn(&[u8], EntityView<'_>),
    /// Serialize component from entity, if present.
    /// fn(entity) -> Option<Vec<u8>>
    pub save: fn(EntityView<'_>) -> Option<Vec<u8>>,
}

/// Wrapper around `PersistDb` for use as a Flecs singleton.
#[derive(Component)]
pub struct PersistDbSingleton(pub Arc<PersistDb>);

/// Persistence module for Flecs.
#[derive(Component)]
pub struct PersistModule;

impl Module for PersistModule {
    fn module(world: &World) {
        world.module::<Self>("persist");

        world.component::<Persist>();
        world.component::<PersistLoader>();
        world
            .component::<PersistDbSingleton>()
            .add_trait::<flecs::Singleton>();
    }
}

/// Initialize the persistence system.
///
/// This:
/// 1. Opens the database at `db_path`
/// 2. Sets up an observer on `UuidComponent` to load persisted components when UUID is set
///
/// # Panics
/// Panics if the database cannot be opened.
pub fn init<UuidComponent>(world: &World, db_path: &str)
where
    UuidComponent: ComponentId + DataComponent + Copy + Into<u128>,
{
    world.import::<PersistModule>();

    let db = PersistDb::open(db_path).expect("Failed to open persist database");
    world.set(PersistDbSingleton(Arc::new(db)));

    // When Uuid is set on an entity, load all persisted components
    world
        .observer::<flecs::OnSet, &UuidComponent>()
        .each_entity(|entity, uuid| {
            let uuid_val: u128 = (*uuid).into();
            load_all_components(entity, uuid_val);
        });
}

/// Load all persisted components for an entity.
fn load_all_components(entity: EntityView<'_>, uuid: u128) {
    let world = entity.world();

    // Get the database
    let db = world.get::<&PersistDbSingleton>(|db| Arc::clone(&db.0));

    // Query all component entities that have Persist + PersistLoader
    world
        .query::<&PersistLoader>()
        .with(Persist::id())
        .with(flecs::Component::id())
        .build()
        .each_entity(|component_entity, loader| {
            let component_name = component_entity.name();

            match db.load_bytes(uuid, &component_name) {
                Ok(Some(bytes)) => {
                    (loader.load)(&bytes, entity);
                    tracing::debug!("Loaded {component_name} for entity {uuid:032x}");
                }
                Ok(None) => {
                    tracing::trace!("No persisted {component_name} for entity {uuid:032x}");
                }
                Err(e) => {
                    tracing::error!("Failed to load {component_name}: {e}");
                }
            }
        });
}

/// Extension trait for registering persistent components.
pub trait PersistExt<T: ComponentId> {
    /// Mark this component as persistent.
    ///
    /// This:
    /// 1. Adds the `Persist` tag to the component entity
    /// 2. Registers load/save functions via `PersistLoader`
    /// 3. Sets up an `OnSet` observer to save when the component changes
    ///
    /// The component will only be persisted if the entity also has a `UuidComponent`.
    fn persist<UuidComponent>(self) -> Self
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
        UuidComponent: ComponentId + DataComponent + Copy + Into<u128>;
}

impl<'a, T: ComponentId + DataComponent> PersistExt<T> for Component<'a, T> {
    fn persist<UuidComponent>(self) -> Self
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
        UuidComponent: ComponentId + DataComponent + Copy + Into<u128>,
    {
        let world = self.world();
        let component_name = self.name();

        tracing::info!("Registered persistent component: {component_name}");

        // Add Persist tag and PersistLoader to the component entity
        self.entity().add(Persist).set(PersistLoader {
            load: |bytes, entity| match bincode::deserialize::<T>(bytes) {
                Ok(component) => {
                    entity.set(component);
                }
                Err(e) => {
                    tracing::error!("Failed to deserialize component: {e}");
                }
            },
            save: |entity| {
                entity
                    .try_get::<&T>(|c| bincode::serialize(c).ok())
                    .flatten()
            },
        });

        // Create OnSet observer - fires when T is set on an entity that has UuidComponent
        world
            .observer::<flecs::OnSet, (&T, &UuidComponent)>()
            .each_entity(move |entity, (component, uuid)| {
                let uuid_val: u128 = (*uuid).into();

                let Ok(bytes) = bincode::serialize(component) else {
                    tracing::error!("Failed to serialize {component_name}");
                    return;
                };

                entity.world().get::<&PersistDbSingleton>(|db| {
                    if let Err(e) = db.0.save_bytes(uuid_val, &component_name, &bytes) {
                        tracing::error!("Failed to persist {component_name}: {e}");
                    }
                });
            });

        self
    }
}

/// Load a specific persisted component for an entity.
///
/// Returns `true` if data was loaded, `false` otherwise.
pub fn load_component<T>(world: &World, entity: EntityView<'_>, uuid: u128) -> bool
where
    T: ComponentId + serde::de::DeserializeOwned,
{
    let component_name = world.component::<T>().name();

    world.get::<&PersistDbSingleton>(|db| match db.0.load_bytes(uuid, &component_name) {
        Ok(Some(bytes)) => match bincode::deserialize::<T>(&bytes) {
            Ok(component) => {
                entity.set(component);
                tracing::debug!("Loaded {component_name} for entity");
                true
            }
            Err(e) => {
                tracing::error!("Failed to deserialize {component_name}: {e}");
                false
            }
        },
        Ok(None) => {
            tracing::trace!("No persisted {component_name} for entity");
            false
        }
        Err(e) => {
            tracing::error!("Failed to load {component_name}: {e}");
            false
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// Test UUID component
    #[derive(Component, Debug, Clone, Copy)]
    struct TestUuid(u128);

    impl From<TestUuid> for u128 {
        fn from(uuid: TestUuid) -> Self {
            uuid.0
        }
    }

    /// Test position component
    #[derive(Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
    struct TestPosition {
        x: f64,
        y: f64,
        z: f64,
    }

    /// Test health component
    #[derive(Component, Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
    struct TestHealth {
        value: i32,
    }

    #[test]
    fn test_persist_saves_on_component_set() {
        let dir = tempfile::tempdir().unwrap();
        let world = World::new();

        // Initialize persistence
        init::<TestUuid>(&world, dir.path().to_str().unwrap());

        // Register Position as persistent
        world.component::<TestPosition>().persist::<TestUuid>();

        // Create entity with UUID first, then set Position
        let uuid = 0x1234_5678_9abc_def0_u128;
        let entity = world.entity().set(TestUuid(uuid));

        // Set position - should trigger save
        entity.set(TestPosition {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        });

        // Verify it was saved to DB
        world.get::<&PersistDbSingleton>(|db| {
            let bytes = db.0.load_bytes(uuid, "TestPosition").unwrap();
            assert!(bytes.is_some(), "Position should be saved to DB");

            let loaded: TestPosition = bincode::deserialize(&bytes.unwrap()).unwrap();
            assert_eq!(loaded.x, 1.0);
            assert_eq!(loaded.y, 2.0);
            assert_eq!(loaded.z, 3.0);
        });
    }

    #[test]
    fn test_persist_loads_on_uuid_set() {
        let dir = tempfile::tempdir().unwrap();
        let uuid = 0xabcd_ef01_2345_6789_u128;

        // First: Create a world, save some data, then drop it
        {
            let world = World::new();
            init::<TestUuid>(&world, dir.path().to_str().unwrap());
            world.component::<TestPosition>().persist::<TestUuid>();

            let entity = world.entity().set(TestUuid(uuid)).set(TestPosition {
                x: 10.0,
                y: 20.0,
                z: 30.0,
            });

            // Verify entity has position
            entity.get::<&TestPosition>(|pos| {
                assert_eq!(pos.x, 10.0);
            });
        }

        // Second: Create a NEW world, register components, then set UUID
        // The position should be automatically loaded
        {
            let world = World::new();
            init::<TestUuid>(&world, dir.path().to_str().unwrap());
            world.component::<TestPosition>().persist::<TestUuid>();

            // Create entity and set UUID - should trigger load
            let entity = world.entity().set(TestUuid(uuid));

            // Position should have been loaded automatically
            entity.get::<&TestPosition>(|pos| {
                assert_eq!(pos.x, 10.0);
                assert_eq!(pos.y, 20.0);
                assert_eq!(pos.z, 30.0);
            });
        }
    }

    #[test]
    fn test_persist_multiple_components() {
        let dir = tempfile::tempdir().unwrap();
        let uuid = 0xffff_0000_aaaa_5555_u128;

        // Save multiple components
        {
            let world = World::new();
            init::<TestUuid>(&world, dir.path().to_str().unwrap());
            world.component::<TestPosition>().persist::<TestUuid>();
            world.component::<TestHealth>().persist::<TestUuid>();

            world
                .entity()
                .set(TestUuid(uuid))
                .set(TestPosition {
                    x: 5.0,
                    y: 6.0,
                    z: 7.0,
                })
                .set(TestHealth { value: 100 });
        }

        // Load in new world
        {
            let world = World::new();
            init::<TestUuid>(&world, dir.path().to_str().unwrap());
            world.component::<TestPosition>().persist::<TestUuid>();
            world.component::<TestHealth>().persist::<TestUuid>();

            let entity = world.entity().set(TestUuid(uuid));

            entity.get::<&TestPosition>(|pos| {
                assert_eq!(pos.x, 5.0);
                assert_eq!(pos.y, 6.0);
                assert_eq!(pos.z, 7.0);
            });

            entity.get::<&TestHealth>(|health| {
                assert_eq!(health.value, 100);
            });
        }
    }

    #[test]
    fn test_persist_updates_on_change() {
        let dir = tempfile::tempdir().unwrap();
        let world = World::new();
        let uuid = 0x1111_2222_3333_4444_u128;

        init::<TestUuid>(&world, dir.path().to_str().unwrap());
        world.component::<TestPosition>().persist::<TestUuid>();

        let entity = world.entity().set(TestUuid(uuid)).set(TestPosition {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        });

        // Update position
        entity.set(TestPosition {
            x: 99.0,
            y: 99.0,
            z: 99.0,
        });

        // Verify updated value is in DB
        world.get::<&PersistDbSingleton>(|db| {
            let bytes = db.0.load_bytes(uuid, "TestPosition").unwrap().unwrap();
            let loaded: TestPosition = bincode::deserialize(&bytes).unwrap();
            assert_eq!(loaded.x, 99.0);
            assert_eq!(loaded.y, 99.0);
            assert_eq!(loaded.z, 99.0);
        });
    }

    #[test]
    fn test_no_persist_without_uuid() {
        let dir = tempfile::tempdir().unwrap();
        let world = World::new();

        init::<TestUuid>(&world, dir.path().to_str().unwrap());
        world.component::<TestPosition>().persist::<TestUuid>();

        // Create entity WITHOUT UUID, set position
        let _entity = world.entity().set(TestPosition {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        });

        // Nothing should be saved (no UUID on entity)
        world.get::<&PersistDbSingleton>(|_db| {
            // We can't easily check "nothing was saved" without knowing what UUID would be used
            // But the observer only fires when both T and UuidComponent are present
            // This test just ensures no panic occurs
        });
    }
}
