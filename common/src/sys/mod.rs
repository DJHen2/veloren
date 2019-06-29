pub mod agent;
pub mod animation;
pub mod combat;
pub mod controller;
pub mod phys;
mod stats;
mod action_state;

// External
use specs::DispatcherBuilder;

// System names
const AGENT_SYS: &str = "agent_sys";
const CONTROLLER_SYS: &str = "controller_sys";
const ACTION_STATE_SYS: &str = "action_state_sys";
const PHYS_SYS: &str = "phys_sys";
const COMBAT_SYS: &str = "combat_sys";
const ANIMATION_SYS: &str = "animation_sys";
const STATS_SYS: &str = "stats_sys";

pub fn add_local_systems(dispatch_builder: &mut DispatcherBuilder) {
    dispatch_builder.add(agent::Sys, AGENT_SYS, &[]);
    dispatch_builder.add(controller::Sys, CONTROLLER_SYS, &[AGENT_SYS]);
    dispatch_builder.add(phys::Sys, PHYS_SYS, &[CONTROLLER_SYS]);
    dispatch_builder.add(action_state::Sys, ACTION_STATE_SYS, &[CONTROLLER_SYS, PHYS_SYS]);
    dispatch_builder.add(combat::Sys, COMBAT_SYS, &[ACTION_STATE_SYS]);
    dispatch_builder.add(animation::Sys, ANIMATION_SYS, &[ACTION_STATE_SYS]);
    dispatch_builder.add(stats::Sys, STATS_SYS, &[COMBAT_SYS]);
}
