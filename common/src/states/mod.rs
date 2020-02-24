// Module declarations
pub mod basic_attack;
pub mod basic_block;
pub mod climb;
pub mod glide;
pub mod idle;
pub mod roll;
pub mod sit;
pub mod utils;
pub mod wielded;
pub mod wielding;

use crate::comp::{EcsStateData, StateUpdate};

/// ## A type for implementing State Handling Behavior.
///
/// Called by state machines' update functions to allow current states to handle
/// updating their parent machine's current state.
///
/// Structures must implement a `handle()` fn to handle update behavior, and a
/// `new()` for instantiating new instances of a state. `handle()` function
/// recieves `EcsStateData`, a struct of readonly ECS Component data, and
/// returns a `StateUpdate` tuple, with new components that will overwrite an
/// entitie's old components.
pub trait StateHandler: Default {
    fn handle(&self, ecs_data: &EcsStateData) -> StateUpdate;
    fn new(ecs_data: &EcsStateData) -> Self;
}
