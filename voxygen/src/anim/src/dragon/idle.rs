use super::{
    super::{vek::*, Animation},
    DragonSkeleton, SkeletonAttr,
};
use std::{f32::consts::PI, ops::Mul};

pub struct IdleAnimation;

impl Animation for IdleAnimation {
    type Dependency = f64;
    type Skeleton = DragonSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"dragon_idle\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "dragon_idle")]
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        global_time: Self::Dependency,
        anim_time: f64,
        _rate: &mut f32,
        skeleton_attr: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        let ultra_slow = (anim_time as f32 * 1.0).sin();
        let slow = (anim_time as f32 * 2.5).sin();
        let slowalt = (anim_time as f32 * 2.5 + PI / 2.0).sin();

        let dragon_look = Vec2::new(
            ((global_time + anim_time) as f32 / 8.0)
                .floor()
                .mul(7331.0)
                .sin()
                * 0.5,
            ((global_time + anim_time) as f32 / 8.0)
                .floor()
                .mul(1337.0)
                .sin()
                * 0.25,
        );

        next.head_upper.position = Vec3::new(
            0.0,
            skeleton_attr.head_upper.0,
            skeleton_attr.head_upper.1 + ultra_slow * 0.20,
        );
        next.head_upper.orientation = Quaternion::rotation_z(0.8 * dragon_look.x)
            * Quaternion::rotation_x(0.8 * dragon_look.y);
        next.head_upper.scale = Vec3::one();

        next.head_lower.position = Vec3::new(
            0.0,
            skeleton_attr.head_lower.0,
            skeleton_attr.head_lower.1 + ultra_slow * 0.20,
        );
        next.head_lower.orientation = Quaternion::rotation_z(0.8 * dragon_look.x)
            * Quaternion::rotation_x(-0.2 + 0.8 * dragon_look.y);
        next.head_lower.scale = Vec3::one() * 1.05;

        next.jaw.position = Vec3::new(0.0, skeleton_attr.jaw.0, skeleton_attr.jaw.1);
        next.jaw.orientation = Quaternion::rotation_x(slow * 0.04);
        next.jaw.scale = Vec3::one() * 1.05;

        next.chest_front.position = Vec3::new(
            0.0,
            skeleton_attr.chest_front.0,
            skeleton_attr.chest_front.1,
        );
        next.chest_front.orientation = Quaternion::rotation_y(0.0);
        next.chest_front.scale = Vec3::one() * 1.05;

        next.chest_rear.position =
            Vec3::new(0.0, skeleton_attr.chest_rear.0, skeleton_attr.chest_rear.1);
        next.chest_rear.orientation = Quaternion::rotation_y(0.0);
        next.chest_rear.scale = Vec3::one() * 1.05;

        next.tail_front.position =
            Vec3::new(0.0, skeleton_attr.tail_front.0, skeleton_attr.tail_front.1);
        next.tail_front.orientation =
            Quaternion::rotation_z(slowalt * 0.10) * Quaternion::rotation_x(0.1);
        next.tail_front.scale = Vec3::one() * 0.98;

        next.tail_rear.position =
            Vec3::new(0.0, skeleton_attr.tail_rear.0, skeleton_attr.tail_rear.1);
        next.tail_rear.orientation =
            Quaternion::rotation_z(slowalt * 0.12) * Quaternion::rotation_x(0.05);
        next.tail_rear.scale = Vec3::one() * 0.98;

        next.foot_fl.position = Vec3::new(
            -skeleton_attr.feet_f.0,
            skeleton_attr.feet_f.1,
            skeleton_attr.feet_f.2,
        );
        next.foot_fl.orientation = Quaternion::rotation_x(0.0);
        next.foot_fl.scale = Vec3::one();

        next.foot_fr.position = Vec3::new(
            skeleton_attr.feet_f.0,
            skeleton_attr.feet_f.1,
            skeleton_attr.feet_f.2,
        );
        next.foot_fr.orientation = Quaternion::rotation_x(0.0);
        next.foot_fr.scale = Vec3::one();

        next.foot_bl.position = Vec3::new(
            -skeleton_attr.feet_b.0,
            skeleton_attr.feet_b.1,
            skeleton_attr.feet_b.2,
        );
        next.foot_bl.orientation = Quaternion::rotation_x(0.0);
        next.foot_bl.scale = Vec3::one();

        next.foot_br.position = Vec3::new(
            skeleton_attr.feet_b.0,
            skeleton_attr.feet_b.1,
            skeleton_attr.feet_b.2,
        );
        next.foot_br.orientation = Quaternion::rotation_x(0.0);
        next.foot_br.scale = Vec3::one();

        next.wing_in_l.position = Vec3::new(
            -skeleton_attr.wing_in.0,
            skeleton_attr.wing_in.1,
            skeleton_attr.wing_in.2,
        );
        next.wing_in_l.orientation = Quaternion::rotation_y(0.8 + slow * 0.02);
        next.wing_in_l.scale = Vec3::one();

        next.wing_in_r.position = Vec3::new(
            skeleton_attr.wing_in.0,
            skeleton_attr.wing_in.1,
            skeleton_attr.wing_in.2,
        );
        next.wing_in_r.orientation = Quaternion::rotation_y(-0.8 - slow * 0.02);
        next.wing_in_r.scale = Vec3::one();

        next.wing_out_l.position = Vec3::new(
            -skeleton_attr.wing_out.0,
            skeleton_attr.wing_out.1,
            skeleton_attr.wing_out.2,
        );
        next.wing_out_l.orientation = Quaternion::rotation_y(-2.0 + slow * 0.02);
        next.wing_out_l.scale = Vec3::one();

        next.wing_out_r.position = Vec3::new(
            skeleton_attr.wing_out.0,
            skeleton_attr.wing_out.1,
            skeleton_attr.wing_out.2,
        );
        next.wing_out_r.orientation = Quaternion::rotation_y(2.0 - slow * 0.02);
        next.wing_out_r.scale = Vec3::one();

        next
    }
}
