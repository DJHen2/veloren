use super::{super::Animation, CharacterSkeleton, SkeletonAttr};
use common::comp::item::ToolKind;
use std::{f32::consts::PI, ops::Mul};

use vek::*;

pub struct WieldAnimation;

impl Animation for WieldAnimation {
    type Dependency = (Option<ToolKind>, f32, f64);
    type Skeleton = CharacterSkeleton;

    fn update_skeleton(
        skeleton: &Self::Skeleton,
        (active_tool_kind, velocity, global_time): Self::Dependency,
        anim_time: f64,
        rate: &mut f32,
        skeleton_attr: &SkeletonAttr,
    ) -> Self::Skeleton {
        *rate = 1.0;
        let mut next = (*skeleton).clone();
        let slow_cos = (anim_time as f32 * 6.0 + PI).cos();
        let ultra_slow = (anim_time as f32 * 1.0 + PI).sin();
        let ultra_slow_cos = (anim_time as f32 * 3.0 + PI).cos();

        let wave = (anim_time as f32 * 16.0).sin();
        match active_tool_kind {
            //TODO: Inventory
            Some(ToolKind::Sword(_)) => {
                next.l_hand.offset = Vec3::new(-0.25, -5.0, -5.0);
                next.l_hand.ori = Quaternion::rotation_x(1.47) * Quaternion::rotation_y(-0.2);
                next.l_hand.scale = Vec3::one() * 1.04;
                next.r_hand.offset = Vec3::new(1.25, -5.5, -8.0);
                next.r_hand.ori = Quaternion::rotation_x(1.47) * Quaternion::rotation_y(0.3);
                next.r_hand.scale = Vec3::one() * 1.05;
                next.main.offset = Vec3::new(0.0, 0.0, -6.0);
                next.main.ori = Quaternion::rotation_x(-0.1)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(0.0);
                next.main.scale = Vec3::one();

                next.control.offset = Vec3::new(-7.0, 6.0, 6.0);
                next.control.ori = Quaternion::rotation_x(ultra_slow * 0.15)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(ultra_slow_cos * 0.08);
                next.control.scale = Vec3::one();
            },
            Some(ToolKind::Axe(_)) => {
                next.l_hand.offset = Vec3::new(-4.0, 3.0, 6.0);
                next.l_hand.ori = Quaternion::rotation_x(-0.3)
                    * Quaternion::rotation_z(3.14 - 0.3)
                    * Quaternion::rotation_y(-0.8);
                next.l_hand.scale = Vec3::one() * 1.08;
                next.r_hand.offset = Vec3::new(-2.5, 9.0, 4.0);
                next.r_hand.ori = Quaternion::rotation_x(-0.3)
                    * Quaternion::rotation_z(3.14 - 0.3)
                    * Quaternion::rotation_y(-0.8);
                next.r_hand.scale = Vec3::one() * 1.06;
                next.main.offset = Vec3::new(-6.0, 10.0, -1.0);
                next.main.ori = Quaternion::rotation_x(1.27)
                    * Quaternion::rotation_y(-0.3)
                    * Quaternion::rotation_z(-0.8);
                next.main.scale = Vec3::one();

                next.control.offset = Vec3::new(0.0, 0.0, 0.0);
                next.control.ori = Quaternion::rotation_x(ultra_slow_cos * 0.1 + 0.2)
                    * Quaternion::rotation_y(-0.3)
                    * Quaternion::rotation_z(ultra_slow * 0.1 + 0.0);
                next.control.scale = Vec3::one();
            },
            Some(ToolKind::Hammer(_)) => {
                next.l_hand.offset = Vec3::new(-7.0, 4.6, 7.5);
                next.l_hand.ori = Quaternion::rotation_x(0.3) * Quaternion::rotation_y(0.32);
                next.l_hand.scale = Vec3::one() * 1.08;
                next.r_hand.offset = Vec3::new(8.0, 5.75, 4.0);
                next.r_hand.ori = Quaternion::rotation_x(0.3) * Quaternion::rotation_y(0.22);
                next.r_hand.scale = Vec3::one() * 1.06;
                next.main.offset = Vec3::new(6.0, 7.0, 0.0);
                next.main.ori = Quaternion::rotation_x(0.3)
                    * Quaternion::rotation_y(-1.35)
                    * Quaternion::rotation_z(1.57);
                next.main.scale = Vec3::one();

                next.control.offset = Vec3::new(0.0, 0.0, 0.0);
                next.control.ori = Quaternion::rotation_x(ultra_slow * 0.15)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(ultra_slow_cos * 0.08);
                next.control.scale = Vec3::one();
            },
            Some(ToolKind::Staff(_)) => {
                next.l_hand.offset = Vec3::new(1.0, -2.0, -5.0);
                next.l_hand.ori = Quaternion::rotation_x(1.47) * Quaternion::rotation_y(-0.3);
                next.l_hand.scale = Vec3::one() * 1.05;
                next.r_hand.offset = Vec3::new(9.0, 1.0, 0.0);
                next.r_hand.ori = Quaternion::rotation_x(1.8)
                    * Quaternion::rotation_y(0.5)
                    * Quaternion::rotation_z(-0.27);
                next.r_hand.scale = Vec3::one() * 1.05;
                next.main.offset = Vec3::new(11.0, 9.0, 10.0);
                next.main.ori = Quaternion::rotation_x(-0.3)
                    * Quaternion::rotation_y(3.14 + 0.3)
                    * Quaternion::rotation_z(0.9);
                next.main.scale = Vec3::one();

                next.control.offset = Vec3::new(-7.0, 6.0, 6.0);
                next.control.ori = Quaternion::rotation_x(ultra_slow * 0.2)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(ultra_slow_cos * 0.1);
                next.control.scale = Vec3::one();
            },
            Some(ToolKind::Shield(_)) => {
                next.l_hand.offset = Vec3::new(-6.0, 3.5, 0.0);
                next.l_hand.ori = Quaternion::rotation_x(-0.3);
                next.l_hand.scale = Vec3::one() * 1.01;
                next.r_hand.offset = Vec3::new(-6.0, 3.0, -2.0);
                next.r_hand.ori = Quaternion::rotation_x(-0.3);
                next.r_hand.scale = Vec3::one() * 1.01;
                next.main.offset = Vec3::new(
                    -6.0 + skeleton_attr.weapon_x,
                    4.5 + skeleton_attr.weapon_y,
                    0.0,
                );
                next.main.ori = Quaternion::rotation_x(-0.3)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(0.0);
                next.main.scale = Vec3::one();
            },
            Some(ToolKind::Bow(_)) => {
                next.l_hand.offset = Vec3::new(1.0, -4.0, -1.0);
                next.l_hand.ori = Quaternion::rotation_x(1.20)
                    * Quaternion::rotation_y(-0.6)
                    * Quaternion::rotation_z(-0.3);
                next.l_hand.scale = Vec3::one() * 1.05;
                next.r_hand.offset = Vec3::new(3.0, -1.0, -5.0);
                next.r_hand.ori = Quaternion::rotation_x(1.20)
                    * Quaternion::rotation_y(-0.6)
                    * Quaternion::rotation_z(-0.3);
                next.r_hand.scale = Vec3::one() * 1.05;
                next.main.offset = Vec3::new(3.0, 2.0, -13.0);
                next.main.ori = Quaternion::rotation_x(-0.3)
                    * Quaternion::rotation_y(0.3)
                    * Quaternion::rotation_z(-0.6);
                next.main.scale = Vec3::one();

                next.control.offset = Vec3::new(-7.0, 6.0, 6.0);
                next.control.ori = Quaternion::rotation_x(ultra_slow * 0.2)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(ultra_slow_cos * 0.1);
                next.control.scale = Vec3::one();
            },
            Some(ToolKind::Dagger(_)) => {
                next.l_hand.offset = Vec3::new(-6.0, 3.5, 0.0);
                next.l_hand.ori = Quaternion::rotation_x(-0.3);
                next.l_hand.scale = Vec3::one() * 1.01;
                next.r_hand.offset = Vec3::new(-6.0, 3.0, -2.0);
                next.r_hand.ori = Quaternion::rotation_x(-0.3);
                next.r_hand.scale = Vec3::one() * 1.01;
                next.main.offset = Vec3::new(
                    -6.0 + skeleton_attr.weapon_x,
                    4.5 + skeleton_attr.weapon_y,
                    0.0,
                );
                next.main.ori = Quaternion::rotation_x(-0.3)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(0.0);
                next.main.scale = Vec3::one();
            },
            Some(ToolKind::Debug(_)) => {
                next.l_hand.offset = Vec3::new(-7.0, 4.0, 3.0);
                next.l_hand.ori = Quaternion::rotation_x(1.27 + wave * 0.25)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(0.0);
                next.l_hand.scale = Vec3::one() * 1.01;
                next.r_hand.offset = Vec3::new(7.0, 2.5, -1.25);
                next.r_hand.ori = Quaternion::rotation_x(1.27 + wave * 0.25)
                    * Quaternion::rotation_y(0.0)
                    * Quaternion::rotation_z(-0.3);
                next.r_hand.scale = Vec3::one() * 1.01;
                next.main.offset = Vec3::new(
                    5.0 + skeleton_attr.weapon_x,
                    8.75 + skeleton_attr.weapon_y,
                    -2.0,
                );
                next.main.ori = Quaternion::rotation_x(-0.3)
                    * Quaternion::rotation_y(-1.27)
                    * Quaternion::rotation_z(wave * -0.25);
                next.main.scale = Vec3::one();
            },
            _ => {},
        }

        if velocity > 0.5 {
            next.torso.offset = Vec3::new(0.0, 0.0, 0.0) * skeleton_attr.scaler;
            next.torso.ori = Quaternion::rotation_x(-0.2);
            next.torso.scale = Vec3::one() / 11.0 * skeleton_attr.scaler;

            next.l_control.offset = Vec3::new(0.0, 0.0, 0.0);
            next.l_control.ori = Quaternion::rotation_x(0.0);
            next.l_control.scale = Vec3::one();

            next.r_control.offset = Vec3::new(0.0, 0.0, 0.0);
            next.r_control.ori = Quaternion::rotation_x(0.0);
            next.r_control.scale = Vec3::one();
        } else {
            let head_look = Vec2::new(
                ((global_time + anim_time) as f32 / 10.0)
                    .floor()
                    .mul(7331.0)
                    .sin()
                    * 0.2,
                ((global_time + anim_time) as f32 / 10.0)
                    .floor()
                    .mul(1337.0)
                    .sin()
                    * 0.1,
            );
            next.head.offset = Vec3::new(
                0.0 + skeleton_attr.neck_right,
                -2.0 + skeleton_attr.neck_forward,
                skeleton_attr.neck_height + 14.0 + ultra_slow * 0.1,
            );
            next.head.ori =
                Quaternion::rotation_z(head_look.x) * Quaternion::rotation_x(head_look.y.abs());
            next.head.scale = Vec3::one() * skeleton_attr.head_scale;

            next.chest.offset = Vec3::new(0.0 + slow_cos * 0.5, 0.0, 7.0 + ultra_slow * 0.5);
            next.chest.ori =
                Quaternion::rotation_y(ultra_slow_cos * 0.04) * Quaternion::rotation_z(0.15);
            next.chest.scale = Vec3::one();

            next.belt.offset = Vec3::new(0.0, 0.0, -2.0);
            next.belt.ori =
                Quaternion::rotation_y(ultra_slow_cos * 0.03) * Quaternion::rotation_z(0.22);
            next.belt.scale = Vec3::one() * 1.02;

            next.shorts.offset = Vec3::new(0.0, 0.0, -5.0);
            next.shorts.ori = Quaternion::rotation_z(0.3);
            next.shorts.scale = Vec3::one();

            next.l_foot.offset = Vec3::new(-3.4, -2.5, 8.0);
            next.l_foot.ori = Quaternion::rotation_x(ultra_slow_cos * 0.035 - 0.2);
            next.l_foot.scale = Vec3::one();

            next.r_foot.offset = Vec3::new(3.4, 3.5, 8.0);
            next.r_foot.ori = Quaternion::rotation_x(ultra_slow * 0.035);
            next.r_foot.scale = Vec3::one();

            next.torso.offset = Vec3::new(0.0, 0.0, 0.1) * skeleton_attr.scaler;
            next.torso.ori = Quaternion::rotation_x(0.0);
            next.torso.scale = Vec3::one() / 11.0 * skeleton_attr.scaler;

            next.l_control.offset = Vec3::new(0.0, 0.0, 0.0);
            next.l_control.ori = Quaternion::rotation_x(0.0);
            next.l_control.scale = Vec3::one();

            next.r_control.offset = Vec3::new(0.0, 0.0, 0.0);
            next.r_control.ori = Quaternion::rotation_x(0.0);
            next.r_control.scale = Vec3::one();
        }

        next
    }
}
