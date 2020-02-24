use crate::{
    comp::{Attacking, CharacterState, EcsStateData, ItemKind::Tool, StateUpdate, ToolData},
    states::StateHandler,
};
use std::{collections::VecDeque, time::Duration};

#[derive(Clone, Copy, Default, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct State {
    /// How long the state has until exitting
    pub remaining_duration: Duration,
    /// Whether the attack can deal more damage
    pub exhausted: bool,
}

impl StateHandler for State {
    fn new(ecs_data: &EcsStateData) -> Self {
        let remaining_duration =
            if let Some(Tool(data)) = ecs_data.stats.equipment.main.as_ref().map(|i| i.kind) {
                data.attack_duration()
            } else {
                Duration::from_millis(300)
            };
        Self {
            remaining_duration,
            exhausted: false,
        }
    }

    fn handle(&self, ecs_data: &EcsStateData) -> StateUpdate {
        let mut update = StateUpdate {
            pos: *ecs_data.pos,
            vel: *ecs_data.vel,
            ori: *ecs_data.ori,
            energy: *ecs_data.energy,
            character: *ecs_data.character,
            local_events: VecDeque::new(),
            server_events: VecDeque::new(),
        };

        let tool_kind = ecs_data.stats.equipment.main.as_ref().map(|i| i.kind);

        let can_apply_damage = !self.exhausted
            && if let Some(Tool(data)) = tool_kind {
                (self.remaining_duration < data.attack_recover_duration())
            } else {
                true
            };

        let mut exhausted = self.exhausted;

        if can_apply_damage {
            if let Some(Tool(data)) = tool_kind {
                ecs_data
                    .updater
                    .insert(*ecs_data.entity, Attacking { weapon: Some(data) });
            } else {
                ecs_data
                    .updater
                    .insert(*ecs_data.entity, Attacking { weapon: None });
            }
            exhausted = true;
        } else {
            ecs_data.updater.remove::<Attacking>(*ecs_data.entity);
        }

        let remaining_duration = self
            .remaining_duration
            .checked_sub(Duration::from_secs_f32(ecs_data.dt.0))
            .unwrap_or_default();

        // Tick down
        update.character = CharacterState::BasicAttack(Some(State {
            remaining_duration,
            exhausted,
        }));

        // Check if attack duration has expired
        if remaining_duration == Duration::default() {
            update.character = CharacterState::Wielded(None);
            ecs_data.updater.remove::<Attacking>(*ecs_data.entity);
        }

        update
    }
}
