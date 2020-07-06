use crate::{comp, sync::Uid};
use specs::{Component, FlaggedStorage};
use specs_idvs::IdvStorage;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Effect {
    Damage(comp::HealthChange),
    Knockback(f32),
    RewardEnergy(u32),
    Explode { power: f32 },
    Vanish,
    Stick,
    Possess,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Projectile {
    // TODO: use SmallVec for these effects
    pub hit_solid: Vec<Effect>,
    pub hit_entity: Vec<Effect>,
    /// Time left until the projectile will despawn
    pub time_left: Duration,
    pub owner: Option<Uid>,
}

impl Projectile {
    pub fn set_owner(&mut self, new_owner: Uid) {
        self.owner = Some(new_owner);
        for e in self.hit_solid.iter_mut().chain(self.hit_entity.iter_mut()) {
            if let Effect::Damage(comp::HealthChange {
                cause: comp::HealthSource::Projectile { owner, .. },
                ..
            }) = e
            {
                *owner = Some(new_owner);
            }
        }
    }
}

impl Component for Projectile {
    type Storage = FlaggedStorage<Self, IdvStorage<Self>>;
}
