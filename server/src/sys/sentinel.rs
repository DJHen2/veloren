use super::SysTimer;
use common::{
    comp::{
        AbilityPool, Body, CanBuild, Energy, Gravity, Item, LightEmitter, Mass, MountState,
        Mounting, Player, Scale, Stats, Sticky,
    },
    msg::EcsCompPacket,
    sync::{EntityPackage, SyncPackage, Uid, UpdateTracker, WorldSyncExt},
};
use hashbrown::HashMap;
use specs::{
    shred::ResourceId, Entity as EcsEntity, Join, ReadExpect, ReadStorage, System, SystemData,
    World, Write, WriteExpect,
};
use vek::*;

/// Always watching
/// This system will monitor specific components for insertion, removal, and modification
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Write<'a, SysTimer<Self>>,
        TrackedComps<'a>,
        WriteTrackers<'a>,
    );

    fn run(&mut self, (mut timer, comps, mut trackers): Self::SystemData) {
        timer.start();

        record_changes(&comps, &mut trackers);

        timer.end();
    }
}

// Probably more difficult than it needs to be :p
#[derive(SystemData)]
pub struct TrackedComps<'a> {
    pub uid: ReadStorage<'a, Uid>,
    pub body: ReadStorage<'a, Body>,
    pub player: ReadStorage<'a, Player>,
    pub stats: ReadStorage<'a, Stats>,
    pub energy: ReadStorage<'a, Energy>,
    pub can_build: ReadStorage<'a, CanBuild>,
    pub light_emitter: ReadStorage<'a, LightEmitter>,
    pub item: ReadStorage<'a, Item>,
    pub scale: ReadStorage<'a, Scale>,
    pub mounting: ReadStorage<'a, Mounting>,
    pub mount_state: ReadStorage<'a, MountState>,
    pub mass: ReadStorage<'a, Mass>,
    pub sticky: ReadStorage<'a, Sticky>,
    pub gravity: ReadStorage<'a, Gravity>,
    pub ability_pool: ReadStorage<'a, AbilityPool>,
}
impl<'a> TrackedComps<'a> {
    pub fn create_entity_package(&self, entity: EcsEntity) -> EntityPackage<EcsCompPacket> {
        let uid = self
            .uid
            .get(entity)
            .copied()
            .expect("No uid to create an entity package")
            .0;
        let mut comps = Vec::new();
        self.body.get(entity).copied().map(|c| comps.push(c.into()));
        self.player
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.stats
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.energy
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.can_build
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.light_emitter
            .get(entity)
            .copied()
            .map(|c| comps.push(c.into()));
        self.item.get(entity).cloned().map(|c| comps.push(c.into()));
        self.scale
            .get(entity)
            .copied()
            .map(|c| comps.push(c.into()));
        self.mounting
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.mount_state
            .get(entity)
            .cloned()
            .map(|c| comps.push(c.into()));
        self.mass.get(entity).copied().map(|c| comps.push(c.into()));
        self.sticky
            .get(entity)
            .copied()
            .map(|c| comps.push(c.into()));
        self.gravity
            .get(entity)
            .copied()
            .map(|c| comps.push(c.into()));
        self.ability_pool
            .get(entity)
            .copied()
            .map(|c| comps.push(c.into()));

        EntityPackage { uid, comps }
    }
}
#[derive(SystemData)]
pub struct ReadTrackers<'a> {
    pub uid: ReadExpect<'a, UpdateTracker<Uid>>,
    pub body: ReadExpect<'a, UpdateTracker<Body>>,
    pub player: ReadExpect<'a, UpdateTracker<Player>>,
    pub stats: ReadExpect<'a, UpdateTracker<Stats>>,
    pub energy: ReadExpect<'a, UpdateTracker<Energy>>,
    pub can_build: ReadExpect<'a, UpdateTracker<CanBuild>>,
    pub light_emitter: ReadExpect<'a, UpdateTracker<LightEmitter>>,
    pub item: ReadExpect<'a, UpdateTracker<Item>>,
    pub scale: ReadExpect<'a, UpdateTracker<Scale>>,
    pub mounting: ReadExpect<'a, UpdateTracker<Mounting>>,
    pub mount_state: ReadExpect<'a, UpdateTracker<MountState>>,
    pub mass: ReadExpect<'a, UpdateTracker<Mass>>,
    pub sticky: ReadExpect<'a, UpdateTracker<Sticky>>,
    pub gravity: ReadExpect<'a, UpdateTracker<Gravity>>,
    pub ability_pool: ReadExpect<'a, UpdateTracker<AbilityPool>>,
}
impl<'a> ReadTrackers<'a> {
    pub fn create_sync_package(
        &self,
        comps: &TrackedComps,
        filter: impl Join + Copy,
        deleted_entities: Vec<u64>,
    ) -> SyncPackage<EcsCompPacket> {
        SyncPackage::new(&comps.uid, &self.uid, filter, deleted_entities)
            .with_component(&comps.uid, &*self.body, &comps.body, filter)
            .with_component(&comps.uid, &*self.player, &comps.player, filter)
            .with_component(&comps.uid, &*self.stats, &comps.stats, filter)
            .with_component(&comps.uid, &*self.energy, &comps.energy, filter)
            .with_component(&comps.uid, &*self.can_build, &comps.can_build, filter)
            .with_component(
                &comps.uid,
                &*self.light_emitter,
                &comps.light_emitter,
                filter,
            )
            .with_component(&comps.uid, &*self.item, &comps.item, filter)
            .with_component(&comps.uid, &*self.scale, &comps.scale, filter)
            .with_component(&comps.uid, &*self.mounting, &comps.mounting, filter)
            .with_component(&comps.uid, &*self.mount_state, &comps.mount_state, filter)
            .with_component(&comps.uid, &*self.mass, &comps.mass, filter)
            .with_component(&comps.uid, &*self.sticky, &comps.sticky, filter)
            .with_component(&comps.uid, &*self.gravity, &comps.gravity, filter)
            .with_component(&comps.uid, &*self.ability_pool, &comps.ability_pool, filter)
    }
}

#[derive(SystemData)]
pub struct WriteTrackers<'a> {
    uid: WriteExpect<'a, UpdateTracker<Uid>>,
    body: WriteExpect<'a, UpdateTracker<Body>>,
    player: WriteExpect<'a, UpdateTracker<Player>>,
    stats: WriteExpect<'a, UpdateTracker<Stats>>,
    energy: WriteExpect<'a, UpdateTracker<Energy>>,
    can_build: WriteExpect<'a, UpdateTracker<CanBuild>>,
    light_emitter: WriteExpect<'a, UpdateTracker<LightEmitter>>,
    item: WriteExpect<'a, UpdateTracker<Item>>,
    scale: WriteExpect<'a, UpdateTracker<Scale>>,
    mounting: WriteExpect<'a, UpdateTracker<Mounting>>,
    mount_state: WriteExpect<'a, UpdateTracker<MountState>>,
    mass: WriteExpect<'a, UpdateTracker<Mass>>,
    sticky: WriteExpect<'a, UpdateTracker<Sticky>>,
    gravity: WriteExpect<'a, UpdateTracker<Gravity>>,
    ability_pool: WriteExpect<'a, UpdateTracker<AbilityPool>>,
}

fn record_changes(comps: &TrackedComps, trackers: &mut WriteTrackers) {
    // Update trackers
    trackers.uid.record_changes(&comps.uid);
    trackers.body.record_changes(&comps.body);
    trackers.player.record_changes(&comps.player);
    trackers.stats.record_changes(&comps.stats);
    trackers.energy.record_changes(&comps.energy);
    trackers.can_build.record_changes(&comps.can_build);
    trackers.light_emitter.record_changes(&comps.light_emitter);
    trackers.item.record_changes(&comps.item);
    trackers.scale.record_changes(&comps.scale);
    trackers.mounting.record_changes(&comps.mounting);
    trackers.mount_state.record_changes(&comps.mount_state);
    trackers.mass.record_changes(&comps.mass);
    trackers.sticky.record_changes(&comps.sticky);
    trackers.gravity.record_changes(&comps.gravity);
    trackers.ability_pool.record_changes(&comps.ability_pool);
}

pub fn register_trackers(world: &mut World) {
    world.register_tracker::<Uid>();
    world.register_tracker::<Body>();
    world.register_tracker::<Player>();
    world.register_tracker::<Stats>();
    world.register_tracker::<Energy>();
    world.register_tracker::<CanBuild>();
    world.register_tracker::<LightEmitter>();
    world.register_tracker::<Item>();
    world.register_tracker::<Scale>();
    world.register_tracker::<Mounting>();
    world.register_tracker::<MountState>();
    world.register_tracker::<Mass>();
    world.register_tracker::<Sticky>();
    world.register_tracker::<Gravity>();
    world.register_tracker::<AbilityPool>();
}

/// Deleted entities grouped by region
pub struct DeletedEntities {
    map: HashMap<Vec2<i32>, Vec<u64>>,
}

impl Default for DeletedEntities {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl DeletedEntities {
    pub fn record_deleted_entity(&mut self, uid: Uid, region_key: Vec2<i32>) {
        self.map
            .entry(region_key)
            .or_insert(Vec::new())
            .push(uid.into());
    }
    pub fn take_deleted_in_region(&mut self, key: Vec2<i32>) -> Option<Vec<u64>> {
        self.map.remove(&key)
    }
    pub fn get_deleted_in_region(&mut self, key: Vec2<i32>) -> Option<&Vec<u64>> {
        self.map.get(&key)
    }
    pub fn take_remaining_deleted(&mut self) -> Vec<(Vec2<i32>, Vec<u64>)> {
        // TODO: don't allocate
        self.map.drain().collect()
    }
}
