use crate::{client::Client, Server, SpawnPoint, StateExt};
use common::{
    assets,
    comp::{self, object, Body, HealthChange, HealthSource, Item, Player, Stats},
    msg::ServerMsg,
    state::BlockChange,
    sync::{Uid, WorldSyncExt},
    sys::combat::{BLOCK_ANGLE, BLOCK_EFFICIENCY},
    terrain::{Block, TerrainGrid},
    vol::{ReadVol, Vox},
};
use log::error;
use specs::{join::Join, Entity as EcsEntity, WorldExt};
use vek::Vec3;

pub fn handle_damage(server: &Server, uid: Uid, change: HealthChange) {
    let state = &server.state;
    let ecs = state.ecs();
    if let Some(entity) = ecs.entity_from_uid(uid.into()) {
        if let Some(stats) = ecs.write_storage::<Stats>().get_mut(entity) {
            stats.health.change_by(change);
        }
    }
}

pub fn handle_destroy(server: &mut Server, entity: EcsEntity, cause: HealthSource) {
    let state = server.state_mut();

    // Chat message
    if let Some(player) = state.ecs().read_storage::<Player>().get(entity) {
        let msg = if let HealthSource::Attack { by }
        | HealthSource::Projectile { owner: Some(by) } = cause
        {
            state.ecs().entity_from_uid(by.into()).and_then(|attacker| {
                state
                    .ecs()
                    .read_storage::<Player>()
                    .get(attacker)
                    .map(|attacker_alias| {
                        format!("{} was killed by {}", &player.alias, &attacker_alias.alias)
                    })
            })
        } else {
            None
        }
        .unwrap_or(format!("{} died", &player.alias));

        state.notify_registered_clients(ServerMsg::kill(msg));
    }

    {
        // Give EXP to the killer if entity had stats
        let mut stats = state.ecs().write_storage::<Stats>();
        if let Some(entity_stats) = stats.get(entity).cloned() {
            if let HealthSource::Attack { by } | HealthSource::Projectile { owner: Some(by) } =
                cause
            {
                state.ecs().entity_from_uid(by.into()).map(|attacker| {
                    if let Some(attacker_stats) = stats.get_mut(attacker) {
                        // TODO: Discuss whether we should give EXP by Player
                        // Killing or not.
                        attacker_stats
                            .exp
                            .change_by((entity_stats.level.level() * 10) as i64);
                    }
                });
            }
        }
    }

    if state
        .ecs()
        .write_storage::<Client>()
        .get_mut(entity)
        .is_some()
    {
        state
            .ecs()
            .write_storage()
            .insert(entity, comp::Vel(Vec3::zero()))
            .err()
            .map(|err| error!("Failed to set zero vel on dead client: {:?}", err));
        state
            .ecs()
            .write_storage()
            .insert(entity, comp::ForceUpdate)
            .err()
            .map(|err| error!("Failed to insert ForceUpdate on dead client: {:?}", err));
        state
            .ecs()
            .write_storage::<comp::LightEmitter>()
            .remove(entity);
        state
            .ecs()
            .write_storage::<comp::Energy>()
            .get_mut(entity)
            .map(|energy| energy.set_to(energy.maximum(), comp::EnergySource::Revive));
        let _ = state
            .ecs()
            .write_storage::<comp::CharacterState>()
            .insert(entity, comp::CharacterState::default());
    } else {
        if state.ecs().read_storage::<comp::Agent>().contains(entity) {
            // Replace npc with loot
            let _ = state
                .ecs()
                .write_storage()
                .insert(entity, Body::Object(object::Body::Pouch));

            let mut item_drops = state.ecs().write_storage::<comp::ItemDrop>();
            let item = if let Some(item_drop) = item_drops.get(entity).cloned() {
                item_drops.remove(entity);
                item_drop.0
            } else {
                assets::load_expect_cloned::<Item>("common.items.cheese")
            };

            let _ = state.ecs().write_storage().insert(entity, item);

            state.ecs().write_storage::<comp::Stats>().remove(entity);
            state.ecs().write_storage::<comp::Agent>().remove(entity);
            state
                .ecs()
                .write_storage::<comp::LightEmitter>()
                .remove(entity);
            state
                .ecs()
                .write_storage::<comp::CharacterState>()
                .remove(entity);
            state
                .ecs()
                .write_storage::<comp::Controller>()
                .remove(entity);
        } else {
            if let Err(err) = state.delete_entity_recorded(entity) {
                error!("Failed to delete destroyed entity: {:?}", err);
            }
        }

        // TODO: Add Delete(time_left: Duration) component
        /*
        // If not a player delete the entity
        if let Err(err) = state.delete_entity_recorded(entity) {
            error!("Failed to delete destroyed entity: {:?}", err);
        }
        */
    }
}

pub fn handle_land_on_ground(server: &Server, entity: EcsEntity, vel: Vec3<f32>) {
    let state = &server.state;
    if vel.z <= -30.0 {
        if let Some(stats) = state.ecs().write_storage::<comp::Stats>().get_mut(entity) {
            let falldmg = vel.z.powi(2) as i32 / 20 - 40;
            stats.health.change_by(comp::HealthChange {
                amount: -falldmg,
                cause: comp::HealthSource::World,
            });
        }
    }
}

pub fn handle_respawn(server: &Server, entity: EcsEntity) {
    let state = &server.state;

    // Only clients can respawn
    if state
        .ecs()
        .write_storage::<Client>()
        .get_mut(entity)
        .is_some()
    {
        let respawn_point = state
            .read_component_cloned::<comp::Waypoint>(entity)
            .map(|wp| wp.get_pos())
            .unwrap_or(state.ecs().read_resource::<SpawnPoint>().0);

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
        state
            .ecs()
            .write_storage()
            .insert(entity, comp::ForceUpdate)
            .err()
            .map(|err| {
                error!(
                    "Error inserting ForceUpdate component when respawning client: {:?}",
                    err
                )
            });
    }
}

pub fn handle_explosion(server: &Server, pos: Vec3<f32>, power: f32, owner: Option<Uid>) {
    // Go through all other entities
    let hit_range = 3.0 * power;
    let ecs = &server.state.ecs();
    for (pos_b, ori_b, character_b, stats_b) in (
        &ecs.read_storage::<comp::Pos>(),
        &ecs.read_storage::<comp::Ori>(),
        &ecs.read_storage::<comp::CharacterState>(),
        &mut ecs.write_storage::<comp::Stats>(),
    )
        .join()
    {
        let distance_squared = pos.distance_squared(pos_b.0);
        // Check if it is a hit
        if !stats_b.is_dead
            // Spherical wedge shaped attack field
            // RADIUS
            && distance_squared < hit_range.powi(2)
        {
            // Weapon gives base damage
            let mut dmg = ((1.0 - distance_squared / hit_range.powi(2)) * power * 10.0) as u32;

            if rand::random() {
                dmg += 1;
            }

            // Block
            if character_b.is_block()
                && ori_b.0.angle_between(pos - pos_b.0) < BLOCK_ANGLE.to_radians() / 2.0
            {
                dmg = (dmg as f32 * (1.0 - BLOCK_EFFICIENCY)) as u32
            }

            stats_b.health.change_by(HealthChange {
                amount: -(dmg as i32),
                cause: HealthSource::Projectile { owner },
            });
        }
    }

    const RAYS: usize = 500;

    // Color terrain
    let mut touched_blocks = Vec::new();
    let color_range = power * 2.7;
    for _ in 0..RAYS {
        let dir = Vec3::new(
            rand::random::<f32>() - 0.5,
            rand::random::<f32>() - 0.5,
            rand::random::<f32>() - 0.5,
        )
        .normalized();

        let _ = ecs
            .read_resource::<TerrainGrid>()
            .ray(pos, pos + dir * color_range)
            .until(|_| rand::random::<f32>() < 0.05)
            .for_each(|pos| touched_blocks.push(pos))
            .cast();
    }

    let terrain = ecs.read_resource::<TerrainGrid>();
    let mut block_change = ecs.write_resource::<BlockChange>();
    for block_pos in touched_blocks {
        if let Ok(block) = terrain.get(block_pos) {
            let diff2 = block_pos.map(|b| b as f32).distance_squared(pos);
            let fade = (1.0 - diff2 / color_range.powi(2)).max(0.0);
            if let Some(mut color) = block.get_color() {
                let r = color[0] as f32 + (fade * (color[0] as f32 * 0.5 - color[0] as f32));
                let g = color[1] as f32 + (fade * (color[1] as f32 * 0.3 - color[1] as f32));
                let b = color[2] as f32 + (fade * (color[2] as f32 * 0.3 - color[2] as f32));
                color[0] = r as u8;
                color[1] = g as u8;
                color[2] = b as u8;
                block_change.set(block_pos, Block::new(block.kind(), color));
            }
        }
    }

    // Destroy terrain
    for _ in 0..RAYS {
        let dir = Vec3::new(
            rand::random::<f32>() - 0.5,
            rand::random::<f32>() - 0.5,
            rand::random::<f32>() - 0.15,
        )
        .normalized();

        let _ = ecs
            .read_resource::<TerrainGrid>()
            .ray(pos, pos + dir * power)
            .until(|_| rand::random::<f32>() < 0.05)
            .for_each(|pos| block_change.set(pos, Block::empty()))
            .cast();
    }
}
