use super::{GLIDE_ACCEL, GLIDE_ANTIGRAV, GLIDE_SPEED};
use crate::comp::{
    ActionState::*, ClimbState, EcsStateData, FallState, IdleState, MoveState::*, StandState,
    StateHandle, StateUpdate,
};
use vek::{Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct GlideState;

impl StateHandle for GlideState {
    fn handle(&self, ecs_data: &EcsStateData) -> StateUpdate {
        let mut update = StateUpdate {
            pos: *ecs_data.pos,
            vel: *ecs_data.vel,
            ori: *ecs_data.ori,
            character: *ecs_data.character,
        };

        // Defaults for this state
        update.character.action_state = Idle(IdleState);
        update.character.move_state = Glide(GlideState);

        // Move player according to movement direction vector
        update.vel.0 += Vec2::broadcast(ecs_data.dt.0)
            * ecs_data.inputs.move_dir
            * if ecs_data.vel.0.magnitude_squared() < GLIDE_SPEED.powf(2.0) {
                GLIDE_ACCEL
            } else {
                0.0
            };

        // Determine orientation vector from movement direction vector
        let ori_dir = Vec2::from(update.vel.0);
        if ori_dir.magnitude_squared() > 0.0001
            && (update.ori.0.normalized() - Vec3::from(ori_dir).normalized()).magnitude_squared()
                > 0.001
        {
            update.ori.0 =
                vek::ops::Slerp::slerp(update.ori.0, ori_dir.into(), 2.0 * ecs_data.dt.0);
        }

        // Apply Glide antigrav lift
        if Vec2::<f32>::from(update.vel.0).magnitude_squared() < GLIDE_SPEED.powf(2.0)
            && update.vel.0.z < 0.0
        {
            let lift = GLIDE_ANTIGRAV + update.vel.0.z.abs().powf(2.0) * 0.15;
            update.vel.0.z += ecs_data.dt.0
                * lift
                * (Vec2::<f32>::from(update.vel.0).magnitude() * 0.075)
                    .min(1.0)
                    .max(0.2);
        }

        // If glide button isn't held, start falling
        if !ecs_data.inputs.glide.is_pressed() {
            update.character.move_state = Fall(FallState);
            return update;
        }

        // If there is a wall in front of character go to climb
        if let Some(_wall_dir) = ecs_data.physics.on_wall {
            update.character.move_state = Climb(ClimbState);
            return update;
        }

        // If on ground go to stand
        if ecs_data.physics.on_ground {
            update.character.move_state = Stand(StandState);
            return update;
        }

        // Otherwise keep gliding
        return update;
    }
}
