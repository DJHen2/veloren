use crate::{
    comp::{
        ActionState::*, Body, CharacterState, Controller, MovementState::*, PhysicsState, Stats,
        Vel,
    },
    event::{EventBus, LocalEvent, ServerEvent},
};
use specs::{Entities, Join, Read, ReadStorage, System, WriteStorage};
use std::time::Duration;

/// This system is responsible for validating controller inputs
pub struct Sys;
impl<'a> System<'a> for Sys {
    type SystemData = (
        Entities<'a>,
        Read<'a, EventBus<ServerEvent>>,
        Read<'a, EventBus<LocalEvent>>,
        WriteStorage<'a, Controller>,
        ReadStorage<'a, Stats>,
        ReadStorage<'a, Body>,
        ReadStorage<'a, Vel>,
        ReadStorage<'a, PhysicsState>,
        WriteStorage<'a, CharacterState>,
    );

    fn run(
        &mut self,
        (
            entities,
            server_bus,
            local_bus,
            mut controllers,
            stats,
            bodies,
            velocities,
            physics_states,
            mut character_states,
        ): Self::SystemData,
    ) {
        let mut server_emitter = server_bus.emitter();
        let mut local_emitter = local_bus.emitter();

        for (entity, controller, stats, body, vel, physics, mut character) in (
            &entities,
            &mut controllers,
            &stats,
            &bodies,
            &velocities,
            &physics_states,
            &mut character_states,
        )
            .join()
        {
            if stats.is_dead {
                // Respawn
                if controller.respawn {
                    server_emitter.emit(ServerEvent::Respawn(entity));
                }
                continue;
            }

            // Move
            controller.move_dir = if controller.move_dir.magnitude_squared() > 1.0 {
                controller.move_dir.normalized()
            } else {
                controller.move_dir
            };

            if character.movement == Stand && controller.move_dir.magnitude_squared() > 0.0 {
                character.movement = Run;
            } else if character.movement == Run && controller.move_dir.magnitude_squared() == 0.0 {
                character.movement = Stand;
            }

            // Look
            controller.look_dir = controller
                .look_dir
                .try_normalized()
                .unwrap_or(controller.move_dir.into());

            // Glide
            if controller.glide
                && !physics.on_ground
                && (character.action == Idle || character.action.is_wield())
                && character.movement == Jump
                // TODO: Ask zesterer if we can remove this
                && body.is_humanoid()
            {
                character.movement = Glide;
            } else if !controller.glide && character.movement == Glide {
                character.movement = Jump;
            }

            // Wield
            if controller.attack
                && character.action == Idle
                && (character.movement == Stand || character.movement == Run)
            {
                character.action = Wield {
                    time_left: Duration::from_millis(300),
                };
            }

            // Attack
            if controller.attack
                && (character.movement == Stand
                    || character.movement == Run
                    || character.movement == Jump)
            {
                // TODO: Check if wield ability exists
                if let Wield { time_left } = character.action {
                    if time_left == Duration::default() {
                        character.action = Attack {
                            time_left: Duration::from_millis(300),
                            applied: false,
                        };
                    }
                }
            }

            // Block
            if controller.block
                && (character.movement == Stand || character.movement == Run)
                && (character.action == Idle || character.action.is_wield())
            {
                character.action = Block {
                    time_left: Duration::from_secs(5),
                };
            } else if !controller.block && character.action.is_block() {
                character.action = Idle;
            }

            // Roll
            if controller.roll
                && (character.action == Idle || character.action.is_wield())
                && character.movement == Run
                && physics.on_ground
            {
                character.movement = Roll {
                    time_left: Duration::from_millis(600),
                };
            }

            // Jump
            if controller.jump && physics.on_ground && vel.0.z <= 0.0 {
                local_emitter.emit(LocalEvent::Jump(entity));
            }
        }
    }
}
