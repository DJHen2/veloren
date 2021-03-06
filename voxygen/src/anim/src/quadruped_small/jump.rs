use super::{
    super::{vek::*, Animation},
    QuadrupedSmallSkeleton, SkeletonAttr,
};

pub struct JumpAnimation;

impl Animation for JumpAnimation {
    type Dependency = (f32, f64);
    type Skeleton = QuadrupedSmallSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"quadruped_small_jump\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "quadruped_small_jump")]
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (_velocity, _global_time): Self::Dependency,
        _anim_time: f64,
        _rate: &mut f32,
        skeleton_attr: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        next.head.position = Vec3::new(0.0, skeleton_attr.head.0, skeleton_attr.head.1);
        next.head.orientation = Quaternion::rotation_z(-0.8) * Quaternion::rotation_x(0.5);
        next.head.scale = Vec3::one();

        next.chest.position = Vec3::new(0.0, skeleton_attr.chest.0, skeleton_attr.chest.1)
            * skeleton_attr.scaler
            / 11.0;
        next.chest.orientation = Quaternion::rotation_y(0.0);
        next.chest.scale = Vec3::one() * skeleton_attr.scaler / 11.0;

        next.leg_fl.position = Vec3::new(
            -skeleton_attr.feet_f.0,
            skeleton_attr.feet_f.1,
            skeleton_attr.feet_f.2,
        );
        next.leg_fl.orientation = Quaternion::rotation_x(0.0);
        next.leg_fl.scale = Vec3::one();

        next.leg_fr.position = Vec3::new(
            skeleton_attr.feet_f.0,
            skeleton_attr.feet_f.1,
            skeleton_attr.feet_f.2,
        );
        next.leg_fr.orientation = Quaternion::rotation_x(0.0);
        next.leg_fr.scale = Vec3::one();

        next.leg_bl.position = Vec3::new(
            -skeleton_attr.feet_b.0,
            skeleton_attr.feet_b.1,
            skeleton_attr.feet_b.2,
        );
        next.leg_bl.orientation = Quaternion::rotation_x(0.0);
        next.leg_bl.scale = Vec3::one();

        next.leg_br.position = Vec3::new(
            skeleton_attr.feet_b.0,
            skeleton_attr.feet_b.1,
            skeleton_attr.feet_b.2,
        );
        next.leg_br.orientation = Quaternion::rotation_x(0.0);
        next.leg_br.scale = Vec3::one();

        next.tail.position = Vec3::new(0.0, skeleton_attr.tail.0, skeleton_attr.tail.1);
        next.tail.orientation = Quaternion::rotation_x(-0.3);
        next.tail.scale = Vec3::one();
        next
    }
}
