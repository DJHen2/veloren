use crate::{
    comp::{Energy, Ori, Pos, ToolData, Vel},
    event::{LocalEvent, ServerEvent},
    states::*,
};
use serde::{Deserialize, Serialize};
use specs::{Component, FlaggedStorage, HashMapStorage, VecStorage};
use std::collections::VecDeque;

/// Data returned from character behavior fn's to Character Behavior System.
pub struct StateUpdate {
    pub character: CharacterState,
    pub pos: Pos,
    pub vel: Vel,
    pub ori: Ori,
    pub energy: Energy,
    pub local_events: VecDeque<LocalEvent>,
    pub server_events: VecDeque<ServerEvent>,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum CharacterState {
    Idle,
    Climb,
    Sit,
    Glide,
    /// A basic blocking state
    BasicBlock,
    /// Player is busy equipping or unequipping weapons
    Equipping(equipping::Data),
    /// Player is holding a weapon and can perform other actions
    Wielding(wielding::Data),
    /// Player rushes forward and slams an enemy with their weapon
    ChargeAttack(charge_attack::Data),
    /// A dodge where player can roll
    Roll(roll::Data),
    /// A basic attacking state
    BasicAttack(basic_attack::Data),
    /// A three-stage attack where play must click at appropriate times
    /// to continue attack chain.
    TimedCombo(timed_combo::Data),
    /// A three-stage attack where each attack pushes player forward
    /// and successive attacks increase in damage, while player holds button.
    TripleStrike(triple_strike::Data),
}

impl CharacterState {
    pub fn is_wield(&self) -> bool {
        match self {
            CharacterState::Wielding(_)
            | CharacterState::BasicAttack(_)
            | CharacterState::TimedCombo(_)
            | CharacterState::BasicBlock => true,
            _ => false,
        }
    }

    pub fn is_attack(&self) -> bool {
        match self {
            CharacterState::BasicAttack(_)
            | CharacterState::TimedCombo(_)
            | CharacterState::ChargeAttack(_) => true,
            _ => false,
        }
    }

    pub fn is_block(&self) -> bool {
        match self {
            CharacterState::BasicBlock => true,
            _ => false,
        }
    }

    pub fn is_dodge(&self) -> bool {
        match self {
            CharacterState::Roll(_) => true,
            _ => false,
        }
    }

    /// Compares for shallow equality (does not check internal struct equality)
    pub fn equals(&self, other: &Self) -> bool {
        // Check if state is the same without looking at the inner data
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Default for CharacterState {
    fn default() -> Self { Self::Idle }
}

impl Component for CharacterState {
    type Storage = FlaggedStorage<Self, HashMapStorage<Self>>;
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Attacking {
    pub base_damage: u32,
    pub applied: bool,
    pub hit_count: u32,
}

impl Component for Attacking {
    type Storage = VecStorage<Self>;
}
