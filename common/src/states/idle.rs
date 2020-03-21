use super::utils::*;
use crate::{
    comp::StateUpdate,
    sys::character_behavior::{CharacterBehavior, JoinData},
};

pub struct Data;

impl CharacterBehavior for Data {
    fn behavior(&self, data: &JoinData) -> StateUpdate {
        let mut update = StateUpdate::from(data);

        handle_move(data, &mut update);
        handle_jump(data, &mut update);
        handle_wield(data, &mut update);
        handle_sit(data, &mut update);
        handle_climb(data, &mut update);
        handle_glide(data, &mut update);
        handle_dodge_input(data, &mut update);

        update
    }
}
