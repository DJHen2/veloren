use crate::{
    comp::{
        ActionState::*, CharacterState, Controller, HealthChange, HealthSource, ItemKind, Ori, Pos,
        Stats,
    },
    event::{EventBus, LocalEvent, ServerEvent},
    state::{DeltaTime, Uid},
};
use specs::{Entities, Join, Read, ReadStorage, System, WriteStorage};
use std::time::Duration;
use vek::*;

pub const WIELD_DURATION: Duration = Duration::from_millis(300);
pub const ATTACK_DURATION: Duration = Duration::from_millis(500);

// Delay before hit
const PREPARE_DURATION: Duration = Duration::from_millis(100);

const BLOCK_EFFICIENCY: f32 = 0.9;

const ATTACK_RANGE: f32 = 4.0;
const BLOCK_ANGLE: f32 = 180.0;

/// This system is responsible for handling accepted inputs like moving or attacking
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Read<'a, EventBus<ServerEvent>>,
        Read<'a, EventBus<LocalEvent>>,
        Read<'a, DeltaTime>,
        ReadStorage<'a, Uid>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Ori>,
        ReadStorage<'a, Controller>,
        WriteStorage<'a, CharacterState>,
        WriteStorage<'a, Stats>,
    );

    fn run(
        &mut self,
        (
            entities,
            server_bus,
            local_bus,
            dt,
            uids,
            positions,
            orientations,
            controllers,
            mut character_states,
            stats,
        ): Self::SystemData,
    ) {
        let mut server_emitter = server_bus.emitter();
        let mut _local_emitter = local_bus.emitter();

        // Attacks
        for (entity, uid, pos, ori, _, stat) in (
            &entities,
            &uids,
            &positions,
            &orientations,
            &controllers,
            &stats,
        )
            .join()
        {
            let (deal_damage, should_end) = if let Some(Attack { time_left, applied }) =
                &mut character_states.get_mut(entity).map(|c| &mut c.action)
            {
                *time_left = time_left
                    .checked_sub(Duration::from_secs_f32(dt.0))
                    .unwrap_or_default();
                if !*applied && ATTACK_DURATION - *time_left > PREPARE_DURATION {
                    *applied = true;
                    (true, false)
                } else if *time_left == Duration::default() {
                    (false, true)
                } else {
                    (false, false)
                }
            } else {
                (false, false)
            };

            if deal_damage {
                if let Some(Attack { .. }) = &character_states.get(entity).map(|c| c.action) {
                    // Go through all other entities
                    for (b, uid_b, pos_b, ori_b, character_b, stat_b) in (
                        &entities,
                        &uids,
                        &positions,
                        &orientations,
                        &character_states,
                        &stats,
                    )
                        .join()
                    {
                        // 2D versions
                        let pos2 = Vec2::from(pos.0);
                        let pos_b2: Vec2<f32> = Vec2::from(pos_b.0);
                        let ori2 = Vec2::from(ori.0);

                        // Check if it is a hit
                        if entity != b
                            && !stat_b.is_dead
                            && pos.0.distance_squared(pos_b.0) < ATTACK_RANGE.powi(2)
                            // TODO: Use size instead of 1.0
                            && ori2.angle_between(pos_b2 - pos2) < (1.0 / pos2.distance(pos_b2)).atan()
                        {
                            // Weapon gives base damage
                            let mut dmg = if let Some(ItemKind::Tool { power, .. }) =
                                stat.equipment.main.as_ref().map(|i| &i.kind)
                            {
                                *power as i32
                            } else {
                                1
                            };

                            // Block
                            if character_b.action.is_block()
                                && ori_b.0.angle_between(pos.0 - pos_b.0).to_degrees()
                                    < BLOCK_ANGLE / 2.0
                            {
                                dmg = (dmg as f32 * (1.0 - BLOCK_EFFICIENCY)) as i32
                            }

                            server_emitter.emit(ServerEvent::Damage {
                                uid: *uid_b,
                                change: HealthChange {
                                    amount: -dmg,
                                    cause: HealthSource::Attack { by: *uid },
                                },
                            });
                        }
                    }
                }
            }

            if should_end {
                if let Some(character) = &mut character_states.get_mut(entity) {
                    character.action = Wield {
                        time_left: Duration::default(),
                    };
                }
            }

            if let Some(Wield { time_left }) =
                &mut character_states.get_mut(entity).map(|c| &mut c.action)
            {
                if *time_left != Duration::default() {
                    *time_left = time_left
                        .checked_sub(Duration::from_secs_f32(dt.0))
                        .unwrap_or_default();
                }
            }
        }
    }
}
