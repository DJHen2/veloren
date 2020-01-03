use self::ActionState::*;
use super::states::*;
use crate::{
    comp::{Body, ControllerInputs, Ori, PhysicsState, Pos, Stats, Vel},
    event::{EventBus, LocalEvent, ServerEvent},
    state::DeltaTime,
};
use specs::LazyUpdate;
use specs::{Component, Entity, FlaggedStorage, HashMapStorage, NullStorage};
use sphynx::Uid;
use std::time::Duration;

pub struct EcsStateData<'a> {
    pub entity: &'a Entity,
    pub uid: &'a Uid,
    pub character: &'a CharacterState,
    pub pos: &'a Pos,
    pub vel: &'a Vel,
    pub ori: &'a Ori,
    pub dt: &'a DeltaTime,
    pub inputs: &'a ControllerInputs,
    pub stats: &'a Stats,
    pub body: &'a Body,
    pub physics: &'a PhysicsState,
    pub updater: &'a LazyUpdate,
    pub server_bus: &'a EventBus<ServerEvent>,
    pub local_bus: &'a EventBus<LocalEvent>,
}

pub struct StateUpdate {
    pub character: CharacterState,
    pub pos: Pos,
    pub vel: Vel,
    pub ori: Ori,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum MoveState {
    Stand(StandState),
    Run(RunState),
    Sit(SitState),
    Jump(JumpState),
    Fall(FallState),
    Glide(GlideState),
    Swim(SwimState),
    Climb(ClimbState),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum ActionState {
    Idle(IdleState),
    Wield(WieldState),
    Attack(AttackKind),
    Block(BlockKind),
    Dodge(DodgeKind),
    // Interact?,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum AttackKind {
    BasicAttack(BasicAttackState),
    Charge(ChargeAttackState),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum BlockKind {
    BasicBlock(BasicBlockState),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum DodgeKind {
    Roll(RollState),
}

impl ActionState {
    pub fn is_equip_finished(&self) -> bool {
        match self {
            Wield(WieldState { equip_delay }) => *equip_delay == Duration::default(),
            _ => true,
        }
    }

    /// Returns the current `equip_delay` if in `WieldState`, otherwise `Duration::default()`
    pub fn get_delay(&self) -> Duration {
        match self {
            Wield(WieldState { equip_delay }) => *equip_delay,
            _ => Duration::default(),
        }
    }

    pub fn is_attacking(&self) -> bool {
        match self {
            Block(_) => true,
            _ => false,
        }
    }

    pub fn is_blocking(&self) -> bool {
        match self {
            Attack(_) => true,
            _ => false,
        }
    }

    pub fn is_dodging(&self) -> bool {
        match self {
            Dodge(_) => true,
            _ => false,
        }
    }

    pub fn is_wielding(&self) -> bool {
        if let Wield(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_idling(&self) -> bool {
        if let Idle(_) = self {
            true
        } else {
            false
        }
    }
}

/// __A concurrent state machine that allows for separate `ActionState`s and `MoveState`s.__
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct CharacterState {
    /// __How the character is currently moving, e.g. Running, Standing, Falling.__
    ///
    /// _Primarily `handle()`s updating `Pos`, `Vel`, `Ori`, and lower body animations.
    pub move_state: MoveState,

    /// __How the character is currently acting, e.g. Wielding, Attacking, Dodging.__
    ///
    /// _Primarily `handle()`s how character interacts with world, and upper body animations.
    pub action_state: ActionState,
}

impl CharacterState {
    /// Compares `move_state`s for shallow equality (does not check internal struct equality)
    pub fn is_same_move_state(&self, other: &Self) -> bool {
        // Check if state is the same without looking at the inner data
        std::mem::discriminant(&self.move_state) == std::mem::discriminant(&other.move_state)
    }

    /// Compares `action_state`s for shallow equality (does not check internal struct equality)
    pub fn is_same_action_state(&self, other: &Self) -> bool {
        // Check if state is the same without looking at the inner data
        std::mem::discriminant(&self.action_state) == std::mem::discriminant(&other.action_state)
    }

    /// Compares both `move_state`s and `action_state`a for shallow equality
    /// (does not check internal struct equality)
    pub fn is_same_state(&self, other: &Self) -> bool {
        self.is_same_move_state(other) && self.is_same_action_state(other)
    }
}

impl Default for CharacterState {
    fn default() -> Self {
        Self {
            move_state: MoveState::Fall(FallState),
            action_state: ActionState::Idle(IdleState),
        }
    }
}

impl Component for CharacterState {
    type Storage = FlaggedStorage<Self, HashMapStorage<Self>>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct OverrideState;
impl Component for OverrideState {
    type Storage = FlaggedStorage<Self, NullStorage<Self>>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct OverrideAction;
impl Component for OverrideAction {
    type Storage = FlaggedStorage<Self, NullStorage<Self>>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct OverrideMove;
impl Component for OverrideMove {
    type Storage = FlaggedStorage<Self, NullStorage<Self>>;
}
