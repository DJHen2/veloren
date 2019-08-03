use crate::comp::{Agent, Controller, Pos, Stats};
use rand::{seq::SliceRandom, thread_rng};
use specs::{Entities, Join, ReadStorage, System, WriteStorage};
use vek::*;

/// This system will allow NPCs to modify their controller
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Pos>,
        ReadStorage<'a, Stats>,
        WriteStorage<'a, Agent>,
        WriteStorage<'a, Controller>,
    );

    fn run(&mut self, (entities, positions, stats, mut agents, mut controllers): Self::SystemData) {
        for (entity, pos, agent, controller) in
            (&entities, &positions, &mut agents, &mut controllers).join()
        {
            match agent {
                Agent::Wanderer(bearing) => {
                    *bearing += Vec2::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5)
                        * 0.1
                        - *bearing * 0.01
                        - pos.0 * 0.0002;

                    if bearing.magnitude_squared() != 0.0 {
                        controller.move_dir = bearing.normalized();
                    }
                }
                Agent::Pet { target, offset } => {
                    // Run towards target.
                    match positions.get(*target) {
                        Some(tgt_pos) => {
                            let tgt_pos = tgt_pos.0 + *offset;

                            if tgt_pos.z > pos.0.z + 1.0 {
                                controller.jump = true;
                            }

                            // Move towards the target.
                            let dist: f32 = Vec2::from(tgt_pos - pos.0).magnitude();
                            controller.move_dir = if dist > 5.0 {
                                Vec2::from(tgt_pos - pos.0).normalized()
                            } else if dist < 1.5 && dist > 0.0 {
                                Vec2::from(pos.0 - tgt_pos).normalized()
                            } else {
                                Vec2::zero()
                            };
                        }
                        _ => controller.move_dir = Vec2::zero(),
                    }

                    // Change offset occasionally.
                    if rand::random::<f32>() < 0.003 {
                        *offset =
                            Vec2::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5)
                                * 10.0;
                    }
                }
                Agent::Enemy { bearing, target } => {
                    const SIGHT_DIST: f32 = 30.0;
                    let mut choose_new = false;

                    if let Some((Some(target_pos), Some(target_stats))) =
                        target.map(|target| (positions.get(target), stats.get(target)))
                    {
                        let dist = Vec2::<f32>::from(target_pos.0 - pos.0).magnitude();
                        if target_stats.is_dead {
                            choose_new = true;
                        } else if dist < 3.0 {
                            // Get more distance
                            controller.move_dir =
                                Vec2::<f32>::from(target_pos.0 - pos.0).normalized() * -0.96;
                        } else if dist < 4.0 {
                            // Fight and slowly move closer
                            controller.move_dir =
                                Vec2::<f32>::from(target_pos.0 - pos.0).normalized() * 0.01;

                            if rand::random::<f32>() < 0.1 {
                                controller.attack = true;
                            } else {
                                controller.attack = false;
                            }
                        } else if dist < SIGHT_DIST {
                            controller.move_dir =
                                Vec2::<f32>::from(target_pos.0 - pos.0).normalized() * 0.96;
                        } else {
                            choose_new = true;
                        }
                    } else {
                        *bearing +=
                            Vec2::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5)
                                * 0.1
                                - *bearing * 0.005;

                        controller.move_dir = if bearing.magnitude_squared() > 0.1 {
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
                                Vec2::<f32>::from(e_pos.0 - pos.0).magnitude() < SIGHT_DIST
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
        }
    }
}
