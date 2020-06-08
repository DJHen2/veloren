use crate::{comp, sync::Uid, util::Dir};
use comp::{item::ToolKind, InventoryUpdateEvent, Item};
use parking_lot::Mutex;
use serde::Deserialize;
use specs::Entity as EcsEntity;
use std::{collections::VecDeque, ops::DerefMut};
use vek::*;

pub struct SfxEventItem {
    pub sfx: SfxEvent,
    pub pos: Option<Vec3<f32>>,
    pub vol: Option<f32>,
}

impl SfxEventItem {
    pub fn new(sfx: SfxEvent, pos: Option<Vec3<f32>>, vol: Option<f32>) -> Self {
        Self { sfx, pos, vol }
    }

    pub fn at_player_position(sfx: SfxEvent) -> Self {
        Self {
            sfx,
            pos: None,
            vol: None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Hash, Eq)]
pub enum SfxEvent {
    Idle,
    Run,
    Roll,
    Climb,
    GliderOpen,
    Glide,
    GliderClose,
    Jump,
    Fall,
    ExperienceGained,
    LevelUp,
    Attack(ToolKind),
    Wield(ToolKind),
    Unwield(ToolKind),
    Inventory(InventoryUpdateEvent),
}

pub enum LocalEvent {
    /// Applies upward force to entity's `Vel`
    Jump(EcsEntity),
    /// Applies the `force` to `entity`'s `Vel`
    ApplyForce { entity: EcsEntity, force: Vec3<f32> },
    /// Applies leaping force to `entity`'s `Vel` away from `wall_dir` direction
    WallLeap {
        entity: EcsEntity,
        wall_dir: Vec3<f32>,
    },
    /// Applies `vel` velocity to `entity`
    Boost { entity: EcsEntity, vel: Vec3<f32> },
}

pub enum ServerEvent {
    Explosion {
        pos: Vec3<f32>,
        power: f32,
        owner: Option<Uid>,
    },
    Damage {
        uid: Uid,
        change: comp::HealthChange,
    },
    Destroy {
        entity: EcsEntity,
        cause: comp::HealthSource,
    },
    InventoryManip(EcsEntity, comp::InventoryManip),
    Respawn(EcsEntity),
    Shoot {
        entity: EcsEntity,
        dir: Dir,
        body: comp::Body,
        light: Option<comp::LightEmitter>,
        projectile: comp::Projectile,
        gravity: Option<comp::Gravity>,
    },
    LandOnGround {
        entity: EcsEntity,
        vel: Vec3<f32>,
    },
    ToggleLantern(EcsEntity),
    Mount(EcsEntity, EcsEntity),
    Unmount(EcsEntity),
    Possess(Uid, Uid),
    LevelUp(EcsEntity, u32),
    SelectCharacter {
        entity: EcsEntity,
        character_id: i32,
        body: comp::Body,
    },
    ExitIngame {
        entity: EcsEntity,
    },
    CreateNpc {
        pos: comp::Pos,
        stats: comp::Stats,
        loadout: comp::Loadout,
        body: comp::Body,
        agent: comp::Agent,
        alignment: comp::Alignment,
        scale: comp::Scale,
        drop_item: Option<Item>,
    },
    CreateWaypoint(Vec3<f32>),
    ClientDisconnect(EcsEntity),
    ChunkRequest(EcsEntity, Vec2<i32>),
    ChatCmd(EcsEntity, String),
}

pub struct EventBus<E> {
    queue: Mutex<VecDeque<E>>,
}

impl<E> Default for EventBus<E> {
    fn default() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
        }
    }
}

impl<E> EventBus<E> {
    pub fn emitter(&self) -> Emitter<E> {
        Emitter {
            bus: self,
            events: VecDeque::new(),
        }
    }

    pub fn emit_now(&self, event: E) { self.queue.lock().push_back(event); }

    pub fn recv_all(&self) -> impl ExactSizeIterator<Item = E> {
        std::mem::replace(self.queue.lock().deref_mut(), VecDeque::new()).into_iter()
    }
}

pub struct Emitter<'a, E> {
    bus: &'a EventBus<E>,
    events: VecDeque<E>,
}

impl<'a, E> Emitter<'a, E> {
    pub fn emit(&mut self, event: E) { self.events.push_back(event); }

    pub fn append(&mut self, other: &mut VecDeque<E>) { self.events.append(other) }
}

impl<'a, E> Drop for Emitter<'a, E> {
    fn drop(&mut self) { self.bus.queue.lock().append(&mut self.events); }
}
