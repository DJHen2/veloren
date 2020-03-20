use crate::{
    comp::{CharacterState, ItemKind::Tool, StateUpdate, ToolData},
    event::LocalEvent,
    states::*,
    sys::{character_behavior::JoinData, phys::GRAVITY},
};
use std::time::Duration;
use vek::vec::{Vec2, Vec3};

pub const MOVEMENT_THRESHOLD_VEL: f32 = 3.0;
const BASE_HUMANOID_ACCEL: f32 = 100.0;
const BASE_HUMANOID_SPEED: f32 = 150.0;
const BASE_HUMANOID_AIR_ACCEL: f32 = 15.0;
const BASE_HUMANOID_AIR_SPEED: f32 = 8.0;
const BASE_HUMANOID_WATER_ACCEL: f32 = 70.0;
const BASE_HUMANOID_WATER_SPEED: f32 = 120.0;
// const BASE_HUMANOID_CLIMB_ACCEL: f32 = 10.0;
// const ROLL_SPEED: f32 = 17.0;
// const CHARGE_SPEED: f32 = 20.0;
// const GLIDE_ACCEL: f32 = 15.0;
// const GLIDE_SPEED: f32 = 45.0;
// const BLOCK_ACCEL: f32 = 30.0;
// const BLOCK_SPEED: f32 = 75.0;
// Gravity is 9.81 * 4, so this makes gravity equal to .15 //TODO: <- is wrong
//
// const GLIDE_ANTIGRAV: f32 = GRAVITY * 0.96;
// const CLIMB_SPEED: f32 = 5.0;
// const CLIMB_COST: i32 = 5;

/// Handles updating `Components` to move player based on state of `JoinData`
pub fn handle_move(data: &JoinData, update: &mut StateUpdate) {
    if data.physics.in_fluid {
        swim_move(data, update);
    } else {
        basic_move(data, update);
    }
}

/// Updates components to move player as if theyre on ground or in air
fn basic_move(data: &JoinData, update: &mut StateUpdate) {
    let (accel, speed): (f32, f32) = if data.physics.on_ground {
        (BASE_HUMANOID_ACCEL, BASE_HUMANOID_SPEED)
    } else {
        (BASE_HUMANOID_AIR_ACCEL, BASE_HUMANOID_AIR_SPEED)
    };

    // Move player according to move_dir
    if update.vel.0.magnitude_squared() < speed.powf(2.0) {
        update.vel.0 = update.vel.0 + Vec2::broadcast(data.dt.0) * data.inputs.move_dir * accel;
        let mag2 = update.vel.0.magnitude_squared();
        if mag2 > speed.powf(2.0) {
            update.vel.0 = update.vel.0.normalized() * speed;
        }
    }

    // Set direction based on move direction
    let ori_dir = if update.character.is_attack() || update.character.is_block() {
        Vec2::from(data.inputs.look_dir).normalized()
    } else {
        Vec2::from(data.inputs.move_dir)
    };

    // Smooth orientation
    if ori_dir.magnitude_squared() > 0.0001
        && (update.ori.0.normalized() - Vec3::from(ori_dir).normalized()).magnitude_squared()
            > 0.001
    {
        update.ori.0 = vek::ops::Slerp::slerp(update.ori.0, ori_dir.into(), 9.0 * data.dt.0);
    }
}

/// Updates components to move player as if theyre swimming
fn swim_move(data: &JoinData, update: &mut StateUpdate) {
    // Update velocity
    update.vel.0 += Vec2::broadcast(data.dt.0)
        * data.inputs.move_dir
        * if update.vel.0.magnitude_squared() < BASE_HUMANOID_WATER_SPEED.powf(2.0) {
            BASE_HUMANOID_WATER_ACCEL
        } else {
            0.0
        };

    // Set direction based on move direction when on the ground
    let ori_dir = if update.character.is_attack() || update.character.is_block() {
        Vec2::from(data.inputs.look_dir).normalized()
    } else {
        Vec2::from(update.vel.0)
    };

    if ori_dir.magnitude_squared() > 0.0001
        && (update.ori.0.normalized() - Vec3::from(ori_dir).normalized()).magnitude_squared()
            > 0.001
    {
        update.ori.0 = vek::ops::Slerp::slerp(
            update.ori.0,
            ori_dir.into(),
            if data.physics.on_ground { 9.0 } else { 2.0 } * data.dt.0,
        );
    }

    // Force players to pulse jump button to swim up
    if data.inputs.jump.is_pressed() && !data.inputs.jump.is_long_press(Duration::from_millis(600))
    {
        update.vel.0.z =
            (update.vel.0.z + data.dt.0 * GRAVITY * 1.25).min(BASE_HUMANOID_WATER_SPEED);
    }
}

/// First checks whether `primary` input is pressed, then
/// attempts to go into Equipping state, otherwise Idle
pub fn handle_wield(data: &JoinData, update: &mut StateUpdate) {
    if data.inputs.primary.is_pressed() {
        attempt_wield(data, update);
    }
}

/// If a tool is equipped, goes into Equipping state, otherwise goes to Idle
pub fn attempt_wield(data: &JoinData, update: &mut StateUpdate) {
    if let Some(Tool(tool)) = data.loadout.active_item.as_ref().map(|i| &i.item.kind) {
        update.character = CharacterState::Equipping(equipping::Data {
            time_left: tool.equip_time(),
        });
    } else {
        update.character = CharacterState::Idle {};
    };
}

/// Checks that player can `Sit` and updates `CharacterState` if so
pub fn handle_sit(data: &JoinData, update: &mut StateUpdate) {
    if data.inputs.sit.is_pressed() && data.physics.on_ground && data.body.is_humanoid() {
        update.character = CharacterState::Sit {};
    }
}

/// Checks that player can `Climb` and updates `CharacterState` if so
pub fn handle_climb(data: &JoinData, update: &mut StateUpdate) {
    if (data.inputs.climb.is_pressed() || data.inputs.climb_down.is_pressed())
        && data.physics.on_wall.is_some()
        && !data.physics.on_ground
        //&& update.vel.0.z < 0.0
        && data.body.is_humanoid()
        && update.energy.current() > 100
    {
        update.character = CharacterState::Climb {};
    }
}

/// Checks that player can `Glide` and updates `CharacterState` if so
pub fn handle_unwield(data: &JoinData, update: &mut StateUpdate) {
    if let CharacterState::Wielding { .. } = update.character {
        if data.inputs.toggle_wield.is_pressed() {
            update.character = CharacterState::Idle {};
        }
    }
}

/// Checks that player can glide and updates `CharacterState` if so
pub fn handle_glide(data: &JoinData, update: &mut StateUpdate) {
    if let CharacterState::Idle { .. } | CharacterState::Wielding { .. } = update.character {
        if data.inputs.glide.is_pressed()
            && !data.physics.on_ground
            && !data.physics.in_fluid
            && data.body.is_humanoid()
        {
            update.character = CharacterState::Glide {};
        }
    }
}

/// Checks that player can jump and sends jump event if so
pub fn handle_jump(data: &JoinData, update: &mut StateUpdate) {
    if data.inputs.jump.is_pressed() && data.physics.on_ground && !data.physics.in_fluid {
        update
            .local_events
            .push_front(LocalEvent::Jump(data.entity));
    }
}

/// If `inputs.primary` is pressed and in `Wielding` state,
/// will attempt to go into `loadout.active_item.primary_ability`
pub fn handle_primary_input(data: &JoinData, update: &mut StateUpdate) {
    if data.inputs.primary.is_pressed() {
        if let Some(ability) = data
            .loadout
            .active_item
            .as_ref()
            .and_then(|i| i.primary_ability.as_ref())
            .filter(|ability| ability.test_requirements(data, update))
        {
            update.character = ability.into();
        }
    }
}

/// If `inputs.secondary` is pressed and in `Wielding` state,
/// will attempt to go into `loadout.active_item.secondary_ability`
pub fn handle_secondary_input(data: &JoinData, update: &mut StateUpdate) {
    if data.inputs.secondary.is_pressed() {
        if let Some(ability) = data
            .loadout
            .active_item
            .as_ref()
            .and_then(|i| i.secondary_ability.as_ref())
            .filter(|ability| ability.test_requirements(data, update))
        {
            update.character = ability.into();
        }
    }
}

/// Checks that player can perform a dodge, then
/// attempts to go into `loadout.active_item.dodge_ability`
pub fn handle_dodge_input(data: &JoinData, update: &mut StateUpdate) {
    if data.inputs.roll.is_pressed() {
        if let Some(ability) = data
            .loadout
            .active_item
            .as_ref()
            .and_then(|i| i.dodge_ability.as_ref())
            .filter(|ability| ability.test_requirements(data, update))
        {
            update.character = ability.into();
        }
    }
}

pub fn unwrap_tool_data<'a>(data: &'a JoinData) -> Option<&'a ToolData> {
    if let Some(Tool(tool)) = data.loadout.active_item.as_ref().map(|i| &i.item.kind) {
        Some(tool)
    } else {
        None
    }
}
