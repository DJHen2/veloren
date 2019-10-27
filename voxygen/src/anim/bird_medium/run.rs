use super::{
    super::{Animation, SkeletonAttr},
    BirdMediumSkeleton,
};
//use std::{f32::consts::PI, ops::Mul};
use vek::*;

pub struct RunAnimation;

impl Animation for RunAnimation {
    type Skeleton = BirdMediumSkeleton;
    type Dependency = (f32, f64);

    fn update_skeleton(
        skeleton: &Self::Skeleton,
        (_velocity, _global_time): Self::Dependency,
        _anim_time: f64,
        _rate: &mut f32,
        _skeleton_attr: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        next.head.offset = Vec3::new(0.0, 7.5, 15.0) / 11.0;
        next.head.ori = Quaternion::rotation_z(0.0) * Quaternion::rotation_x(0.0);
        next.head.scale = Vec3::one() / 10.88;

        next.torso.offset = Vec3::new(0.0, 4.5, 2.0);
        next.torso.ori = Quaternion::rotation_x(0.0);
        next.torso.scale = Vec3::one() * 1.01;

        next.tail.offset = Vec3::new(0.0, 3.1, -4.5);
        next.tail.ori = Quaternion::rotation_z(0.0);
        next.tail.scale = Vec3::one() * 0.98;

        next.wing_l.offset = Vec3::new(0.0, -13.0, 8.0) / 11.0;
        next.wing_l.ori = Quaternion::rotation_z(0.0) * Quaternion::rotation_x(0.0);
        next.wing_l.scale = Vec3::one() / 11.0;

        next.wing_r.offset = Vec3::new(0.0, -11.7, 11.0) / 11.0;
        next.wing_r.ori = Quaternion::rotation_y(0.0);
        next.wing_r.scale = Vec3::one() / 11.0;

        next.leg_l.offset = Vec3::new(0.0, 0.0, 12.0) / 11.0;
        next.leg_l.ori = Quaternion::rotation_y(0.0);
        next.leg_l.scale = Vec3::one() / 10.5;

        next.leg_r.offset = Vec3::new(0.0, 0.75, 5.25);
        next.leg_r.ori = Quaternion::rotation_x(0.0);
        next.leg_r.scale = Vec3::one() * 1.00;
        next
    }
}
