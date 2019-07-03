// Reexports
pub use sphynx::Uid;

use crate::{
    comp,
    msg::{EcsCompPacket, EcsResPacket},
    sys,
    terrain::{Block, TerrainChunk, TerrainMap},
    vol::WriteVol,
};
use rayon::{ThreadPool, ThreadPoolBuilder};
use serde_derive::{Deserialize, Serialize};
use specs::{
    shred::{Fetch, FetchMut},
    storage::{MaskedStorage as EcsMaskedStorage, Storage as EcsStorage},
    Component, DispatcherBuilder, Entity as EcsEntity,
};
use sphynx;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};
use vek::*;

/// How much faster should an in-game day be compared to a real day?
// TODO: Don't hard-code this.
const DAY_CYCLE_FACTOR: f64 = 24.0 * 2.0;

/// A resource that stores the time of day.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeOfDay(pub f64);

/// A resource that stores the tick (i.e: physics) time.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Time(pub f64);

/// A resource that stores the time since the previous tick.
#[derive(Default)]
pub struct DeltaTime(pub f32);

/// At what point should we stop speeding up physics to compensate for lag? If we speed physics up
/// too fast, we'd skip important physics events like collisions. This constant determines the
/// upper limit. If delta time exceeds this value, the game's physics will begin to produce time
/// lag. Ideally, we'd avoid such a situation.
const MAX_DELTA_TIME: f32 = 1.0;

pub struct TerrainChange {
    pub blocks: HashMap<Vec3<i32>, Block>,
}

impl Default for TerrainChange {
    fn default() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }
}

impl TerrainChange {
    pub fn set(&mut self, pos: Vec3<i32>, block: Block) {
        self.blocks.insert(pos, block);
    }

    pub fn clear(&mut self) {
        self.blocks.clear();
    }
}

pub struct ChunkChanges {
    pub new_chunks: HashSet<Vec2<i32>>,
    pub modified_chunks: HashSet<Vec2<i32>>,
    pub removed_chunks: HashSet<Vec2<i32>>,
}

impl Default for ChunkChanges {
    fn default() -> Self {
        Self {
            new_chunks: HashSet::new(),
            modified_chunks: HashSet::new(),
            removed_chunks: HashSet::new(),
        }
    }
}
impl ChunkChanges {
    pub fn clear(&mut self) {
        self.new_chunks.clear();
        self.modified_chunks.clear();
        self.removed_chunks.clear();
    }
}

/// A type used to represent game state stored on both the client and the server. This includes
/// things like entity components, terrain data, and global states like weather, time of day, etc.
pub struct State {
    ecs: sphynx::World<EcsCompPacket, EcsResPacket>,
    // Avoid lifetime annotation by storing a thread pool instead of the whole dispatcher
    thread_pool: Arc<ThreadPool>,
}

impl Default for State {
    /// Create a new `State`.
    fn default() -> Self {
        Self {
            ecs: sphynx::World::new(specs::World::new(), Self::setup_sphynx_world),
            thread_pool: Arc::new(ThreadPoolBuilder::new().build().unwrap()),
        }
    }
}

impl State {
    /// Create a new `State` from an ECS state package.
    pub fn from_state_package(
        state_package: sphynx::StatePackage<EcsCompPacket, EcsResPacket>,
    ) -> Self {
        Self {
            ecs: sphynx::World::from_state_package(
                specs::World::new(),
                Self::setup_sphynx_world,
                state_package,
            ),
            thread_pool: Arc::new(ThreadPoolBuilder::new().build().unwrap()),
        }
    }

    // Create a new Sphynx ECS world.
    fn setup_sphynx_world(ecs: &mut sphynx::World<EcsCompPacket, EcsResPacket>) {
        // Register server->client synced components.
        ecs.register_synced::<comp::Body>();
        ecs.register_synced::<comp::Player>();
        ecs.register_synced::<comp::Stats>();

        // Register components synced by other means
        ecs.register::<comp::Pos>();
        ecs.register::<comp::Vel>();
        ecs.register::<comp::Ori>();
        ecs.register::<comp::MoveDir>();
        ecs.register::<comp::OnGround>();
        ecs.register::<comp::CanBuild>();
        ecs.register::<comp::Controller>();
        ecs.register::<comp::Attacking>();
        ecs.register::<comp::Wielding>();
        ecs.register::<comp::Rolling>();
        ecs.register::<comp::Gliding>();
        ecs.register::<comp::ActionState>();

        // Register client-local components
        ecs.register::<comp::AnimationInfo>();
        ecs.register::<comp::Jumping>();

        // Register server-local components
        ecs.register::<comp::Agent>();
        ecs.register::<comp::Respawning>();
        ecs.register::<comp::Dying>();
        ecs.register::<comp::ForceUpdate>();
        ecs.register::<comp::Inventory>();

        // Register synced resources used by the ECS.
        ecs.add_resource_synced(TimeOfDay(0.0));

        // Register unsynced resources used by the ECS.
        ecs.add_resource(Time(0.0));
        ecs.add_resource(DeltaTime(0.0));
        ecs.add_resource(TerrainMap::new().unwrap());
        ecs.add_resource(TerrainChange::default());
        ecs.add_resource(ChunkChanges::default());
    }

    /// Register a component with the state's ECS.
    pub fn with_component<T: Component>(mut self) -> Self
    where
        <T as Component>::Storage: Default,
    {
        self.ecs.register::<T>();
        self
    }

    /// Write a component attributed to a particular entity.
    pub fn write_component<C: Component>(&mut self, entity: EcsEntity, comp: C) {
        let _ = self.ecs.write_storage().insert(entity, comp);
    }

    /// Read a component attributed to a particular entity.
    pub fn read_component_cloned<C: Component + Clone>(&self, entity: EcsEntity) -> Option<C> {
        self.ecs.read_storage().get(entity).cloned()
    }

    /// Get a read-only reference to the storage of a particular component type.
    pub fn read_storage<C: Component>(&self) -> EcsStorage<C, Fetch<EcsMaskedStorage<C>>> {
        self.ecs.read_storage::<C>()
    }

    /// Get a reference to the internal ECS world.
    pub fn ecs(&self) -> &sphynx::World<EcsCompPacket, EcsResPacket> {
        &self.ecs
    }

    /// Get a mutable reference to the internal ECS world.
    pub fn ecs_mut(&mut self) -> &mut sphynx::World<EcsCompPacket, EcsResPacket> {
        &mut self.ecs
    }

    /// Get a reference to the `Changes` structure of the state. This contains
    /// information about state that has changed since the last game tick.
    pub fn chunk_changes(&self) -> Fetch<ChunkChanges> {
        self.ecs.read_resource()
    }

    /// Get the current in-game time of day.
    ///
    /// Note that this should not be used for physics, animations or other such localised timings.
    pub fn get_time_of_day(&self) -> f64 {
        self.ecs.read_resource::<TimeOfDay>().0
    }

    /// Get the current in-game time.
    ///
    /// Note that this does not correspond to the time of day.
    pub fn get_time(&self) -> f64 {
        self.ecs.read_resource::<Time>().0
    }

    /// Get the current delta time.
    pub fn get_delta_time(&self) -> f32 {
        self.ecs.read_resource::<DeltaTime>().0
    }

    /// Get a reference to this state's terrain.
    pub fn terrain(&self) -> Fetch<TerrainMap> {
        self.ecs.read_resource()
    }

    /// Get a writable reference to this state's terrain.
    pub fn terrain_mut(&self) -> FetchMut<TerrainMap> {
        self.ecs.write_resource()
    }

    /// Removes every chunk of the terrain.
    pub fn clear_terrain(&mut self) {
        let keys = self
            .terrain_mut()
            .drain()
            .map(|(key, _)| key)
            .collect::<Vec<_>>();

        for key in keys {
            self.remove_chunk(key);
        }
    }

    /// Insert the provided chunk into this state's terrain.
    pub fn insert_chunk(&mut self, key: Vec2<i32>, chunk: TerrainChunk) {
        if self
            .ecs
            .write_resource::<TerrainMap>()
            .insert(key, Arc::new(chunk))
            .is_some()
        {
            self.ecs
                .write_resource::<ChunkChanges>()
                .modified_chunks
                .insert(key);
        } else {
            self.ecs
                .write_resource::<ChunkChanges>()
                .new_chunks
                .insert(key);
        }
    }

    /// Remove the chunk with the given key from this state's terrain, if it exists.
    pub fn remove_chunk(&mut self, key: Vec2<i32>) {
        if self
            .ecs
            .write_resource::<TerrainMap>()
            .remove(key)
            .is_some()
        {
            self.ecs
                .write_resource::<ChunkChanges>()
                .removed_chunks
                .insert(key);
        }
    }

    /// Execute a single tick, simulating the game state by the given duration.
    pub fn tick(&mut self, dt: Duration) {
        // Change the time accordingly.
        self.ecs.write_resource::<TimeOfDay>().0 += dt.as_secs_f64() * DAY_CYCLE_FACTOR;
        self.ecs.write_resource::<Time>().0 += dt.as_secs_f64();

        // Update delta time.
        // Beyond a delta time of MAX_DELTA_TIME, start lagging to avoid skipping important physics events.
        self.ecs.write_resource::<DeltaTime>().0 = dt.as_secs_f32().min(MAX_DELTA_TIME);

        // Run systems to update the world.
        // Create and run a dispatcher for ecs systems.
        let mut dispatch_builder = DispatcherBuilder::new().with_pool(self.thread_pool.clone());
        sys::add_local_systems(&mut dispatch_builder);
        // This dispatches all the systems in parallel.
        dispatch_builder.build().dispatch(&self.ecs.res);

        self.ecs.maintain();

        // Apply terrain changes
        let mut terrain = self.ecs.write_resource::<TerrainMap>();
        let mut chunk_changes = self.ecs.write_resource::<ChunkChanges>();
        self.ecs
            .write_resource::<TerrainChange>()
            .blocks
            .drain()
            .for_each(|(pos, block)| {
                if terrain.set(pos, block).is_ok() {
                    chunk_changes.modified_chunks.insert(terrain.pos_key(pos));
                } else {
                    warn!("Tried to modify block outside of terrain at {:?}", pos);
                }
            });
    }

    /// Clean up the state after a tick.
    pub fn cleanup(&mut self) {
        // Clean up data structures from the last tick.
        self.ecs.write_resource::<TerrainChange>().clear();
        self.ecs.write_resource::<ChunkChanges>().clear();
    }
}
