pub mod idle;
pub mod jump;
pub mod run;

// Reexports
pub use self::{idle::IdleAnimation, jump::JumpAnimation, run::RunAnimation};

use super::{Bone, FigureBoneData, Skeleton};
use common::comp::{self};
use vek::Vec3;

#[derive(Clone, Default)]
pub struct QuadrupedMediumSkeleton {
    head_upper: Bone,
    head_lower: Bone,
    jaw: Bone,
    tail: Bone,
    torso_front: Bone,
    torso_back: Bone,
    ears: Bone,
    leg_fl: Bone,
    leg_fr: Bone,
    leg_bl: Bone,
    leg_br: Bone,
    foot_fl: Bone,
    foot_fr: Bone,
    foot_bl: Bone,
    foot_br: Bone,
}

#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const HEAD_UPPER_Y: f32 = 0.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const HEAD_UPPER_Z: f32 = 0.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const HEAD_LOWER_Y: f32 = 0.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const HEAD_LOWER_Z: f32 = 0.5;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const JAW_Y: f32 = 0.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const JAW_Z: f32 = -4.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const TAIL_Y: f32 = 0.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const TAIL_Z: f32 = -7.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const TORSO_BACK_Y: f32 = 0.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const TORSO_BACK_Z: f32 = 0.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const TORSO_FRONT_Y: f32 = 0.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const TORSO_FRONT_Z: f32 = 0.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const EARS_Y: f32 = 0.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const EARS_Z: f32 = 5.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const LEG_FRONT_X: f32 = 6.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const LEG_FRONT_Y: f32 = -0.5;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const LEG_FRONT_Z: f32 = 14.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const LEG_BACK_X: f32 = 5.5;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const LEG_BACK_Y: f32 = -14.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const LEG_BACK_Z: f32 = 13.5;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const FEET_FRONT_X: f32 = 0.5;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const FEET_FRONT_Y: f32 = 0.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const FEET_FRONT_Z: f32 = -8.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const FEET_BACK_X: f32 = 0.5;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const FEET_BACK_Y: f32 = 0.0;
#[const_tweaker::tweak(min = -20.0, max = 20.0, step = 0.5)]
const FEET_BACK_Z: f32 = -7.5;


impl QuadrupedMediumSkeleton {
    pub fn new() -> Self { Self::default() }
}

impl Skeleton for QuadrupedMediumSkeleton {
    type Attr = SkeletonAttr;

    fn bone_count(&self) -> usize { 15 }

    #[cfg_attr(feature = "be-dyn-lib", export_name = "quadruped_medium_compute_mats")]
    fn compute_matrices_inner(&self) -> ([FigureBoneData; 16], Vec3<f32>) {
        let ears_mat = self.ears.compute_base_matrix();
        let head_upper_mat = self.head_upper.compute_base_matrix();
        let head_lower_mat = self.head_lower.compute_base_matrix();
        let torso_front_mat = self.torso_front.compute_base_matrix();
		let torso_back_mat = self.torso_back.compute_base_matrix();
        let leg_fl_mat = self.leg_fl.compute_base_matrix();
        let leg_fr_mat = self.leg_fr.compute_base_matrix();
        let leg_bl_mat = self.leg_bl.compute_base_matrix();
        let leg_br_mat = self.leg_br.compute_base_matrix();

        (
            [
                FigureBoneData::new(torso_front_mat * head_lower_mat * head_upper_mat),
                FigureBoneData::new(torso_front_mat * head_lower_mat),
                FigureBoneData::new(torso_front_mat *head_lower_mat * head_upper_mat * self.jaw.compute_base_matrix()),
                FigureBoneData::new(torso_front_mat * torso_back_mat * self.tail.compute_base_matrix()),
                FigureBoneData::new(torso_front_mat),
                FigureBoneData::new(torso_front_mat * torso_back_mat),
                FigureBoneData::new(torso_front_mat *head_lower_mat*head_upper_mat * ears_mat),
                FigureBoneData::new(leg_fl_mat),
                FigureBoneData::new(leg_fr_mat),
                FigureBoneData::new(leg_bl_mat),
                FigureBoneData::new(leg_br_mat),
                FigureBoneData::new(leg_fl_mat * self.foot_fl.compute_base_matrix()),
                FigureBoneData::new(leg_fr_mat * self.foot_fr.compute_base_matrix()),
                FigureBoneData::new(leg_bl_mat * self.foot_bl.compute_base_matrix()),
                FigureBoneData::new(leg_br_mat * self.foot_br.compute_base_matrix()),
                FigureBoneData::default(),
            ],
            Vec3::default(),
        )
    }

    fn interpolate(&mut self, target: &Self, dt: f32) {
        self.head_upper.interpolate(&target.head_upper, dt);
        self.head_lower.interpolate(&target.head_lower, dt);
        self.jaw.interpolate(&target.jaw, dt);
        self.tail.interpolate(&target.tail, dt);
        self.torso_back.interpolate(&target.torso_back, dt);
        self.torso_front.interpolate(&target.torso_front, dt);
        self.ears.interpolate(&target.ears, dt);
        self.leg_fl.interpolate(&target.leg_fl, dt);
        self.leg_fr.interpolate(&target.leg_fr, dt);
        self.leg_bl.interpolate(&target.leg_bl, dt);
        self.leg_br.interpolate(&target.leg_br, dt);
        self.foot_fl.interpolate(&target.foot_fl, dt);
        self.foot_fr.interpolate(&target.foot_fr, dt);
        self.foot_bl.interpolate(&target.foot_bl, dt);
        self.foot_br.interpolate(&target.foot_br, dt);
    }
}

pub struct SkeletonAttr {
    head_upper: (f32, f32),
    head_lower: (f32, f32),
    jaw: (f32, f32),
    tail: (f32, f32),
    torso_back: (f32, f32),
    torso_front: (f32, f32),
    ears: (f32, f32),
    leg_f: (f32, f32, f32),
    leg_b: (f32, f32, f32),
    feet_f: (f32, f32, f32),
    feet_b: (f32, f32, f32),
    height: f32,
}

impl<'a> std::convert::TryFrom<&'a comp::Body> for SkeletonAttr {
    type Error = ();

    fn try_from(body: &'a comp::Body) -> Result<Self, Self::Error> {
        match body {
            comp::Body::QuadrupedMedium(body) => Ok(SkeletonAttr::from(body)),
            _ => Err(()),
        }
    }
}

impl Default for SkeletonAttr {
    fn default() -> Self {
        Self {
            head_upper: (0.0, 0.0),
            head_lower: (0.0, 0.0),
            jaw: (0.0, 0.0),
            tail: (0.0, 0.0),
            torso_back: (0.0, 0.0),
            torso_front: (0.0, 0.0),
            ears: (0.0, 0.0),
            leg_f: (0.0, 0.0, 0.0),
            leg_b: (0.0, 0.0, 0.0),
            feet_f: (0.0, 0.0, 0.0),
            feet_b: (0.0, 0.0, 0.0),
            height: (0.0),
        }
    }
}

impl<'a> From<&'a comp::quadruped_medium::Body> for SkeletonAttr {
    fn from(body: &'a comp::quadruped_medium::Body) -> Self {
        use comp::quadruped_medium::Species::*;
        Self {
            head_upper: match (body.species, body.body_type) {
                (Grolgar, _) => (12.0, 16.0),
                (Saber, _) => (14.0, 12.0),
                (Tuskram, _) => (9.0, 12.0),
                (Lion, _) => (12.5, 14.0),
                (Tarasque, _) => (*HEAD_UPPER_Y+14.0, *HEAD_UPPER_Z+2.5),
                (Tiger, _) => (12.0, 19.0),
            },
            head_lower: match (body.species, body.body_type) {
                (Grolgar, _) => (-4.0, -7.0),
                (Saber, _) => (-6.0, 0.0),
                (Tuskram, _) => (-3.0, -1.0),
                (Lion, _) => (-5.0, -1.0),
                (Tiger, _) => (*HEAD_LOWER_X, *HEAD_LOWER_Z),
                (Tarasque, _) => (*HEAD_LOWER_Y+0.5, *HEAD_LOWER_Z-3.0),
                (Tiger, _) => (-5.0, -6.0),
            },
            jaw: match (body.species, body.body_type) {
                (Grolgar, _) => (3.0, -5.0),
                (Saber, _) => (2.0, -1.0),
                (Tuskram, _) => (2.0, -2.0),
                (Lion, _) => (2.0, -3.0),
                (Tarasque, _) => (4.0, -9.0),
                (Tarasque, _) => (*JAW_Y+0.5, *JAW_Z-11.0),
                (Tiger, _) => (4.0, -9.0),
            },
            tail: match (body.species, body.body_type) {
                (Grolgar, _) => (-6.0, -2.0),
                (Saber, _) => (-4.0, -2.0),
                (Tuskram, _) => (-6.0, -2.0),
                (Lion, _) => (-8.0, -6.0),
                (Tarasque, _) => (*TAIL_Y-4.5, *TAIL_Z+0.0),
                (Tiger, _) => (-7.0, -2.0),
            },
            torso_front: match (body.species, body.body_type) {
                (Grolgar, _) => (-7.0, 10.5),
                (Saber, _) => (-7.0, 9.5),
                (Tuskram, _) => (-7.0, 9.0),
                (Lion, _) => (-9.0, 9.0),
                (Tarasque, _) => (*TORSO_FRONT_Y+11.5, *TORSO_FRONT_Z+18.5),
                (Tiger, _) => (-7.0, 8.0),
            },
            torso_back: match (body.species, body.body_type) {
                (Grolgar, _) => (4.0, 11.0),
                (Saber, _) => (4.0, 9.0),
                (Tuskram, _) => (4.0, 9.0),
                (Lion, _) => (4.0, 10.0),
                (Tarasque, _) => (*TORSO_BACK_Y-20.0, *TORSO_BACK_Z-1.0),
                (Tiger, _) => (4.0, 9.0),
            },
            ears: match (body.species, body.body_type) {
                (Grolgar, _) => (-1.0, 5.0),
                (Saber, _) => (-1.0, 6.0),
                (Tuskram, _) => (10.0, 2.0),
                (Lion, _) => (-2.0, 4.0),
                (Tarasque, _) => (*EARS_Y-5.0, *EARS_Z+0.5),
                (Tiger, _) => (1.5, -2.0),
            },
            leg_f: match (body.species, body.body_type) {
                (Grolgar, _) => (5.0, 6.0, 2.0),
                (Saber, _) => (4.0, 6.0, 3.0),
                (Tuskram, _) => (4.0, 6.0, 4.5),
                (Lion, _) => (5.0, 6.0, 3.0),
                (Tarasque, _) => (*LEG_FRONT_X+8.0, *LEG_FRONT_Y+4.5, *LEG_FRONT_Z+12.5),
                (Tiger, _) => (4.0, 6.0, 3.0),
            },
            leg_b: match (body.species, body.body_type) {
                (Grolgar, _) => (5.0, -4.0, 3.0),
                (Saber, _) => (4.0, -6.0, 3.5),
                (Tuskram, _) => (4.0, -8.0, 5.5),
                (Lion, _) => (5.5, -8.0, 3.5),
                (Tarasque, _) => (*LEG_BACK_X+6.0, *LEG_BACK_Y-12.0, *LEG_BACK_Z+10.5),
                (Tiger, _) => (4.0, -8.0, 3.5),
            },
            feet_f: match (body.species, body.body_type) {
                (Grolgar, _) => (5.0, 6.0, 2.0),
                (Saber, _) => (4.0, 6.0, 3.0),
                (Tuskram, _) => (4.0, 6.0, 4.5),
                (Lion, _) => (5.0, 6.0, 3.0),
                (Tarasque, _) => (4.0, 6.0, 3.0),
                (Tarasque, _) => (*FEET_FRONT_X+2.0, *FEET_FRONT_Y+3.5, *FEET_FRONT_Z-3.5),
                (Tiger, _) => (4.0, 6.0, 3.0),
            },
            feet_b: match (body.species, body.body_type) {
                (Grolgar, _) => (5.0, -4.0, 3.0),
                (Saber, _) => (4.0, -6.0, 3.5),
                (Tuskram, _) => (4.0, -8.0, 5.5),
                (Lion, _) => (5.5, -8.0, 3.5),
                (Tarasque, _) => (*FEET_BACK_X+1.5, *FEET_BACK_Y+1.5, *FEET_BACK_Z-2.5),
                (Tiger, _) => (4.0, -8.0, 3.5),
            },
            height: match (body.species, body.body_type) {
                (Grolgar, _) => (1.2),
                (Saber, _) => (1.0),
                (Tuskram, _) => (1.0),
                (Lion, _) => (1.4),
                (Tarasque, _) => (1.1),
                (Tiger, _) => (1.0),
            },
        }
    }
}
