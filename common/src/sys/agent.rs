use crate::terrain::TerrainGrid;
use crate::{
    comp::{
        self, Agent, Alignment, CharacterState, Controller, MountState, MovementState::Glide, Pos,
        Stats,
    },
    sync::{Uid, UidAllocator},
};
use rand::{seq::SliceRandom, thread_rng, Rng};
use specs::{
    saveload::{Marker, MarkerAllocator},
    Entities, Join, Read, ReadExpect, ReadStorage, System, WriteStorage,
};
use vek::*;

/// This system will allow NPCs to modify their controller
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Read<'a, UidAllocator>,
        Entities<'a>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Stats>,
        ReadStorage<'a, CharacterState>,
        ReadExpect<'a, TerrainGrid>,
        ReadStorage<'a, Alignment>,
        WriteStorage<'a, Agent>,
        WriteStorage<'a, Controller>,
        ReadStorage<'a, MountState>,
    );

    fn run(
        &mut self,
        (
            uid_allocator,
            entities,
            positions,
            stats,
            character_states,
            terrain,
            alignments,
            mut agents,
            mut controllers,
            mount_states,
        ): Self::SystemData,
    ) {
        for (entity, pos, alignment, agent, controller, mount_state) in (
            &entities,
            &positions,
            alignments.maybe(),
            &mut agents,
            &mut controllers,
            mount_states.maybe(),
        )
            .join()
        {
            // Skip mounted entities
            if mount_state
                .map(|ms| {
                    if let MountState::Unmounted = ms {
                        false
                    } else {
                        true
                    }
                })
                .unwrap_or(false)
            {
                continue;
            }

            controller.reset();

            let mut inputs = &mut controller.inputs;

            const PET_DIST: f32 = 12.0;
            const PATROL_DIST: f32 = 48.0;
            const SIGHT_DIST: f32 = 18.0;
            const MIN_ATTACK_DIST: f32 = 3.25;

            let mut chase_tgt = None;
            let mut choose_target = false;

            if let Some(target) = agent.target {
                // Chase / attack target
                if let (Some(tgt_pos), stats) = (positions.get(target), stats.get(target)) {
                    if stats.map(|s| s.is_dead).unwrap_or(false) {
                        // Don't target dead entities
                        choose_target = true;
                    } else if pos.0.distance(tgt_pos.0) < SIGHT_DIST {
                        chase_tgt = Some((tgt_pos.0, 1.0, true))
                    } else {
                        // Lose sight of enemies
                        choose_target = true;
                    }
                } else {
                    choose_target = true;
                }
            } else if let Some(owner) = agent.owner {
                if let Some(tgt_pos) = positions.get(owner) {
                    if pos.0.distance(tgt_pos.0) > PET_DIST || agent.target.is_none() {
                        // Follow owner
                        chase_tgt = Some((tgt_pos.0, 6.0, false));
                    } else {
                        choose_target = thread_rng().gen::<f32>() < 0.02;
                    }
                } else {
                    agent.owner = None;
                }
            } else if let Some(patrol_origin) = agent.patrol_origin {
                if pos.0.distance(patrol_origin) < PATROL_DIST {
                    // Return to patrol origin
                    chase_tgt = Some((patrol_origin, 64.0, false));
                }
            } else {
                choose_target = thread_rng().gen::<f32>() < 0.05;
            }

            // Attack a target that's attacking us
            if let Some(stats) = stats.get(entity) {
                match stats.health.last_change.1.cause {
                    comp::HealthSource::Attack { by } => {
                        if agent.target.is_none() {
                            agent.target = uid_allocator.retrieve_entity_internal(by.id());
                        } else if thread_rng().gen::<f32>() < 0.005 {
                            agent.target = uid_allocator.retrieve_entity_internal(by.id());
                        }
                    }
                    _ => {}
                }
            }

            if choose_target {
                // Search for new targets
                let entities = (&entities, &positions, &stats, alignments.maybe())
                    .join()
                    .filter(|(e, e_pos, e_stats, e_alignment)| {
                        (e_pos.0 - pos.0).magnitude() < SIGHT_DIST
                            && *e != entity
                            && !e_stats.is_dead
                            && alignment
                                .and_then(|a| e_alignment.map(|b| a.hostile_towards(*b)))
                                .unwrap_or(false)
                    })
                    .map(|(e, _, _, _)| e)
                    .collect::<Vec<_>>();

                agent.target = (&entities).choose(&mut thread_rng()).cloned();
            }

            // Chase target
            if let Some((tgt_pos, min_dist, aggressive)) = chase_tgt {
                if let Some(bearing) = agent.chaser.chase(&*terrain, pos.0, tgt_pos, min_dist) {
                    inputs.move_dir = Vec2::from(bearing).try_normalized().unwrap_or(Vec2::zero());
                    inputs.jump.set_state(bearing.z > 1.0);
                }

                if pos.0.distance(tgt_pos) < MIN_ATTACK_DIST && aggressive {
                    inputs.look_dir = tgt_pos - pos.0;
                    inputs.move_dir = Vec2::from(tgt_pos - pos.0)
                        .try_normalized()
                        .unwrap_or(Vec2::zero())
                        * 0.01;
                    inputs.primary.set_state(true);
                }
            }

            /*
            match agent {
                Agent::Wanderer(bearing) => {
                    *bearing += Vec2::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5)
                        * 0.1
                        - *bearing * 0.01
                        - pos.0 * 0.0002;

                    if bearing.magnitude_squared() > 0.001 {
                        inputs.move_dir = bearing.normalized();
                    }
                }
                Agent::Pet { target, chaser } => {
                    // Run towards target.
                    if let Some(tgt_pos) = positions.get(*target) {
                        if let Some(bearing) = chaser.chase(&*terrain, pos.0, tgt_pos.0, 5.0) {
                            inputs.move_dir =
                                Vec2::from(bearing).try_normalized().unwrap_or(Vec2::zero());
                            inputs.jump.set_state(bearing.z > 1.0);
                        }
                    } else {
                        inputs.move_dir = Vec2::zero();
                    }
                }
                Agent::Enemy { bearing, target } => {
                    const SIGHT_DIST: f32 = 18.0;
                    const MIN_ATTACK_DIST: f32 = 3.25;
                    let mut choose_new = false;

                    if let Some((Some(target_pos), Some(target_stats), Some(target_character))) =
                        target.map(|target| {
                            (
                                positions.get(target),
                                stats.get(target),
                                character_states.get(target),
                            )
                        })
                    {
                        inputs.look_dir = target_pos.0 - pos.0;

                        let dist = Vec2::<f32>::from(target_pos.0 - pos.0).magnitude();
                        if target_stats.is_dead {
                            choose_new = true;
                        } else if dist < 0.001 {
                            // Probably can only happen when entities are at a different z-level
                            // since at the same level repulsion would keep them apart.
                            // Distinct from the first if block since we may want to change the
                            // behavior for this case.
                            choose_new = true;
                        } else if dist < MIN_ATTACK_DIST {
                            // Fight (and slowly move closer)
                            inputs.move_dir =
                                Vec2::<f32>::from(target_pos.0 - pos.0).normalized() * 0.01;
                            inputs.primary.set_state(true);
                        } else if dist < SIGHT_DIST {
                            inputs.move_dir =
                                Vec2::<f32>::from(target_pos.0 - pos.0).normalized() * 0.96;

                            if rand::random::<f32>() < 0.02 {
                                inputs.roll.set_state(true);
                            }

                            if target_character.movement == Glide && target_pos.0.z > pos.0.z + 5.0
                            {
                                inputs.glide.set_state(true);
                                inputs.jump.set_state(true);
                            }
                        } else {
                            choose_new = true;
                        }
                    } else {
                        *bearing +=
                            Vec2::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5)
                                * 0.1
                                - *bearing * 0.005;

                        inputs.move_dir = if bearing.magnitude_squared() > 0.001 {
                            bearing.normalized()
                        } else {
                            Vec2::zero()
                        };

                        choose_new = true;
                    }

                    if choose_new && rand::random::<f32>() < 0.1 {
                        let entities = (&entities, &positions, &stats)
                            .join()
                            .filter(|(e, e_pos, e_stats)| {
                                (e_pos.0 - pos.0).magnitude() < SIGHT_DIST
                                    && *e != entity
                                    && !e_stats.is_dead
                            })
                            .map(|(e, _, _)| e)
                            .collect::<Vec<_>>();

                        let mut rng = thread_rng();
                        *target = (&entities).choose(&mut rng).cloned();
                    }
                }
            }
            */

            debug_assert!(inputs.move_dir.map(|e| !e.is_nan()).reduce_and());
            debug_assert!(inputs.look_dir.map(|e| !e.is_nan()).reduce_and());
        }
    }
}
