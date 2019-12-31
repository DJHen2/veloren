use crate::comp::{
    ActionState::*, EcsStateData, IdleState, JumpState, MoveState::*, RunState, StandState,
    StateHandle, StateUpdate,
};
use crate::util::movement_utils::*;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct SitState;

impl StateHandle for SitState {
    fn handle(&self, ecs_data: &EcsStateData) -> StateUpdate {
        let mut update = StateUpdate {
            character: *ecs_data.character,
            pos: *ecs_data.pos,
            vel: *ecs_data.vel,
            ori: *ecs_data.ori,
        };

        // Prevent action state handling
        update.character.action_disabled_this_tick = true;
        update.character.action_state = Idle(IdleState);
        update.character.move_state = Sit(SitState);

        // Try to Fall
        // ... maybe the ground disappears,
        // suddenly maybe a water spell appears.
        // Can't hurt to be safe :shrug:
        if !ecs_data.physics.on_ground {
            update.character.move_state = determine_fall_or_swim(ecs_data.physics);
            update.character.move_disabled_this_tick = false;
            return update;
        }
        // Try to jump
        if ecs_data.inputs.jump.is_pressed() {
            update.character.move_state = Jump(JumpState);
            update.character.action_disabled_this_tick = false;
            return update;
        }

        // Try to Run
        if ecs_data.inputs.move_dir.magnitude_squared() > 0.0 {
            update.character.move_state = Run(RunState);
            update.character.action_disabled_this_tick = false;
            return update;
        }

        // Try to Stand
        if ecs_data.inputs.sit.is_just_pressed() {
            update.character.move_state = Stand(StandState);
            update.character.action_disabled_this_tick = false;
            return update;
        }

        // No move has occurred, keep sitting
        return update;
    }
}
