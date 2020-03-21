use crate::{
    comp::{CharacterState, StateUpdate},
    sys::character_behavior::{CharacterBehavior, JoinData},
    util::safe_slerp,
};
use std::time::Duration;
use vek::Vec3;

const ROLL_SPEED: f32 = 17.0;
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Data {
    /// How long the state has until exiting
    pub remaining_duration: Duration,
}

impl CharacterBehavior for Data {
    fn behavior(&self, data: &JoinData) -> StateUpdate {
        let mut update = StateUpdate::from(data);

        // Update velocity
        update.vel.0 = Vec3::new(0.0, 0.0, update.vel.0.z)
            + (update.vel.0 * Vec3::new(1.0, 1.0, 0.0)
                + 1.5 * data.inputs.move_dir.try_normalized().unwrap_or_default())
            .try_normalized()
            .unwrap_or_default()
                * ROLL_SPEED;

        // Smooth orientation
        update.ori.0 = safe_slerp(update.ori.0, update.vel.0.into(), 9.0 * data.dt.0);

        if self.remaining_duration == Duration::default() {
            // Roll duration has expired
            update.vel.0 *= 0.3;
            update.character = CharacterState::Idle {};
        } else {
            // Otherwise, tick down remaining_duration
            update.character = CharacterState::Roll(Data {
                remaining_duration: self
                    .remaining_duration
                    .checked_sub(Duration::from_secs_f32(data.dt.0))
                    .unwrap_or_default(),
            });
        }

        update
    }
}
