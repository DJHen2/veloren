use super::{super::Animation, DragonSkeleton, SkeletonAttr};
use std::{f32::consts::PI, ops::Mul};
use vek::*;

pub struct IdleAnimation;

#[const_tweaker::tweak(min = -40.0, max = 40.0, step = 0.1)]
const TEST_R: f32 = 2.5;
#[const_tweaker::tweak(min = -40.0, max = 40.0, step = 0.1)]
const TEST_L: f32 = -2.5;
#[const_tweaker::tweak(min = -40.0, max = 40.0, step = 0.1)]
const OFF1: f32 = -1.4;
#[const_tweaker::tweak(min = -40.0, max = 40.0, step = 0.1)]
const OFF2: f32 = -1.4;

impl Animation for IdleAnimation {
    type Dependency = f64;
    type Skeleton = DragonSkeleton;

    fn update_skeleton(
        skeleton: &Self::Skeleton,
        global_time: Self::Dependency,
        anim_time: f64,
        _rate: &mut f32,
        skeleton_attr: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        let ultra_slow = (anim_time as f32 * 1.0).sin();
        let wave_slow = (anim_time as f32 * 2.5).sin();
        let wave_slow_cos = (anim_time as f32 * 4.5).cos();

        let look = Vec2::new(
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

        next.head_upper.offset = Vec3::new(
            0.0,
            skeleton_attr.head_upper.0,
            skeleton_attr.head_upper.1 + ultra_slow * 0.20,
        ) * 1.05;
        next.head_upper.ori =
            Quaternion::rotation_z(0.8 * look.x) * Quaternion::rotation_x(0.8 * look.y);
        next.head_upper.scale = Vec3::one() * 1.05;

        next.head_lower.offset = Vec3::new(
            0.0,
            skeleton_attr.head_lower.0,
            skeleton_attr.head_lower.1 + ultra_slow * 0.20,
        ) * 1.05;
        next.head_lower.ori =
            Quaternion::rotation_z(0.8 * look.x) * Quaternion::rotation_x(0.8 * look.y);
        next.head_lower.scale = Vec3::one() * 1.05;

        next.jaw.offset = Vec3::new(
            0.0,
            skeleton_attr.jaw.0,
            skeleton_attr.jaw.1,
        ) * 1.05;
        next.jaw.ori = Quaternion::rotation_x(wave_slow * 0.05);
        next.jaw.scale = Vec3::one() * 0.98;

        next.chest_front.offset = Vec3::new(
            0.0,
            skeleton_attr.chest_front.0,
            skeleton_attr.chest_front.1,
        ) * 1.05;
        next.chest_front.ori = Quaternion::rotation_y(wave_slow * 0.03);
        next.chest_front.scale = Vec3::one() * 1.05;

        next.chest_rear.offset = Vec3::new(
            0.0,
            skeleton_attr.chest_rear.0,
            skeleton_attr.chest_rear.1,
        ) * 1.05;
        next.chest_rear.ori = Quaternion::rotation_y(wave_slow * 0.03);
        next.chest_rear.scale = Vec3::one() * 1.05;

        next.tail_front.offset = Vec3::new(0.0, skeleton_attr.tail_front.0, skeleton_attr.tail_front.1);
        next.tail_front.ori = Quaternion::rotation_x(wave_slow_cos * 0.03);
        next.tail_front.scale = Vec3::one();

        next.tail_rear.offset = Vec3::new(0.0, skeleton_attr.tail_rear.0, skeleton_attr.tail_rear.1);
        next.tail_rear.ori = Quaternion::rotation_x(wave_slow_cos * 0.03);
        next.tail_rear.scale = Vec3::one();

        next.wing_in_l.offset = Vec3::new(
            -skeleton_attr.wing_in.0,
            skeleton_attr.wing_in.1,
            skeleton_attr.wing_in.2,
        );
        next.wing_in_l.ori = Quaternion::rotation_y(0.2);
        next.wing_in_l.scale = Vec3::one() * 1.05;

        next.wing_in_r.offset = Vec3::new(
            skeleton_attr.wing_in.0,
            skeleton_attr.wing_in.1,
            skeleton_attr.wing_in.2,
        );
        next.wing_in_r.ori = Quaternion::rotation_y(-0.2);//.8
        next.wing_in_r.scale = Vec3::one() * 1.05;

        next.wing_out_l.offset = Vec3::new(
            -skeleton_attr.wing_out.0,
            skeleton_attr.wing_out.1,
            skeleton_attr.wing_out.2 + *OFF1,
        );
        next.wing_out_l.ori = Quaternion::rotation_y(-0.3);//2.0
        next.wing_out_l.scale = Vec3::one() * 1.05;

        next.wing_out_r.offset = Vec3::new(
            skeleton_attr.wing_out.0,
            skeleton_attr.wing_out.1,
            skeleton_attr.wing_out.2 + *OFF2,
        );
        next.wing_out_r.ori = Quaternion::rotation_y(0.3);
        next.wing_out_r.scale = Vec3::one() * 1.05;

        next.foot_fl.offset = Vec3::new(
            -skeleton_attr.feet_f.0,
            skeleton_attr.feet_f.1,
            skeleton_attr.feet_f.2,
        ) * 1.05;
        next.foot_fl.ori = Quaternion::rotation_x(0.0);
        next.foot_fl.scale = Vec3::one() * 1.05;

        next.foot_fr.offset = Vec3::new(
            skeleton_attr.feet_f.0,
            skeleton_attr.feet_f.1,
            skeleton_attr.feet_f.2,
        ) * 1.05;
        next.foot_fr.ori = Quaternion::rotation_x(0.0);
        next.foot_fr.scale = Vec3::one() * 1.05;

        next.foot_bl.offset = Vec3::new(
            -skeleton_attr.feet_b.0,
            skeleton_attr.feet_b.1,
            skeleton_attr.feet_b.2,
        ) * 1.05;
        next.foot_bl.ori = Quaternion::rotation_x(0.0);
        next.foot_bl.scale = Vec3::one() * 1.05;

        next.foot_br.offset = Vec3::new(
            skeleton_attr.feet_b.0,
            skeleton_attr.feet_b.1,
            skeleton_attr.feet_b.2,
        ) * 1.05;
        next.foot_br.ori = Quaternion::rotation_x(0.0);
        next.foot_br.scale = Vec3::one() * 1.05;

        next
    }
}
