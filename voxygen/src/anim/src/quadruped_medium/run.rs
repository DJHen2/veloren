use super::{super::Animation, QuadrupedMediumSkeleton, SkeletonAttr};
use std::{f32::consts::PI, ops::Mul};
use vek::*;

pub struct RunAnimation;

impl Animation for RunAnimation {
    type Dependency = (f32, f64);
    type Skeleton = QuadrupedMediumSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"quadruped_medium_run\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "quadruped_medium_run")]
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (_velocity, global_time): Self::Dependency,
        anim_time: f64,
        _rate: &mut f32,
        skeleton_attr: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        let lab = 14;
        let vertlf = (anim_time as f32 * lab as f32 + PI * 1.8).sin().max(0.15);
        let vertrfoffset = (anim_time as f32 * lab as f32 + PI * 0.80).sin().max(0.15);
        let vertlboffset = (anim_time as f32 * lab as f32).sin().max(0.15);
        let vertrb = (anim_time as f32 * lab as f32 + PI).sin().max(0.15);

        let horilf = (anim_time as f32 * lab as f32 + PI * 1.2).sin();
        let horirfoffset = (anim_time as f32 * lab as f32 + PI * 0.20).sin();
        let horilboffset = (anim_time as f32 * lab as f32 + PI * 1.4).sin();
        let horirb = (anim_time as f32 * lab as f32 + PI * 0.4).sin();

        let vertchest = (anim_time as f32 * lab as f32 + PI * 0.3).sin().max(0.2);
        let horichest = (anim_time as f32 * lab as f32 + PI * 0.8).sin();
        let verthead = (anim_time as f32 * lab as f32 + PI * 0.3).sin();

        let wolf_look = Vec2::new(
            ((global_time + anim_time) as f32 / 4.0)
                .floor()
                .mul(7331.0)
                .sin()
                * 0.25,
            ((global_time + anim_time) as f32 / 4.0)
                .floor()
                .mul(1337.0)
                .sin()
                * 0.125,
        );

        let wave_ultra_slow = (anim_time as f32 * 1.0 + PI).sin();
        let wave_ultra_slow_cos = (anim_time as f32 * 1.0 + PI).cos();
        let wave_slow = (anim_time as f32 * 3.5 + PI).sin();
        let wave_slow_cos = (anim_time as f32 * 3.5 + PI).cos();

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
        let tailmove = Vec2::new(
            ((global_time + anim_time) as f32 / 2.0)
                .floor()
                .mul(7331.0)
                .sin()
                * 0.25,
            ((global_time + anim_time) as f32 / 2.0)
                .floor()
                .mul(1337.0)
                .sin()
                * 0.125,
        );

        next.head_upper.offset = Vec3::new(
            0.0,
            skeleton_attr.head_upper.0 + horichest * 1.8,
            skeleton_attr.head_upper.1 + verthead * -1.8,
        );
        next.head_upper.ori =
            Quaternion::rotation_x(wolf_look.y) * Quaternion::rotation_z(wolf_look.x);
        next.head_upper.scale = Vec3::one();

        next.head_lower.offset = Vec3::new(
            0.0,
            skeleton_attr.head_lower.0 + horichest * 0.8,
            skeleton_attr.head_lower.1 + vertchest * -0.8 + verthead * 1.8,
        );
        next.head_lower.ori = Quaternion::rotation_z(0.0);
        next.head_lower.scale = Vec3::one() * 0.98;

        next.jaw.offset = Vec3::new(0.0, skeleton_attr.jaw.0, skeleton_attr.jaw.1);
        next.jaw.ori = Quaternion::rotation_x(0.0);
        next.jaw.scale = Vec3::one() * 1.02;

        next.tail.offset = Vec3::new(0.0, skeleton_attr.tail.0, skeleton_attr.tail.1);
        next.tail.ori = Quaternion::rotation_x(0.0);
        next.tail.scale = Vec3::one();

        next.torso_front.offset = Vec3::new(
            0.0,
            skeleton_attr.torso_front.0 + horichest * 2.5,
            skeleton_attr.torso_front.1 + vertchest * -3.2 + 1.0,
        ) / 11.0;
        next.torso_front.ori = Quaternion::rotation_y(horichest * -0.09);
        next.torso_front.scale = Vec3::one() * 0.98 / 11.0;

        next.torso_back.offset = Vec3::new(
            0.0,
            skeleton_attr.torso_back.0 + horichest * 2.9,
            skeleton_attr.torso_back.1 + vertchest * -3.6 + 1.0,
        );
        next.torso_back.ori = Quaternion::rotation_y(horichest * -0.12);
        next.torso_back.scale = Vec3::one();

        next.ears.offset = Vec3::new(0.0, skeleton_attr.ears.0, skeleton_attr.ears.1);
        next.ears.ori = Quaternion::rotation_x(0.0);
        next.ears.scale = Vec3::one() * 1.02;



        next.leg_fl.offset = Vec3::new(
            -skeleton_attr.leg_f.0,
            skeleton_attr.leg_f.1,
            skeleton_attr.leg_f.2,
        ) / 11.0;
        next.leg_fl.ori = Quaternion::rotation_x(horilf * 0.4);
        next.leg_fl.scale = Vec3::one() / 11.0;

        next.leg_fr.offset = Vec3::new(
            skeleton_attr.leg_f.0,
            skeleton_attr.leg_f.1,
            skeleton_attr.leg_f.2,
        ) / 11.0;
        next.leg_fr.ori = Quaternion::rotation_x(horilf * 0.4);
        next.leg_fr.scale = Vec3::one() / 11.0;

        next.leg_bl.offset = Vec3::new(
            -skeleton_attr.leg_b.0,
            skeleton_attr.leg_b.1,
            skeleton_attr.leg_b.2,
        ) / 11.0;
        next.leg_bl.ori = Quaternion::rotation_x(horirb * 0.55);
        next.leg_bl.scale = Vec3::one() / 11.0;

        next.leg_br.offset = Vec3::new(
            skeleton_attr.leg_b.0,
            skeleton_attr.leg_b.1 + horirb * 3.0,
            skeleton_attr.leg_b.2,
        ) / 11.0;
        next.leg_br.ori = Quaternion::rotation_x(horirb * 0.55);
        next.leg_br.scale = Vec3::one() / 11.0;

        next.foot_fl.offset = Vec3::new(
            -skeleton_attr.feet_f.0,
            skeleton_attr.feet_f.1,
            skeleton_attr.feet_f.2,
        );
        next.foot_fl.ori = Quaternion::rotation_x(horilf * 0.6);
        next.foot_fl.scale = Vec3::one();

        next.foot_fr.offset = Vec3::new(
            skeleton_attr.feet_f.0,
            skeleton_attr.feet_f.1,
            skeleton_attr.feet_f.2,
        );
        next.foot_fr.ori = Quaternion::rotation_x(horilf * 0.6);
        next.foot_fr.scale = Vec3::one();

        next.foot_bl.offset = Vec3::new(
            -skeleton_attr.feet_b.0,
            skeleton_attr.feet_b.1,
            skeleton_attr.feet_b.2,
        );
        next.foot_bl.ori = Quaternion::rotation_x(horirb * 0.35);
        next.foot_bl.scale = Vec3::one();

        next.foot_br.offset = Vec3::new(
            skeleton_attr.feet_b.0,
            skeleton_attr.feet_b.1,
            skeleton_attr.feet_b.2,
        );
        next.foot_br.ori = Quaternion::rotation_x(horirb * 0.35);
        next.foot_br.scale = Vec3::one();

        next
    }
}
