use super::{
    super::{Animation, SkeletonAttr},
    FishSmallSkeleton,
};
use std::f32::consts::PI;
use vek::*;

pub struct JumpAnimation;

impl Animation for JumpAnimation {
    type Skeleton = FishSmallSkeleton;
    type Dependency = (f32, f64);

    fn update_skeleton(
        skeleton: &Self::Skeleton,
        _global_time: Self::Dependency,
        anim_time: f64,
        _rate: &mut f32,
        _skeleton_attr: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        let wave_ultra_slow = (anim_time as f32 * 1.0 + PI).sin();
        let wave_ultra_slow_cos = (anim_time as f32 * 1.0 + PI).cos();
        let wave_slow = (anim_time as f32 * 3.5 + PI).sin();
        let wave_slow_cos = (anim_time as f32 * 3.5 + PI).cos();

        next.cardinalfish_torso.offset = Vec3::new(0.0, 7.5, 15.0) / 11.0;
        next.cardinalfish_torso.ori = Quaternion::rotation_z(0.0) * Quaternion::rotation_x(0.0);
        next.cardinalfish_torso.scale = Vec3::one() / 10.88;

        next.cardinalfish_tail.offset = Vec3::new(0.0, 4.5 - wave_ultra_slow_cos * 0.12, 2.0);
        next.cardinalfish_tail.ori = Quaternion::rotation_x(0.0);
        next.cardinalfish_tail.scale = Vec3::one() * 1.01;

        next
    }
}
