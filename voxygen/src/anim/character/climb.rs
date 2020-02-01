use super::{super::Animation, CharacterSkeleton, SkeletonAttr};
use common::comp::item::Tool;
use vek::*;

pub struct ClimbAnimation;

impl Animation for ClimbAnimation {
    type Dependency = (Option<Tool>, Vec3<f32>, Vec3<f32>, f64);
    type Skeleton = CharacterSkeleton;

    fn update_skeleton(
        skeleton: &Self::Skeleton,
        (_active_tool_kind, velocity, _orientation, _global_time): Self::Dependency,
        anim_time: f64,
        rate: &mut f32,
        skeleton_attr: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        let speed = velocity.magnitude();
        *rate = speed;

        let constant = 1.0;
        let wave = (anim_time as f32 * constant as f32 * 1.5).sin();
        let wave_cos = (anim_time as f32 * constant as f32 * 1.5).cos();

        let wave_test = (((5.0)
            / (0.6 + 4.0 * ((anim_time as f32 * constant as f32 * 1.5).sin()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * constant as f32 * 1.5).sin());
        let wave_testc = (((5.0)
            / (0.6 + 4.0 * ((anim_time as f32 * constant as f32 * 1.5).cos()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * constant as f32 * 1.5).cos());

        next.head.offset = Vec3::new(
            0.0,
            -1.0 + skeleton_attr.neck_forward,
            skeleton_attr.neck_height + 18.5 + wave_cos * 1.3,
        );
        next.head.ori = Quaternion::rotation_z(wave * 0.1)
            * Quaternion::rotation_x(0.6)
            * Quaternion::rotation_y(wave_test * 0.1);
        next.head.scale = Vec3::one() * skeleton_attr.head_scale;

        next.chest.offset = Vec3::new(0.0, 1.0, 5.0 + wave_cos * 1.1);
        next.chest.ori = Quaternion::rotation_z(wave_test * 0.25)
            * Quaternion::rotation_x(-0.15)
            * Quaternion::rotation_y(wave_test * -0.12);
        next.chest.scale = Vec3::one();

        next.belt.offset = Vec3::new(0.0, 1.0, 3.5 + wave_cos * 1.1);
        next.belt.ori = Quaternion::rotation_z(wave_test * 0.25) * Quaternion::rotation_x(0.0);
        next.belt.scale = Vec3::one();

        next.shorts.offset = Vec3::new(0.0, 1.0, 1.0 + wave_cos * 1.1);
        next.shorts.ori = Quaternion::rotation_z(wave_test * 0.25)
            * Quaternion::rotation_x(0.1)
            * Quaternion::rotation_y(wave_test * 0.10);
        next.shorts.scale = Vec3::one();

        next.l_hand.offset = Vec3::new(-6.0, -0.25 + wave_testc * 1.5, 5.0 - wave_test * 4.0);
        next.l_hand.ori = Quaternion::rotation_x(2.2 + wave_testc * 0.5);
        next.l_hand.scale = Vec3::one();

        next.r_hand.offset = Vec3::new(6.0, -0.25 - wave_testc * 1.5, 5.0 + wave_test * 4.0);
        next.r_hand.ori = Quaternion::rotation_x(2.2 - wave_testc * 0.5);
        next.r_hand.scale = Vec3::one();

        next.l_foot.offset = Vec3::new(-3.4, 1.0, 6.0 + wave_test * 2.5);
        next.l_foot.ori = Quaternion::rotation_x(0.2 - wave_testc * 0.5);
        next.l_foot.scale = Vec3::one();

        next.r_foot.offset = Vec3::new(3.4, 1.0, 6.0 - wave_test * 2.5);
        next.r_foot.ori = Quaternion::rotation_x(0.2 + wave_testc * 0.5);
        next.r_foot.scale = Vec3::one();

        next.main.offset = Vec3::new(
            -7.0 + skeleton_attr.weapon_x,
            -5.0 + skeleton_attr.weapon_y,
            15.0,
        );
        next.main.ori =
            Quaternion::rotation_y(2.5) * Quaternion::rotation_z(1.57 + wave_cos * 0.25);
        next.main.scale = Vec3::one();

        next.l_shoulder.offset = Vec3::new(-5.0, 0.0, 4.7);
        next.l_shoulder.ori = Quaternion::rotation_x(wave_cos * 0.15);
        next.l_shoulder.scale = Vec3::one() * 1.1;

        next.r_shoulder.offset = Vec3::new(5.0, 0.0, 4.7);
        next.r_shoulder.ori = Quaternion::rotation_x(wave * 0.15);
        next.r_shoulder.scale = Vec3::one() * 1.1;

        next.glider.offset = Vec3::new(0.0, 5.0, 0.0);
        next.glider.ori = Quaternion::rotation_y(0.0);
        next.glider.scale = Vec3::one() * 0.0;

        next.lantern.offset = Vec3::new(0.0, 0.0, 0.0);
        next.lantern.ori = Quaternion::rotation_x(0.0);
        next.lantern.scale = Vec3::one() * 0.0;

        next.torso.offset = Vec3::new(0.0, -0.2 + wave * -0.08, 0.4) * skeleton_attr.scaler;
        next.torso.ori = Quaternion::rotation_x(0.0) * Quaternion::rotation_y(0.0);
        next.torso.scale = Vec3::one() / 11.0 * skeleton_attr.scaler;

        next
    }
}
