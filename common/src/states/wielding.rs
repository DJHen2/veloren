use super::utils::*;
use crate::{
    comp::{CharacterState, EcsStateData, ItemKind::Tool, StateUpdate, ToolData},
    states::StateHandler,
};
use std::{collections::VecDeque, time::Duration};

#[derive(Clone, Copy, Default, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct State {
    /// How long before a new action can be performed
    /// after equipping
    pub equip_delay: Duration,
}

impl StateHandler for State {
    fn new(ecs_data: &EcsStateData) -> Self {
        let equip_delay =
            if let Some(Tool(data)) = ecs_data.stats.equipment.main.as_ref().map(|i| i.kind) {
                data.equip_time()
            } else {
                Duration::default()
            };

        Self { equip_delay }
    }

    fn handle(&self, ecs_data: &EcsStateData) -> StateUpdate {
        let mut update = StateUpdate {
            character: *ecs_data.character,
            pos: *ecs_data.pos,
            vel: *ecs_data.vel,
            ori: *ecs_data.ori,
            energy: *ecs_data.energy,
            local_events: VecDeque::new(),
            server_events: VecDeque::new(),
        };

        handle_move_dir(&ecs_data, &mut update);

        if self.equip_delay == Duration::default() {
            // Wield delay has expired
            update.character = CharacterState::Wielded(None);
        } else {
            // Wield delay hasn't expired yet
            // Update wield delay
            update.character = CharacterState::Wielding(Some(State {
                equip_delay: self
                    .equip_delay
                    .checked_sub(Duration::from_secs_f32(ecs_data.dt.0))
                    .unwrap_or_default(),
            }));
        }

        update
    }
}
