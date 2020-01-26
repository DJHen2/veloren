pub mod attack;
pub mod block;
pub mod blockidle;
pub mod charge;
pub mod cidle;
pub mod climb;
pub mod gliding;
pub mod idle;
pub mod jump;
pub mod roll;
pub mod run;
pub mod sit;
pub mod stand;
pub mod swim;
pub mod wield;

// Reexports
pub use self::attack::AttackAnimation;
pub use self::block::BlockAnimation;
pub use self::blockidle::BlockIdleAnimation;
pub use self::charge::ChargeAnimation;
pub use self::cidle::CidleAnimation;
pub use self::climb::ClimbAnimation;
pub use self::gliding::GlidingAnimation;
pub use self::idle::IdleAnimation;
pub use self::jump::JumpAnimation;
pub use self::roll::RollAnimation;
pub use self::run::RunAnimation;
pub use self::sit::SitAnimation;
pub use self::stand::StandAnimation;
pub use self::swim::SwimAnimation;
pub use self::wield::WieldAnimation;

use super::{Bone, Skeleton};
use crate::render::FigureBoneData;
use common::comp::{self, item::Tool};

#[derive(Clone, Default)]
pub struct CharacterSkeleton {
    head: Bone,
    chest: Bone,
    belt: Bone,
    shorts: Bone,
    l_hand: Bone,
    r_hand: Bone,
    l_foot: Bone,
    r_foot: Bone,
    main: Bone,
    l_shoulder: Bone,
    r_shoulder: Bone,
    glider: Bone,
    lantern: Bone,
    torso: Bone,
}

impl CharacterSkeleton {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Skeleton for CharacterSkeleton {
    type Attr = SkeletonAttr;
    fn compute_matrices(&self) -> [FigureBoneData; 16] {
        let chest_mat = self.chest.compute_base_matrix();
        let torso_mat = self.torso.compute_base_matrix();
        let l_hand_mat = self.l_hand.compute_base_matrix();
        let main_mat = self.main.compute_base_matrix();

        let head_mat = self.head.compute_base_matrix();
        [
            FigureBoneData::new(torso_mat * head_mat),
            FigureBoneData::new(torso_mat * chest_mat),
            FigureBoneData::new(torso_mat * self.belt.compute_base_matrix()),
            FigureBoneData::new(torso_mat * self.shorts.compute_base_matrix()),
            FigureBoneData::new(torso_mat * chest_mat * l_hand_mat),
            FigureBoneData::new(torso_mat * chest_mat * self.r_hand.compute_base_matrix()),
            FigureBoneData::new(torso_mat * self.l_foot.compute_base_matrix()),
            FigureBoneData::new(torso_mat * self.r_foot.compute_base_matrix()),
            FigureBoneData::new(torso_mat * chest_mat * main_mat),
            FigureBoneData::new(torso_mat * chest_mat * self.l_shoulder.compute_base_matrix()),
            FigureBoneData::new(torso_mat * chest_mat * self.r_shoulder.compute_base_matrix()),
            FigureBoneData::new(torso_mat * self.glider.compute_base_matrix()),
            FigureBoneData::new(torso_mat * chest_mat * self.lantern.compute_base_matrix()),
            FigureBoneData::new(torso_mat),
            FigureBoneData::default(),
            FigureBoneData::default(),
        ]
    }

    fn interpolate(&mut self, target: &Self, dt: f32) {
        self.head.interpolate(&target.head, dt);
        self.chest.interpolate(&target.chest, dt);
        self.belt.interpolate(&target.belt, dt);
        self.shorts.interpolate(&target.shorts, dt);
        self.l_hand.interpolate(&target.l_hand, dt);
        self.r_hand.interpolate(&target.r_hand, dt);
        self.l_foot.interpolate(&target.l_foot, dt);
        self.r_foot.interpolate(&target.r_foot, dt);
        self.main.interpolate(&target.main, dt);
        self.l_shoulder.interpolate(&target.l_shoulder, dt);
        self.r_shoulder.interpolate(&target.r_shoulder, dt);
        self.glider.interpolate(&target.glider, dt);
        self.lantern.interpolate(&target.lantern, dt);
        self.torso.interpolate(&target.torso, dt);
    }
}

pub struct SkeletonAttr {
    scaler: f32,
    head_scale: f32,
    neck_height: f32,
    neck_forward: f32,
    neck_right: f32,
    weapon_x: f32,
    weapon_y: f32,
}
impl SkeletonAttr {
    pub fn calculate_scale(body: &comp::humanoid::Body) -> f32 {
        use comp::humanoid::{BodyType::*, Race::*};
        match (body.race, body.body_type) {
            (Orc, Male) => 0.95,
            (Orc, Female) => 0.8,
            (Human, Male) => 0.8,
            (Human, Female) => 0.75,
            (Elf, Male) => 0.85,
            (Elf, Female) => 0.8,
            (Dwarf, Male) => 0.7,
            (Dwarf, Female) => 0.65,
            (Undead, Male) => 0.8,
            (Undead, Female) => 0.75,
            (Danari, Male) => 0.58,
            (Danari, Female) => 0.58,
        }
    }
}

impl Default for SkeletonAttr {
    fn default() -> Self {
        Self {
            scaler: 1.0,
            head_scale: 1.0,
            neck_height: 1.0,
            neck_forward: 1.0,
            neck_right: 1.0,
            weapon_x: 1.0,
            weapon_y: 1.0,
        }
    }
}

impl<'a> std::convert::TryFrom<&'a comp::Body> for SkeletonAttr {
    type Error = ();

    fn try_from(body: &'a comp::Body) -> Result<Self, Self::Error> {
        match body {
            comp::Body::Humanoid(body) => Ok(SkeletonAttr::from(body)),
            _ => Err(()),
        }
    }
}

impl<'a> From<&'a comp::humanoid::Body> for SkeletonAttr {
    fn from(body: &'a comp::humanoid::Body) -> Self {
        use comp::humanoid::{BodyType::*, Race::*};
        Self {
            scaler: SkeletonAttr::calculate_scale(body),
            head_scale: match (body.race, body.body_type) {
                (Orc, Male) => 0.9,
                (Orc, Female) => 1.0,
                (Human, Male) => 1.0,
                (Human, Female) => 1.0,
                (Elf, Male) => 0.95,
                (Elf, Female) => 1.0,
                (Dwarf, Male) => 1.0,
                (Dwarf, Female) => 1.0,
                (Undead, Male) => 1.0,
                (Undead, Female) => 1.0,
                (Danari, Male) => 1.15,
                (Danari, Female) => 1.15,
            },
            neck_height: match (body.race, body.body_type) {
                (Orc, Male) => 0.0,
                (Orc, Female) => 0.0,
                (Human, Male) => 0.0,
                (Human, Female) => 0.0,
                (Elf, Male) => 0.0,
                (Elf, Female) => 0.0,
                (Dwarf, Male) => 0.0,
                (Dwarf, Female) => 0.0,
                (Undead, Male) => 0.5,
                (Undead, Female) => 0.5,
                (Danari, Male) => 0.5,
                (Danari, Female) => 0.5,
            },
            neck_forward: match (body.race, body.body_type) {
                (Orc, Male) => 0.0,
                (Orc, Female) => 0.0,
                (Human, Male) => 0.5,
                (Human, Female) => 0.0,
                (Elf, Male) => 0.5,
                (Elf, Female) => 0.5,
                (Dwarf, Male) => 0.5,
                (Dwarf, Female) => 0.0,
                (Undead, Male) => 0.5,
                (Undead, Female) => 0.5,
                (Danari, Male) => 0.0,
                (Danari, Female) => 0.0,
            },
            neck_right: match (body.race, body.body_type) {
                (Orc, Male) => 0.0,
                (Orc, Female) => 0.0,
                (Human, Male) => 0.0,
                (Human, Female) => 0.0,
                (Elf, Male) => 0.0,
                (Elf, Female) => 0.0,
                (Dwarf, Male) => 0.0,
                (Dwarf, Female) => 0.0,
                (Undead, Male) => 0.0,
                (Undead, Female) => 0.0,
                (Danari, Male) => 0.0,
                (Danari, Female) => 0.0,
            },
            weapon_x: match Tool::Hammer {
                // TODO: Inventory
                Tool::Sword => 0.0,
                Tool::Axe => 3.0,
                Tool::Hammer => 0.0,
                Tool::Shield => 3.0,
                Tool::Staff => 3.0,
                Tool::Bow => 0.0,
                Tool::Dagger => 0.0,
                Tool::Debug(_) => 0.0,
            },
            weapon_y: match Tool::Hammer {
                // TODO: Inventory
                Tool::Sword => -1.25,
                Tool::Axe => 0.0,
                Tool::Hammer => -2.0,
                Tool::Shield => 0.0,
                Tool::Staff => 0.0,
                Tool::Bow => -2.0,
                Tool::Dagger => -2.0,
                Tool::Debug(_) => 0.0,
            },
        }
    }
}
