use super::movement::ROLL_DURATION;
const HUMANOID_ACCEL: f32 = 50.0;
const HUMANOID_SPEED: f32 = 120.0;
use crate::{
    comp::{
        self, item, projectile, ActionState, ActionState::*, Body, CharacterState, ControlEvent,
        Controller, ControllerInputs, HealthChange, HealthSource, ItemKind, Mounting,
        MovementState, MovementState::*, Ori, PhysicsState, Pos, Projectile, RunData, StandData,
        Stats, Vel,
    },
    event::{Emitter, EventBus, LocalEvent, ServerEvent},
    state::DeltaTime,
};
use specs::{
    saveload::{Marker, MarkerAllocator},
    Entities, Entity, Join, Read, ReadStorage, System, WriteStorage,
};
use sphynx::{Uid, UidAllocator};
use std::time::Duration;
use vek::*;

/// # Character State System
/// #### Updates then detemrines next Character States based on ControllerInputs
pub struct Sys;

impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Read<'a, UidAllocator>,
        Read<'a, EventBus<ServerEvent>>,
        Read<'a, EventBus<LocalEvent>>,
        Read<'a, DeltaTime>,
        WriteStorage<'a, CharacterState>,
        WriteStorage<'a, Pos>,
        WriteStorage<'a, Vel>,
        WriteStorage<'a, Ori>,
        ReadStorage<'a, Controller>,
        ReadStorage<'a, Stats>,
        ReadStorage<'a, Body>,
        ReadStorage<'a, PhysicsState>,
        ReadStorage<'a, Uid>,
        ReadStorage<'a, Mounting>,
    );
    fn run(
        &mut self,
        (
            entities,
            uid_allocator,
            server_bus,
            local_bus,
            dt,
            mut character_states,
            mut positions,
            mut velocities,
            mut orientations,
            controllers,
            stats,
            bodies,
            physics_states,
            uids,
            mountings,
        ): Self::SystemData,
    ) {
        let mut server_emitter = server_bus.emitter();
        let mut local_emitter = local_bus.emitter();
        for (
            entity,
            uid,
            mut character,
            mut pos,
            mut vel,
            mut ori,
            controller,
            stats,
            body,
            physics,
            mount,
        ) in (
            &entities,
            &uids,
            &mut character_states,
            &mut positions,
            &mut velocities,
            &mut orientations,
            &controllers,
            &stats,
            &bodies,
            &physics_states,
            mountings.maybe(),
        )
            .join()
        {
            let inputs = &controller.inputs;

            // Returns a Wield action, or Idle if nothing to wield
            let try_wield = |stats: &Stats| -> ActionState {
                // Get weapon to wield
                if let Some(ItemKind::Tool { kind, .. }) =
                    stats.equipment.main.as_ref().map(|i| &i.kind)
                {
                    let wield_duration = kind.wield_duration();
                    Wield {
                        time_left: wield_duration,
                    }
                } else {
                    Idle
                }
            };

            let get_state_from_move_dir = |move_dir: &Vec2<f32>| -> MovementState {
                if move_dir.magnitude_squared() > 0.0 {
                    Run(RunData)
                } else {
                    Stand(StandData)
                }
            };

            // Being dead overrides all other states
            if stats.is_dead {
                // Only options: click respawn
                // prevent instant-respawns (i.e. player was holding attack)
                // by disallowing while input is held down
                if inputs.respawn.is_pressed() && !inputs.respawn.is_held_down() {
                    server_emitter.emit(ServerEvent::Respawn(entity));
                }
                // Or do nothing
                continue;
            }
            // If mounted, character state is controlled by mount
            if mount.is_some() {
                character.movement = Sit;
                continue;
            }

            // Update Action States
            match character.action {
                Attack {
                    ref mut time_left, ..
                } => {
                    *time_left = time_left
                        .checked_sub(Duration::from_secs_f32(dt.0))
                        .unwrap_or_default();
                }
                Roll {
                    ref mut time_left, ..
                } => {
                    *time_left = time_left
                        .checked_sub(Duration::from_secs_f32(dt.0))
                        .unwrap_or_default();
                }
                Charge { ref mut time_left } => {
                    *time_left = time_left
                        .checked_sub(Duration::from_secs_f32(dt.0))
                        .unwrap_or_default();
                }
                Wield { ref mut time_left } => {
                    *time_left = time_left
                        .checked_sub(Duration::from_secs_f32(dt.0))
                        .unwrap_or_default();
                }
                Block {
                    ref mut time_active,
                } => {
                    *time_active = time_active
                        .checked_add(Duration::from_secs_f32(dt.0))
                        .unwrap_or_default();
                }
                Idle => {}
            }

            // Determine new states
            match (character.action, character.movement) {
                // Jumping, one frame state that calls jump server event
                (_, Jump) => {
                    character.movement = Fall;
                    local_emitter.emit(LocalEvent::Jump(entity));
                }
                // Charging + Any Movement, prioritizes finishing charge
                // over movement states
                (Charge { time_left }, _) => {
                    if let Some(uid_b) = physics.touch_entity {
                        server_emitter.emit(ServerEvent::Damage {
                            uid: uid_b,
                            change: HealthChange {
                                amount: -20,
                                cause: HealthSource::Attack { by: *uid },
                            },
                        });

                        character.action = try_wield(stats);
                    } else if time_left == Duration::default() || vel.0.magnitude_squared() < 10.0 {
                        character.action = try_wield(stats);
                    }
                }
                // Rolling + Any Movement, prioritizes finishing charge
                // over movement states
                (
                    Roll {
                        time_left,
                        was_wielding,
                    },
                    _,
                ) => {
                    if time_left == Duration::default() {
                        if was_wielding {
                            character.action = try_wield(stats);
                        } else {
                            character.action = Idle;
                        }
                    }
                }
                // Any Action + Falling
                (action_state, Fall) => {
                    // character.movement = get_state_from_move_dir(&inputs.move_dir);
                    if inputs.glide.is_pressed() && !inputs.glide.is_held_down() {
                        character.movement = Glide;
                        continue;
                    }
                    // Reset to Falling while not standing on ground,
                    // otherwise keep the state given above
                    if !physics.on_ground {
                        if physics.in_fluid {
                            character.movement = Swim;
                        } else {
                            character.movement = Fall;
                        }
                    } else {
                        character.movement = Stand(StandData);
                        continue;
                    }

                    match action_state {
                        // Unwield if buttons pressed
                        Wield { .. } | Attack { .. } => {
                            if inputs.toggle_wield.is_just_pressed() {
                                character.action = Idle;
                            }
                        }
                        // Try to wield if any of buttons pressed
                        Idle => {
                            if inputs.primary.is_pressed() || inputs.secondary.is_pressed() {
                                character.action = try_wield(stats);
                            }
                        }
                        // Cancel blocks
                        Block { .. } => {
                            character.action = try_wield(stats);
                        }
                        // Don't change action
                        Charge { .. } | Roll { .. } => {}
                    }
                }
                // Any Action + Swimming
                (_, Swim) => {
                    character.movement = get_state_from_move_dir(&inputs.move_dir);

                    if !physics.on_ground && physics.in_fluid {
                        character.movement = Swim;
                    }
                    if inputs.primary.is_pressed() {
                        // TODO: PrimaryStart
                    } else if inputs.secondary.is_pressed() {
                        // TODO: SecondaryStart
                    }
                }
                // // Blocking, restricted look_dir compared to other states
                // (Block { .. }, Stand) | (Block { .. }, Run) => {
                //     character.movement = get_state_from_move_dir(&inputs.move_dir);

                //     if !inputs.secondary.is_pressed() {
                //         character.action = try_wield(stats);
                //     } else {
                //         // TODO: SecondaryStart
                //     }

                //     if !physics.on_ground && physics.in_fluid {
                //         character.movement = Swim;
                //     }
                // }
                // // Standing and Running states, typical states :shrug:
                // (action_state, Run) | (action_state, Stand) => {
                //     character.movement = get_state_from_move_dir(&inputs.move_dir);
                //     // Try to sit
                //     if inputs.sit.is_pressed() && physics.on_ground && body.is_humanoid() {
                //         character.movement = Sit;
                //         continue;
                //     }

                //     // Try to climb
                //     if let (true, Some(_wall_dir)) = (
                //         inputs.climb.is_pressed() | inputs.climb_down.is_pressed()
                //             && body.is_humanoid(),
                //         physics.on_wall,
                //     ) {
                //         character.movement = Climb;
                //         continue;
                //     }

                //     // Try to swim
                //     if !physics.on_ground && physics.in_fluid {
                //         character.movement = Swim;
                //         continue;
                //     }

                //     // While on ground ...
                //     if physics.on_ground {
                //         // Try to jump
                //         if inputs.jump.is_pressed() && !inputs.jump.is_held_down() {
                //             character.movement = Jump;
                //             continue;
                //         }

                //         // Try to charge
                //         if inputs.charge.is_pressed() && !inputs.charge.is_held_down() {
                //             character.action = Charge {
                //                 time_left: Duration::from_millis(250),
                //             };
                //             continue;
                //         }

                //         // Try to roll
                //         if character.movement == Run
                //             && inputs.roll.is_pressed()
                //             && body.is_humanoid()
                //         {
                //             character.action = Roll {
                //                 time_left: ROLL_DURATION,
                //                 was_wielding: character.action.is_wield(),
                //             };
                //             continue;
                //         }
                //     }
                //     // While not on ground ...
                //     else {
                //         // Try to glide
                //         if physics.on_wall == None
                //             && inputs.glide.is_pressed()
                //             && !inputs.glide.is_held_down()
                //             && body.is_humanoid()
                //         {
                //             character.movement = Glide;
                //             continue;
                //         }
                //         character.movement = Fall;
                //     }

                //     // Tool Actions
                //     if inputs.toggle_wield.is_just_pressed() {
                //         match action_state {
                //             Wield { .. } | Attack { .. } => {
                //                 // Prevent instantaneous reequipping by checking
                //                 // for done wielding
                //                 if character.action.is_action_finished() {
                //                     character.action = Idle;
                //                 }
                //                 continue;
                //             }
                //             Idle => {
                //                 character.action = try_wield(stats);
                //                 continue;
                //             }
                //             Charge { .. } | Roll { .. } | Block { .. } => {}
                //         }
                //     }
                //     if inputs.primary.is_pressed() {
                //         // TODO: PrimaryStart
                //     } else if inputs.secondary.is_pressed() {
                //         // TODO: SecondaryStart
                //     }
                // }
                // Sitting
                (_, Sit) => {
                    character.action = Idle;
                    character.movement = get_state_from_move_dir(&inputs.move_dir);

                    // character.movement will be Stand after updating when
                    // no movement has occurred
                    if character.movement == Stand(StandData) {
                        character.movement = Sit;
                    }
                    if inputs.jump.is_pressed() && !inputs.jump.is_held_down() {
                        character.movement = Jump;
                        continue;
                    }
                    if !physics.on_ground {
                        character.movement = Fall;
                    }
                }
                // Any Action + Gliding, shouldnt care about action,
                // because should be Idle
                (_, Glide) => {
                    character.action = Idle;

                    if !inputs.glide.is_pressed() {
                        character.movement = Fall;
                    } else if let Some(_wall_dir) = physics.on_wall {
                        character.movement = Fall;
                    }

                    if physics.on_ground {
                        character.movement = Stand(StandData)
                    }
                }
                // Any Action + Climbing, shouldnt care about action,
                // because should be Idle
                (_, Climb) => {
                    character.action = Idle;
                    if let None = physics.on_wall {
                        if inputs.jump.is_pressed() {
                            character.movement = Jump;
                        } else {
                            character.movement = Fall;
                        }
                    }
                    if physics.on_ground {
                        character.movement = Stand(StandData);
                    }
                }
                (_, Stand(data)) => {
                    let mut new_char =
                        data.handle(character, pos, vel, ori, &dt, inputs, stats, body, physics);
                    println!("{:?}", new_char);
                    character = &mut new_char;
                }
                (_, _) => {
                    character.movement = Stand(StandData);
                }
            };
        }
    }
}

pub trait State {
    fn handle(
        &self,
        character: &CharacterState,
        pos: &mut Pos,
        vel: &mut Vel,
        ori: &mut Ori,
        dt: &DeltaTime,
        inputs: &ControllerInputs,
        stats: &Stats,
        body: &Body,
        physics: &PhysicsState,
    ) -> CharacterState;
}

impl State for StandData {
    fn handle(
        &self,
        character: &CharacterState,
        pos: &mut Pos,
        vel: &mut Vel,
        ori: &mut Ori,
        dt: &DeltaTime,
        inputs: &ControllerInputs,
        stats: &Stats,
        body: &Body,
        physics: &PhysicsState,
    ) -> CharacterState {
        // Move player according to move_dir
        vel.0 += Vec2::broadcast(dt.0)
            * inputs.move_dir
            * if vel.0.magnitude_squared() < HUMANOID_SPEED.powf(2.0) {
                HUMANOID_ACCEL
            } else {
                0.0
            };

        // Set direction based on move direction when on the ground
        let ori_dir = if character.action.is_attack() || character.action.is_block() {
            Vec2::from(inputs.look_dir).normalized()
        } else {
            Vec2::from(vel.0)
        };

        if ori_dir.magnitude_squared() > 0.0001
            && (ori.0.normalized() - Vec3::from(ori_dir).normalized()).magnitude_squared() > 0.001
        {
            ori.0 = vek::ops::Slerp::slerp(ori.0, ori_dir.into(), 9.0 * dt.0);
        }

        // Try to sit
        if inputs.sit.is_pressed() && physics.on_ground && body.is_humanoid() {
            return CharacterState {
                movement: Sit,
                action: Idle,
            };
        }

        // Try to climb
        if let (true, Some(_wall_dir)) = (
            inputs.climb.is_pressed() | inputs.climb_down.is_pressed() && body.is_humanoid(),
            physics.on_wall,
        ) {
            return CharacterState {
                movement: Climb,
                action: Idle,
            };
        }

        // Try to swim
        if !physics.on_ground && physics.in_fluid {
            return CharacterState {
                action: character.action,
                movement: Swim,
            };
        }

        // While on ground ...
        if physics.on_ground {
            // Try to jump
            if inputs.jump.is_pressed() && !inputs.jump.is_held_down() {
                return CharacterState {
                    action: character.action,
                    movement: Jump,
                };
            }

            // Try to charge
            if inputs.charge.is_pressed() && !inputs.charge.is_held_down() {
                return CharacterState {
                    action: Charge {
                        time_left: Duration::from_millis(250),
                    },
                    movement: Run(RunData),
                };
            }

            // Try to roll
            if inputs.roll.is_pressed() && body.is_humanoid() {
                return CharacterState {
                    action: Roll {
                        time_left: Duration::from_millis(600),
                        was_wielding: character.action.is_wield(),
                    },
                    movement: Run(RunData),
                };
            }
        }
        // While not on ground ...
        else {
            // Try to glide
            if physics.on_wall == None
                && inputs.glide.is_pressed()
                && !inputs.glide.is_held_down()
                && body.is_humanoid()
            {
                return CharacterState {
                    action: Idle,
                    movement: Glide,
                };
            }
            return CharacterState {
                action: character.action,
                movement: Fall,
            };
        }

        // Tool Actions
        if inputs.toggle_wield.is_just_pressed() {
            match character.action {
                Wield { .. } | Attack { .. } => {
                    // Prevent instantaneous reequipping by checking
                    // for done wielding
                    if character.action.is_action_finished() {
                        return CharacterState {
                            action: Idle,
                            movement: character.movement,
                        };
                    }
                }
                Idle => {
                    return CharacterState {
                        // Try to wield if an item is equipped in main hand
                        action: if let Some(ItemKind::Tool { kind, .. }) =
                            stats.equipment.main.as_ref().map(|i| &i.kind)
                        {
                            let wield_duration = kind.wield_duration();
                            Wield {
                                time_left: wield_duration,
                            }
                        } else {
                            Idle
                        },
                        movement: character.movement,
                    };
                }
                Charge { .. } | Roll { .. } | Block { .. } => {}
            }
        }
        if inputs.primary.is_pressed() {
            // TODO: PrimaryStart
        } else if inputs.secondary.is_pressed() {
            // TODO: SecondaryStart
        }

        if inputs.move_dir.magnitude_squared() > 0.0 {
            return CharacterState {
                action: character.action,
                movement: Run(RunData),
            };
        } else {
            return CharacterState {
                action: character.action,
                movement: Stand(StandData),
            };
        }
    }
}
