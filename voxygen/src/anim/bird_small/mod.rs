pub mod idle;
pub mod jump;
pub mod run;

// Reexports
pub use self::idle::IdleAnimation;
pub use self::jump::JumpAnimation;
pub use self::run::RunAnimation;

use super::{Bone, Skeleton};
use crate::render::FigureBoneData;

#[derive(Clone)]
pub struct BirdSmallSkeleton {
    head: Bone,
    torso: Bone,
    wing_l: Bone,
    wing_r: Bone,
}

impl BirdSmallSkeleton {
    pub fn new() -> Self {
        Self {
            head: Bone::default(),
            torso: Bone::default(),
            wing_l: Bone::default(),
            wing_r: Bone::default(),
        }
    }
}

impl Skeleton for BirdSmallSkeleton {
    fn compute_matrices(&self) -> [FigureBoneData; 16] {
        let torso_mat = self.torso.compute_base_matrix();

        [
            FigureBoneData::new(self.head.compute_base_matrix() * torso_mat),
            FigureBoneData::new(torso_mat),
            FigureBoneData::new(self.wing_l.compute_base_matrix() * torso_mat),
            FigureBoneData::new(self.wing_r.compute_base_matrix() * torso_mat),
            FigureBoneData::default(),
            FigureBoneData::default(),
            FigureBoneData::default(),
            FigureBoneData::default(),
            FigureBoneData::default(),
            FigureBoneData::default(),
            FigureBoneData::default(),
            FigureBoneData::default(),
            FigureBoneData::default(),
            FigureBoneData::default(),
            FigureBoneData::default(),
            FigureBoneData::default(),
        ]
    }

    fn interpolate(&mut self, target: &Self, dt: f32) {
        self.head.interpolate(&target.head, dt);
        self.torso.interpolate(&target.torso, dt);
        self.wing_l.interpolate(&target.wing_l, dt);
        self.wing_r.interpolate(&target.wing_r, dt);
    }
}
