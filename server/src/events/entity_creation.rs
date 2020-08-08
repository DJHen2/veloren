use crate::{sys, Server, StateExt};
use common::{
    comp::{
        self, Agent, Alignment, Body, Gravity, Item, ItemDrop, LightEmitter, Loadout,
        ParticleEmitter, ParticleEmitters, Pos, Projectile, Scale, Stats, Vel, WaypointArea,
    },
    util::Dir,
};
use specs::{Builder, Entity as EcsEntity, WorldExt};
use vek::{Rgb, Vec3};

pub fn handle_initialize_character(server: &mut Server, entity: EcsEntity, character_id: i32) {
    server.state.initialize_character_data(entity, character_id);
}

pub fn handle_loaded_character_data(
    server: &mut Server,
    entity: EcsEntity,
    loaded_components: (comp::Body, comp::Stats, comp::Inventory, comp::Loadout),
) {
    server
        .state
        .update_character_data(entity, loaded_components);
    sys::subscription::initialize_region_subscription(server.state.ecs(), entity);
}

#[allow(clippy::too_many_arguments)] // TODO: Pending review in #587
pub fn handle_create_npc(
    server: &mut Server,
    pos: Pos,
    stats: Stats,
    loadout: Loadout,
    body: Body,
    agent: impl Into<Option<Agent>>,
    alignment: Alignment,
    scale: Scale,
    drop_item: Option<Item>,
) {
    let group = match alignment {
        Alignment::Wild => None,
        Alignment::Enemy => Some(group::ENEMY),
        Alignment::Npc | Alignment::Tame => Some(group::NPC),
        // TODO: handle
        Alignment::Owned(_) => None,
    };

    let entity = server
        .state
        .create_npc(pos, stats, loadout, body)
        .with(scale)
        .with(alignment);

    let entity = if let Some(group) = group {
        entity.with(group)
    } else {
        entity
    };

    let entity = if let Some(agent) = agent.into() {
        entity.with(agent)
    } else {
        entity
    };

    let entity = if let Some(drop_item) = drop_item {
        entity.with(ItemDrop(drop_item))
    } else {
        entity
    };

    entity.build();
}

pub fn handle_shoot(
    server: &mut Server,
    entity: EcsEntity,
    dir: Dir,
    body: Body,
    light: Option<LightEmitter>,
    particles: Vec<ParticleEmitter>,
    projectile: Projectile,
    gravity: Option<Gravity>,
) {
    let state = server.state_mut();

    let mut pos = state
        .ecs()
        .read_storage::<Pos>()
        .get(entity)
        .expect("Failed to fetch entity")
        .0;

    // TODO: Player height
    pos.z += 1.2;

    let mut builder = state.create_projectile(Pos(pos), Vel(*dir * 100.0), body, projectile);
    if let Some(light) = light {
        builder = builder.with(light)
    }
    builder = builder.with(ParticleEmitters(particles));
    if let Some(gravity) = gravity {
        builder = builder.with(gravity)
    }

    builder.build();
}

pub fn handle_create_waypoint(server: &mut Server, pos: Vec3<f32>) {
    server
        .state
        .create_object(Pos(pos), comp::object::Body::CampfireLit)
        .with(LightEmitter {
            col: Rgb::new(1.0, 0.65, 0.2),
            strength: 2.0,
            flicker: 1.0,
            animated: true,
        })
        .with(ParticleEmitters::default())
        .with(WaypointArea::default())
        .build();
}
