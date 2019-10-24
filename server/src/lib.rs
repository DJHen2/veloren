#![deny(unsafe_code)]
#![feature(drain_filter)]

pub mod auth_provider;
pub mod chunk_generator;
pub mod client;
pub mod cmd;
pub mod error;
pub mod input;
pub mod metrics;
pub mod settings;
pub mod sys;

// Reexports
pub use crate::{error::Error, input::Input, settings::ServerSettings};

use crate::{
    auth_provider::AuthProvider,
    chunk_generator::ChunkGenerator,
    client::{Client, RegionSubscription},
    cmd::CHAT_COMMANDS,
};
use common::{
    assets, comp,
    effect::Effect,
    event::{EventBus, ServerEvent},
    msg::{ClientMsg, ClientState, ServerError, ServerInfo, ServerMsg},
    net::PostOffice,
    state::{BlockChange, State, TimeOfDay, Uid},
    terrain::{block::Block, TerrainChunkSize, TerrainGrid},
    vol::{ReadVol, RectVolSize, Vox},
};
use log::{debug, trace};
use metrics::ServerMetrics;
use rand::Rng;
use specs::{join::Join, world::EntityBuilder as EcsEntityBuilder, Builder, Entity as EcsEntity};
use std::{
    i32,
    sync::Arc,
    time::{Duration, Instant},
};
use uvth::{ThreadPool, ThreadPoolBuilder};
use vek::*;
use world::World;

const CLIENT_TIMEOUT: f64 = 20.0; // Seconds

pub enum Event {
    ClientConnected {
        entity: EcsEntity,
    },
    ClientDisconnected {
        entity: EcsEntity,
    },
    Chat {
        entity: Option<EcsEntity>,
        msg: String,
    },
}

#[derive(Copy, Clone)]
struct SpawnPoint(Vec3<f32>);

// Tick count used for throttling network updates
// Note this doesn't account for dt (so update rate changes with tick rate)
#[derive(Copy, Clone, Default)]
pub struct Tick(u64);

pub struct Server {
    state: State,
    world: Arc<World>,

    postoffice: PostOffice<ServerMsg, ClientMsg>,

    thread_pool: ThreadPool,

    server_info: ServerInfo,
    metrics: ServerMetrics,

    server_settings: ServerSettings,
}

impl Server {
    /// Create a new `Server`
    pub fn new(settings: ServerSettings) -> Result<Self, Error> {
        let mut state = State::default();
        state
            .ecs_mut()
            .add_resource(SpawnPoint(Vec3::new(16_384.0, 16_384.0, 600.0)));
        state
            .ecs_mut()
            .add_resource(EventBus::<ServerEvent>::default());
        // TODO: anything but this
        state.ecs_mut().add_resource(AuthProvider::new());
        state.ecs_mut().add_resource(Tick(0));
        state.ecs_mut().add_resource(ChunkGenerator::new());
        // System timers
        state
            .ecs_mut()
            .add_resource(sys::EntitySyncTimer::default());
        state.ecs_mut().add_resource(sys::MessageTimer::default());
        state
            .ecs_mut()
            .add_resource(sys::SubscriptionTimer::default());
        state
            .ecs_mut()
            .add_resource(sys::TerrainSyncTimer::default());
        state.ecs_mut().add_resource(sys::TerrainTimer::default());
        // Server-only components
        state.ecs_mut().register::<RegionSubscription>();
        state.ecs_mut().register::<Client>();

        // Set starting time for the server.
        state.ecs_mut().write_resource::<TimeOfDay>().0 = settings.start_time;

        let this = Self {
            state,
            world: Arc::new(World::generate(settings.world_seed)),

            postoffice: PostOffice::bind(settings.gameserver_address)?,

            thread_pool: ThreadPoolBuilder::new()
                .name("veloren-worker".into())
                .build(),

            server_info: ServerInfo {
                name: settings.server_name.clone(),
                description: settings.server_description.clone(),
                git_hash: common::util::GIT_HASH.to_string(),
                git_date: common::util::GIT_DATE.to_string(),
            },
            metrics: ServerMetrics::new(settings.metrics_address)
                .expect("Failed to initialize server metrics submodule."),
            server_settings: settings.clone(),
        };
        debug!("created veloren server");
        trace!("server configuration: {:?}", &settings);

        Ok(this)
    }

    pub fn with_thread_pool(mut self, thread_pool: ThreadPool) -> Self {
        self.thread_pool = thread_pool;
        self
    }

    /// Get a reference to the server's game state.
    pub fn state(&self) -> &State {
        &self.state
    }
    /// Get a mutable reference to the server's game state.
    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    /// Get a reference to the server's world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Build a static object entity
    pub fn create_object(
        &mut self,
        pos: comp::Pos,
        object: comp::object::Body,
    ) -> EcsEntityBuilder {
        self.state
            .ecs_mut()
            .create_entity_synced()
            .with(pos)
            .with(comp::Vel(Vec3::zero()))
            .with(comp::Ori(Vec3::unit_y()))
            .with(comp::Body::Object(object))
            .with(comp::Mass(100.0))
            .with(comp::Gravity(1.0))
        //.with(comp::LightEmitter::default())
    }

    /// Build a projectile
    pub fn create_projectile(
        state: &mut State,
        pos: comp::Pos,
        vel: comp::Vel,
        body: comp::Body,
        projectile: comp::Projectile,
    ) -> EcsEntityBuilder {
        state
            .ecs_mut()
            .create_entity_synced()
            .with(pos)
            .with(vel)
            .with(comp::Ori(vel.0.normalized()))
            .with(comp::Mass(0.0))
            .with(body)
            .with(projectile)
            .with(comp::Sticky)
    }

    pub fn create_player_character(
        state: &mut State,
        entity: EcsEntity,
        name: String,
        body: comp::Body,
        main: Option<String>,
        server_settings: &ServerSettings,
    ) {
        // Give no item when an invalid specifier is given
        let main = main.and_then(|specifier| assets::load_cloned(&specifier).ok());

        let spawn_point = state.ecs().read_resource::<SpawnPoint>().0;

        state.write_component(entity, body);
        state.write_component(entity, comp::Stats::new(name, main));
        state.write_component(entity, comp::Controller::default());
        state.write_component(entity, comp::Pos(spawn_point));
        state.write_component(entity, comp::Vel(Vec3::zero()));
        state.write_component(entity, comp::Ori(Vec3::unit_y()));
        state.write_component(entity, comp::Gravity(1.0));
        state.write_component(entity, comp::CharacterState::default());
        state.write_component(entity, comp::Inventory::default());
        state.write_component(entity, comp::InventoryUpdate);
        // Make sure physics are accepted.
        state.write_component(entity, comp::ForceUpdate);

        // Give the Admin component to the player if their name exists in admin list
        if server_settings.admins.contains(
            &state
                .ecs()
                .read_storage::<comp::Player>()
                .get(entity)
                .expect("Failed to fetch entity.")
                .alias,
        ) {
            state.write_component(entity, comp::Admin);
        }
        // Tell the client its request was successful.
        if let Some(client) = state.ecs().write_storage::<Client>().get_mut(entity) {
            client.allow_state(ClientState::Character);
        }
    }

    /// Handle events coming through via the event bus
    fn handle_events(&mut self) -> Vec<Event> {
        let mut frontend_events = Vec::new();

        let mut requested_chunks = Vec::new();
        let mut dropped_items = Vec::new();
        let mut chat_commands = Vec::new();

        let events = self
            .state
            .ecs()
            .read_resource::<EventBus<ServerEvent>>()
            .recv_all();
        for event in events {
            let state = &mut self.state;

            let server_settings = &self.server_settings;

            let mut todo_remove = None;

            match event {
                ServerEvent::Explosion { pos, radius } => {
                    const RAYS: usize = 500;

                    for _ in 0..RAYS {
                        let dir = Vec3::new(
                            rand::random::<f32>() - 0.5,
                            rand::random::<f32>() - 0.5,
                            rand::random::<f32>() - 0.5,
                        )
                        .normalized();

                        let ecs = state.ecs();
                        let mut block_change = ecs.write_resource::<BlockChange>();

                        let _ = ecs
                            .read_resource::<TerrainGrid>()
                            .ray(pos, pos + dir * radius)
                            .until(|_| rand::random::<f32>() < 0.05)
                            .for_each(|pos| block_change.set(pos, Block::empty()))
                            .cast();
                    }
                }

                ServerEvent::Shoot {
                    entity,
                    dir,
                    body,
                    light,
                    projectile,
                    gravity,
                } => {
                    let mut pos = state
                        .ecs()
                        .read_storage::<comp::Pos>()
                        .get(entity)
                        .expect("Failed to fetch entity")
                        .0;

                    // TODO: Player height
                    pos.z += 1.2;

                    let mut builder = Self::create_projectile(
                        state,
                        comp::Pos(pos),
                        comp::Vel(dir * 100.0),
                        body,
                        projectile,
                    );
                    if let Some(light) = light {
                        builder = builder.with(light)
                    }
                    if let Some(gravity) = gravity {
                        builder = builder.with(gravity)
                    }

                    builder.build();
                }

                ServerEvent::Damage { uid, change } => {
                    let ecs = state.ecs();
                    if let Some(entity) = ecs.entity_from_uid(uid.into()) {
                        if let Some(stats) = ecs.write_storage::<comp::Stats>().get_mut(entity) {
                            stats.health.change_by(change);
                        }
                    }
                }

                ServerEvent::Destroy { entity, cause } => {
                    let ecs = state.ecs();
                    // Chat message
                    if let Some(player) = ecs.read_storage::<comp::Player>().get(entity) {
                        let msg = if let comp::HealthSource::Attack { by } = cause {
                            ecs.entity_from_uid(by.into()).and_then(|attacker| {
                                ecs.read_storage::<comp::Player>().get(attacker).map(
                                    |attacker_alias| {
                                        format!(
                                            "{} was killed by {}",
                                            &player.alias, &attacker_alias.alias
                                        )
                                    },
                                )
                            })
                        } else {
                            None
                        }
                        .unwrap_or(format!("{} died", &player.alias));

                        state.notify_registered_clients(ServerMsg::kill(msg));
                    }

                    // Give EXP to the killer if entity had stats
                    let mut stats = ecs.write_storage::<comp::Stats>();

                    if let Some(entity_stats) = stats.get(entity).cloned() {
                        if let comp::HealthSource::Attack { by } = cause {
                            ecs.entity_from_uid(by.into()).map(|attacker| {
                                if let Some(attacker_stats) = stats.get_mut(attacker) {
                                    // TODO: Discuss whether we should give EXP by Player Killing or not.
                                    attacker_stats
                                        .exp
                                        .change_by((entity_stats.level.level() * 10) as i64);
                                }
                            });
                        }
                    }

                    if let Some(client) = ecs.write_storage::<Client>().get_mut(entity) {
                        let _ = ecs.write_storage().insert(entity, comp::Vel(Vec3::zero()));
                        let _ = ecs.write_storage().insert(entity, comp::ForceUpdate);
                        client.force_state(ClientState::Dead);
                    } else {
                        todo_remove = Some(entity.clone());
                    }
                }

                ServerEvent::InventoryManip(entity, manip) => {
                    match manip {
                        comp::InventoryManip::Pickup(uid) => {
                            // TODO: enforce max pickup range
                            let item_entity = if let (Some((item, item_entity)), Some(inv)) = (
                                state
                                    .ecs()
                                    .entity_from_uid(uid.into())
                                    .and_then(|item_entity| {
                                        state
                                            .ecs()
                                            .write_storage::<comp::Item>()
                                            .get_mut(item_entity)
                                            .map(|item| (item.clone(), item_entity))
                                    }),
                                state
                                    .ecs()
                                    .write_storage::<comp::Inventory>()
                                    .get_mut(entity),
                            ) {
                                if inv.push(item).is_none() {
                                    Some(item_entity)
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            if let Some(item_entity) = item_entity {
                                let _ = state.ecs_mut().delete_entity_synced(item_entity);
                            }

                            state.write_component(entity, comp::InventoryUpdate);
                        }

                        comp::InventoryManip::Collect(pos) => {
                            let block = state.terrain().get(pos).ok().copied();
                            if let Some(block) = block {
                                if block.is_collectible()
                                    && state
                                        .ecs()
                                        .read_storage::<comp::Inventory>()
                                        .get(entity)
                                        .map(|inv| !inv.is_full())
                                        .unwrap_or(false)
                                {
                                    if state.try_set_block(pos, Block::empty()).is_some() {
                                        comp::Item::try_reclaim_from_block(block)
                                            .map(|item| state.give_item(entity, item));
                                    }
                                }
                            }
                        }

                        comp::InventoryManip::Use(slot) => {
                            let item_opt = state
                                .ecs()
                                .write_storage::<comp::Inventory>()
                                .get_mut(entity)
                                .and_then(|inv| inv.remove(slot));

                            match item_opt {
                                Some(item) => match item.kind {
                                    comp::ItemKind::Tool { .. } => {
                                        if let Some(stats) = state
                                            .ecs()
                                            .write_storage::<comp::Stats>()
                                            .get_mut(entity)
                                        {
                                            // Insert old item into inventory
                                            if let Some(old_item) = stats.equipment.main.take() {
                                                state
                                                    .ecs()
                                                    .write_storage::<comp::Inventory>()
                                                    .get_mut(entity)
                                                    .map(|inv| inv.insert(slot, old_item));
                                            }

                                            stats.equipment.main = Some(item);
                                        }
                                    }
                                    comp::ItemKind::Consumable { effect, .. } => {
                                        state.apply_effect(entity, effect);
                                    }
                                    _ => {
                                        // Re-insert it if unused
                                        let _ = state
                                            .ecs()
                                            .write_storage::<comp::Inventory>()
                                            .get_mut(entity)
                                            .map(|inv| inv.insert(slot, item));
                                    }
                                },
                                _ => {}
                            }

                            state.write_component(entity, comp::InventoryUpdate);
                        }

                        comp::InventoryManip::Swap(a, b) => {
                            state
                                .ecs()
                                .write_storage::<comp::Inventory>()
                                .get_mut(entity)
                                .map(|inv| inv.swap_slots(a, b));
                            state.write_component(entity, comp::InventoryUpdate);
                        }

                        comp::InventoryManip::Drop(slot) => {
                            let item = state
                                .ecs()
                                .write_storage::<comp::Inventory>()
                                .get_mut(entity)
                                .and_then(|inv| inv.remove(slot));

                            if let (Some(item), Some(pos)) =
                                (item, state.ecs().read_storage::<comp::Pos>().get(entity))
                            {
                                dropped_items.push((
                                    *pos,
                                    state
                                        .ecs()
                                        .read_storage::<comp::Ori>()
                                        .get(entity)
                                        .copied()
                                        .unwrap_or(comp::Ori(Vec3::unit_y())),
                                    item,
                                ));
                            }
                            state.write_component(entity, comp::InventoryUpdate);
                        }
                    }
                }

                ServerEvent::Respawn(entity) => {
                    // Only clients can respawn
                    if let Some(client) = state.ecs().write_storage::<Client>().get_mut(entity) {
                        let respawn_point = state
                            .read_component_cloned::<comp::Waypoint>(entity)
                            .map(|wp| wp.get_pos())
                            .unwrap_or(state.ecs().read_resource::<SpawnPoint>().0);

                        client.allow_state(ClientState::Character);
                        state
                            .ecs()
                            .write_storage::<comp::Stats>()
                            .get_mut(entity)
                            .map(|stats| stats.revive());
                        state
                            .ecs()
                            .write_storage::<comp::Pos>()
                            .get_mut(entity)
                            .map(|pos| pos.0 = respawn_point);
                        let _ = state
                            .ecs()
                            .write_storage()
                            .insert(entity, comp::ForceUpdate);
                    }
                }

                ServerEvent::LandOnGround { entity, vel } => {
                    if vel.z <= -25.0 {
                        if let Some(stats) =
                            state.ecs().write_storage::<comp::Stats>().get_mut(entity)
                        {
                            let falldmg = (vel.z / 5.0) as i32;
                            if falldmg < 0 {
                                stats.health.change_by(comp::HealthChange {
                                    amount: falldmg,
                                    cause: comp::HealthSource::World,
                                });
                            }
                        }
                    }
                }

                ServerEvent::Mount(mounter, mountee) => {
                    if state
                        .ecs()
                        .read_storage::<comp::Mounting>()
                        .get(mounter)
                        .is_none()
                    {
                        let not_mounting_yet = if let Some(comp::MountState::Unmounted) = state
                            .ecs()
                            .write_storage::<comp::MountState>()
                            .get_mut(mountee)
                            .cloned()
                        {
                            true
                        } else {
                            false
                        };

                        if not_mounting_yet {
                            if let (Some(mounter_uid), Some(mountee_uid)) = (
                                state.ecs().uid_from_entity(mounter),
                                state.ecs().uid_from_entity(mountee),
                            ) {
                                state.write_component(
                                    mountee,
                                    comp::MountState::MountedBy(mounter_uid.into()),
                                );
                                state.write_component(mounter, comp::Mounting(mountee_uid.into()));
                            }
                        }
                    }
                }

                ServerEvent::Unmount(mounter) => {
                    let mountee_entity = state
                        .ecs()
                        .write_storage::<comp::Mounting>()
                        .get(mounter)
                        .and_then(|mountee| state.ecs().entity_from_uid(mountee.0.into()));
                    if let Some(mountee_entity) = mountee_entity {
                        state
                            .ecs()
                            .write_storage::<comp::MountState>()
                            .get_mut(mountee_entity)
                            .map(|ms| *ms = comp::MountState::Unmounted);
                    }
                    state.delete_component::<comp::Mounting>(mounter);
                }

                ServerEvent::Possess(possessor_uid, possesse_uid) => {
                    let ecs = state.ecs();
                    if let (Some(possessor), Some(possesse)) = (
                        ecs.entity_from_uid(possessor_uid.into()),
                        ecs.entity_from_uid(possesse_uid.into()),
                    ) {
                        // You can't possess other players
                        let mut clients = ecs.write_storage::<Client>();
                        if clients.get_mut(possesse).is_none() {
                            if let Some(mut client) = clients.remove(possessor) {
                                client.notify(ServerMsg::SetPlayerEntity(possesse_uid.into()));
                                let _ = clients.insert(possesse, client);
                                // Create inventory if it doesn't exist
                                {
                                    let mut inventories = ecs.write_storage::<comp::Inventory>();
                                    if let Some(inventory) = inventories.get_mut(possesse) {
                                        inventory.push(assets::load_expect_cloned(
                                            "common.items.debug.possess",
                                        ));
                                    } else {
                                        let _ = inventories.insert(
                                            possesse,
                                            comp::Inventory {
                                                slots: vec![Some(assets::load_expect_cloned(
                                                    "common.items.debug.possess",
                                                ))],
                                            },
                                        );
                                    }
                                }
                                let _ = ecs
                                    .write_storage::<comp::InventoryUpdate>()
                                    .insert(possesse, comp::InventoryUpdate);
                                // Move player component
                                {
                                    let mut players = ecs.write_storage::<comp::Player>();
                                    if let Some(player) = players.remove(possessor) {
                                        let _ = players.insert(possesse, player);
                                    }
                                }
                                // Remove will of the entity
                                let _ = ecs.write_storage::<comp::Agent>().remove(possesse);
                                // Transfer admin powers
                                {
                                    let mut admins = ecs.write_storage::<comp::Admin>();
                                    if let Some(admin) = admins.remove(possessor) {
                                        let _ = admins.insert(possesse, admin);
                                    }
                                }
                                // Transfer waypoint
                                {
                                    let mut waypoints = ecs.write_storage::<comp::Waypoint>();
                                    if let Some(waypoint) = waypoints.remove(possessor) {
                                        let _ = waypoints.insert(possesse, waypoint);
                                    }
                                }
                            }
                        }
                    }
                }

                ServerEvent::CreatePlayer {
                    entity,
                    name,
                    body,
                    main,
                } => {
                    Self::create_player_character(
                        state,
                        entity,
                        name,
                        body,
                        main,
                        &server_settings,
                    );
                    Self::initialize_region_subscription(state, entity);
                }

                ServerEvent::CreateNpc {
                    pos,
                    stats,
                    body,
                    agent,
                    scale,
                } => {
                    state
                        .create_npc(pos, stats, body)
                        .with(agent)
                        .with(scale)
                        .build();
                }

                ServerEvent::ClientDisconnect(entity) => {
                    if let Err(err) = state.ecs_mut().delete_entity_synced(entity) {
                        debug!("Failed to delete disconnected client: {:?}", err);
                    }

                    frontend_events.push(Event::ClientDisconnected { entity });
                }

                ServerEvent::ChunkRequest(entity, key) => {
                    requested_chunks.push((entity, key));
                }

                ServerEvent::ChatCmd(entity, cmd) => {
                    chat_commands.push((entity, cmd));
                }
            }

            // TODO: is this needed?
            if let Some(entity) = todo_remove {
                let _ = state.ecs_mut().delete_entity_synced(entity);
            }
        }

        // Generate requested chunks.
        for (entity, key) in requested_chunks {
            self.generate_chunk(entity, key);
        }

        // Drop items
        for (pos, ori, item) in dropped_items {
            let vel = ori.0.normalized() * 5.0
                + Vec3::unit_z() * 10.0
                + Vec3::<f32>::zero().map(|_| rand::thread_rng().gen::<f32>() - 0.5) * 4.0;
            self.create_object(Default::default(), comp::object::Body::Pouch)
                .with(comp::Pos(pos.0 + Vec3::unit_z() * 0.25))
                .with(item)
                .with(comp::Vel(vel))
                .build();
        }

        for (entity, cmd) in chat_commands {
            self.process_chat_cmd(entity, cmd);
        }

        frontend_events
    }

    /// Execute a single server tick, handle input and update the game state by the given duration.
    pub fn tick(&mut self, _input: Input, dt: Duration) -> Result<Vec<Event>, Error> {
        self.state.ecs().write_resource::<Tick>().0 += 1;
        // This tick function is the centre of the Veloren universe. Most server-side things are
        // managed from here, and as such it's important that it stays organised. Please consult
        // the core developers before making significant changes to this code. Here is the
        // approximate order of things. Please update it as this code changes.
        //
        // 1) Collect input from the frontend, apply input effects to the state of the game
        // 2) Go through any events (timer-driven or otherwise) that need handling and apply them
        //    to the state of the game
        // 3) Go through all incoming client network communications, apply them to the game state
        // 4) Perform a single LocalState tick (i.e: update the world and entities in the world)
        // 5) Go through the terrain update queue and apply all changes to the terrain
        // 6) Send relevant state updates to all clients
        // 7) Update Metrics with current data
        // 8) Finish the tick, passing control of the main thread back to the frontend

        let before_tick_1 = Instant::now();
        // 1) Build up a list of events for this frame, to be passed to the frontend.
        let mut frontend_events = Vec::new();

        // If networking has problems, handle them.
        if let Some(err) = self.postoffice.error() {
            return Err(err.into());
        }

        // 2)

        // 3) Handle inputs from clients
        frontend_events.append(&mut self.handle_new_connections()?);

        // Handle game events
        frontend_events.append(&mut self.handle_events());

        let before_tick_4 = Instant::now();
        // 4) Tick the client's LocalState.
        self.state.tick(dt, sys::add_server_systems);

        // Tick the world
        self.world.tick(dt);

        // 5) Fetch any generated `TerrainChunk`s and insert them into the terrain.
        // in sys/terrain.rs

        let before_tick_6 = Instant::now();
        // 6) Synchronise clients with the new state of the world.
        // TODO: Remove sphynx
        // Sync 'logical' state using Sphynx.
        let sync_package = self.state.ecs_mut().next_sync_package();
        self.state
            .notify_registered_clients(ServerMsg::EcsSync(sync_package));

        // Remove NPCs that are outside the view distances of all players
        // This is done by removing NPCs in unloaded chunks
        let to_delete = {
            let terrain = self.state.terrain();
            (
                &self.state.ecs().entities(),
                &self.state.ecs().read_storage::<comp::Pos>(),
                &self.state.ecs().read_storage::<comp::Agent>(),
            )
                .join()
                .filter(|(_, pos, _)| terrain.get(pos.0.map(|e| e.floor() as i32)).is_err())
                .map(|(entity, _, _)| entity)
                .collect::<Vec<_>>()
        };
        for entity in to_delete {
            let _ = self.state.ecs_mut().delete_entity(entity);
        }

        let before_tick_7 = Instant::now();
        // 7) Update Metrics
        let entity_sync_nanos = self
            .state
            .ecs()
            .read_resource::<sys::EntitySyncTimer>()
            .nanos as i64;
        let message_nanos = self.state.ecs().read_resource::<sys::MessageTimer>().nanos as i64;
        let subscription_nanos = self
            .state
            .ecs()
            .read_resource::<sys::SubscriptionTimer>()
            .nanos as i64;
        let terrain_sync_nanos = self
            .state
            .ecs()
            .read_resource::<sys::TerrainSyncTimer>()
            .nanos as i64;
        let terrain_nanos = self.state.ecs().read_resource::<sys::TerrainTimer>().nanos as i64;
        let total_sys_nanos = entity_sync_nanos
            + message_nanos
            + subscription_nanos
            + terrain_sync_nanos
            + terrain_nanos;
        self.metrics
            .tick_time
            .with_label_values(&["input"])
            .set((before_tick_4 - before_tick_1).as_nanos() as i64);
        self.metrics
            .tick_time
            .with_label_values(&["state tick"])
            .set((before_tick_6 - before_tick_4).as_nanos() as i64 - total_sys_nanos);
        self.metrics
            .tick_time
            .with_label_values(&["sphynx sync"])
            .set((before_tick_7 - before_tick_6).as_nanos() as i64);
        self.metrics
            .tick_time
            .with_label_values(&["entity sync"])
            .set(entity_sync_nanos);
        self.metrics
            .tick_time
            .with_label_values(&["message"])
            .set(message_nanos);
        self.metrics
            .tick_time
            .with_label_values(&["subscription"])
            .set(subscription_nanos);
        self.metrics
            .tick_time
            .with_label_values(&["terrain sync"])
            .set(terrain_sync_nanos);
        self.metrics
            .tick_time
            .with_label_values(&["terrain"])
            .set(terrain_nanos);
        self.metrics
            .player_online
            .set(self.state.ecs().read_storage::<Client>().join().count() as i64);
        self.metrics
            .time_of_day
            .set(self.state.ecs().read_resource::<TimeOfDay>().0);
        if self.metrics.is_100th_tick() {
            let mut chonk_cnt = 0;
            let chunk_cnt = self.state.terrain().iter().fold(0, |a, (_, c)| {
                chonk_cnt += 1;
                a + c.sub_chunks_len()
            });
            self.metrics.chonks_count.set(chonk_cnt as i64);
            self.metrics.chunks_count.set(chunk_cnt as i64);
        }
        //self.metrics.entity_count.set(self.state.);
        self.metrics
            .tick_time
            .with_label_values(&["metrics"])
            .set(before_tick_7.elapsed().as_nanos() as i64);

        // 8) Finish the tick, pass control back to the frontend.

        Ok(frontend_events)
    }

    /// Clean up the server after a tick.
    pub fn cleanup(&mut self) {
        // Cleanup the local state
        self.state.cleanup();
    }

    /// Handle new client connections.
    fn handle_new_connections(&mut self) -> Result<Vec<Event>, Error> {
        let mut frontend_events = Vec::new();

        for postbox in self.postoffice.new_postboxes() {
            let mut client = Client {
                client_state: ClientState::Connected,
                postbox,
                last_ping: self.state.get_time(),
            };

            if self.server_settings.max_players
                <= self.state.ecs().read_storage::<Client>().join().count()
            {
                // Note: in this case the client is dropped
                client.notify(ServerMsg::Error(ServerError::TooManyPlayers));
            } else {
                let entity = self
                    .state
                    .ecs_mut()
                    .create_entity_synced()
                    .with(client)
                    .build();
                // Return the state of the current world (all of the components that Sphynx tracks).
                log::info!("Starting initial sync with client.");
                self.state
                    .ecs()
                    .write_storage::<Client>()
                    .get_mut(entity)
                    .unwrap()
                    .notify(ServerMsg::InitialSync {
                        ecs_state: self.state.ecs().gen_state_package(),
                        entity_uid: self.state.ecs().uid_from_entity(entity).unwrap().into(), // Can't fail.
                        server_info: self.server_info.clone(),
                        // world_map: (WORLD_SIZE/*, self.world.sim().get_map()*/),
                    });
                log::info!("Done initial sync with client.");

                frontend_events.push(Event::ClientConnected { entity });
            }
        }

        Ok(frontend_events)
    }

    /// Initialize region subscription
    fn initialize_region_subscription(state: &mut State, entity: specs::Entity) {
        let mut subscription = None;

        if let (Some(client_pos), Some(client_vd), Some(client)) = (
            state.ecs().read_storage::<comp::Pos>().get(entity),
            state
                .ecs()
                .read_storage::<comp::Player>()
                .get(entity)
                .map(|pl| pl.view_distance)
                .and_then(|v| v),
            state.ecs().write_storage::<Client>().get_mut(entity),
        ) {
            use common::region::RegionMap;

            let fuzzy_chunk = (Vec2::<f32>::from(client_pos.0))
                .map2(TerrainChunkSize::RECT_SIZE, |e, sz| e as i32 / sz as i32);
            let chunk_size = TerrainChunkSize::RECT_SIZE.reduce_max() as f32;
            let regions = common::region::regions_in_vd(
                client_pos.0,
                (client_vd as f32 * chunk_size) as f32
                    + (client::CHUNK_FUZZ as f32 + chunk_size) * 2.0f32.sqrt(),
            );

            for (_, region) in state
                .ecs()
                .read_resource::<RegionMap>()
                .iter()
                .filter(|(key, _)| regions.contains(key))
            {
                // Sync physics of all entities in this region
                for (&uid, &pos, vel, ori, character_state, _) in (
                    &state.ecs().read_storage::<Uid>(),
                    &state.ecs().read_storage::<comp::Pos>(), // We assume all these entities have a position
                    state.ecs().read_storage::<comp::Vel>().maybe(),
                    state.ecs().read_storage::<comp::Ori>().maybe(),
                    state.ecs().read_storage::<comp::CharacterState>().maybe(),
                    region.entities(),
                )
                    .join()
                {
                    client.notify(ServerMsg::EntityPos {
                        entity: uid.into(),
                        pos,
                    });
                    if let Some(vel) = vel.copied() {
                        client.notify(ServerMsg::EntityVel {
                            entity: uid.into(),
                            vel,
                        });
                    }
                    if let Some(ori) = ori.copied() {
                        client.notify(ServerMsg::EntityOri {
                            entity: uid.into(),
                            ori,
                        });
                    }
                    if let Some(character_state) = character_state.copied() {
                        client.notify(ServerMsg::EntityCharacterState {
                            entity: uid.into(),
                            character_state,
                        });
                    }
                }
            }

            subscription = Some(RegionSubscription {
                fuzzy_chunk,
                regions,
            });
        }
        if let Some(subscription) = subscription {
            state.write_component(entity, subscription);
        }
    }

    pub fn notify_client(&self, entity: EcsEntity, msg: ServerMsg) {
        if let Some(client) = self.state.ecs().write_storage::<Client>().get_mut(entity) {
            client.notify(msg)
        }
    }

    pub fn generate_chunk(&mut self, entity: EcsEntity, key: Vec2<i32>) {
        self.state
            .ecs()
            .write_resource::<ChunkGenerator>()
            .generate_chunk(entity, key, &mut self.thread_pool, self.world.clone());
    }

    fn process_chat_cmd(&mut self, entity: EcsEntity, cmd: String) {
        // Separate string into keyword and arguments.
        let sep = cmd.find(' ');
        let (kwd, args) = match sep {
            Some(i) => (cmd[..i].to_string(), cmd[(i + 1)..].to_string()),
            None => (cmd, "".to_string()),
        };

        // Find the command object and run its handler.
        let action_opt = CHAT_COMMANDS.iter().find(|x| x.keyword == kwd);
        match action_opt {
            Some(action) => action.execute(self, entity, args),
            // Unknown command
            None => {
                if let Some(client) = self.state.ecs().write_storage::<Client>().get_mut(entity) {
                    client.notify(ServerMsg::private(format!(
                        "Unknown command '/{}'.\nType '/help' for available commands",
                        kwd
                    )));
                }
            }
        }
    }

    fn entity_is_admin(&self, entity: EcsEntity) -> bool {
        self.state
            .read_storage::<comp::Admin>()
            .get(entity)
            .is_some()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.state.notify_registered_clients(ServerMsg::Shutdown);
    }
}

trait StateExt {
    fn give_item(&mut self, entity: EcsEntity, item: comp::Item) -> bool;
    fn apply_effect(&mut self, entity: EcsEntity, effect: Effect);
    fn notify_registered_clients(&self, msg: ServerMsg);
    fn create_npc(
        &mut self,
        pos: comp::Pos,
        stats: comp::Stats,
        body: comp::Body,
    ) -> EcsEntityBuilder;
}

impl StateExt for State {
    fn give_item(&mut self, entity: EcsEntity, item: comp::Item) -> bool {
        let success = self
            .ecs()
            .write_storage::<comp::Inventory>()
            .get_mut(entity)
            .map(|inv| inv.push(item).is_none())
            .unwrap_or(false);
        if success {
            self.write_component(entity, comp::InventoryUpdate);
        }
        success
    }

    fn apply_effect(&mut self, entity: EcsEntity, effect: Effect) {
        match effect {
            Effect::Health(change) => {
                self.ecs()
                    .write_storage::<comp::Stats>()
                    .get_mut(entity)
                    .map(|stats| stats.health.change_by(change));
            }
            Effect::Xp(xp) => {
                self.ecs()
                    .write_storage::<comp::Stats>()
                    .get_mut(entity)
                    .map(|stats| stats.exp.change_by(xp));
            }
        }
    }

    /// Build a non-player character.
    fn create_npc(
        &mut self,
        pos: comp::Pos,
        stats: comp::Stats,
        body: comp::Body,
    ) -> EcsEntityBuilder {
        self.ecs_mut()
            .create_entity_synced()
            .with(pos)
            .with(comp::Vel(Vec3::zero()))
            .with(comp::Ori(Vec3::unit_y()))
            .with(comp::Controller::default())
            .with(body)
            .with(stats)
            .with(comp::Gravity(1.0))
            .with(comp::CharacterState::default())
    }

    fn notify_registered_clients(&self, msg: ServerMsg) {
        for client in (&mut self.ecs().write_storage::<Client>())
            .join()
            .filter(|c| c.is_registered())
        {
            client.notify(msg.clone())
        }
    }
}
