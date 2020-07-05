use super::{super::Animation, QuadrupedLowSkeleton, SkeletonAttr};
use std::{f32::consts::PI, ops::Mul};
use vek::*;

pub struct RunAnimation;

impl Animation for RunAnimation {
    type Dependency = (f32, f64);
    type Skeleton = QuadrupedLowSkeleton;

    fn update_skeleton(
        skeleton: &Self::Skeleton,
        (_velocity, global_time): Self::Dependency,
        anim_time: f64,
        _rate: &mut f32,
        skeleton_attr: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        let lab = 0.1;

        let wave_ultra_slow_cos = (anim_time as f32 * 3.0 + PI).cos();
        let wave_slow = (anim_time as f32 * 4.5).sin();

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

        let center = (anim_time as f32 * lab as f32 + PI / 2.0).sin();
        let centeroffset = (anim_time as f32 * lab as f32 + PI * 1.5).sin();


        let short = (((5.0)
            / (2.5
                + 2.5 * ((anim_time as f32 *16.0* lab as f32+ PI * 0.25).sin()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 *16.0* lab as f32+ PI * 0.25).sin());



        let foothoril = (anim_time as f32 * 16.0 * lab as f32 + PI * 1.45).sin();
        let foothorir = (anim_time as f32 * 16.0 * lab as f32 + PI * (0.45)).sin();

        let footvertl = (anim_time as f32 * 16.0 * lab as f32).sin();
        let footvertr = (anim_time as f32 * 16.0 * lab as f32 + PI).sin();

        let footrotl = (((5.0)
            / (2.5
                + (2.5)
                    * ((anim_time as f32 * 16.0 * lab as f32 + PI * 1.4).sin())
                        .powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * 16.0 * lab as f32 + PI * 1.4).sin());

        let footrotr = (((5.0)
            / (1.0
                + (4.0)
                    * ((anim_time as f32 * 16.0 * lab as f32 + PI * 0.4).sin())
                        .powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * 16.0 * lab as f32 + PI * 0.4).sin());
///
        let foothorilb = (anim_time as f32 * 16.0 * lab as f32 + PI * 1.45).sin();
        let foothorirb = (anim_time as f32 * 16.0 * lab as f32 + PI * (0.45)).sin();

        let footvertlb = (anim_time as f32 * 16.0 * lab as f32).sin();
        let footvertrb = (anim_time as f32 * 16.0 * lab as f32 + PI*1.0).sin();

        let footrotlb = (((5.0)
            / (2.5
                + (2.5)
                    * ((anim_time as f32 * 16.0 * lab as f32 + PI * 1.4).sin())
                        .powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * 16.0 * lab as f32 + PI * 1.4).sin());

        let footrotrb = (((5.0)
            / (1.0
                + (4.0)
                    * ((anim_time as f32 * 16.0 * lab as f32 + PI * 0.4).sin())
                        .powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * 16.0 * lab as f32 + PI * 0.4).sin());








        let dragon_look = Vec2::new(
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

        next.head_upper.offset =
            Vec3::new(0.0, skeleton_attr.head_upper.0, skeleton_attr.head_upper.1);
        next.head_upper.ori =
            Quaternion::rotation_x(0.0) * Quaternion::rotation_z(short*-0.15);
        next.head_upper.scale = Vec3::one();

        next.head_lower.offset =
            Vec3::new(0.0, skeleton_attr.head_lower.0, skeleton_attr.head_lower.1);
        next.head_lower.ori = Quaternion::rotation_x(0.0)* Quaternion::rotation_z(short*-0.25);
        next.head_lower.scale = Vec3::one();

        next.jaw.offset = Vec3::new(
            0.0,
            skeleton_attr.jaw.0,
            skeleton_attr.jaw.1,
        );
        next.jaw.ori = Quaternion::rotation_x(0.0);
        next.jaw.scale = Vec3::one()*0.98;

        next.tail_front.offset = Vec3::new(
            0.0,
            skeleton_attr.tail_front.0,
            skeleton_attr.tail_front.1,
        );
        next.tail_front.ori = Quaternion::rotation_z(short*0.2)*Quaternion::rotation_y(short*0.15)*Quaternion::rotation_x(0.06);
        next.tail_front.scale = Vec3::one();

        next.tail_rear.offset = Vec3::new(
            0.0,
            skeleton_attr.tail_rear.0,
            skeleton_attr.tail_rear.1 + centeroffset * 0.6,
        );
        next.tail_rear.ori = Quaternion::rotation_z(short*0.3)*Quaternion::rotation_y(short*0.1)*Quaternion::rotation_x(-0.04);
        next.tail_rear.scale = Vec3::one();

        next.chest.offset =
            Vec3::new(0.0, skeleton_attr.chest.0, skeleton_attr.chest.1)/6.0;
        next.chest.ori = Quaternion::rotation_z(short*0.25)*Quaternion::rotation_y(short*0.15);
        next.chest.scale = Vec3::one()/6.0;

        next.foot_fl.offset = Vec3::new(
            -skeleton_attr.feet_f.0,
            skeleton_attr.feet_f.1 + foothoril * -3.5,
            skeleton_attr.feet_f.2 + ((footvertl * -0.6).max(-1.0)),
        );
        next.foot_fl.ori = Quaternion::rotation_x(-0.2 + footrotl * -0.5);
        next.foot_fl.scale = Vec3::one();

        next.foot_fr.offset = Vec3::new(
            skeleton_attr.feet_f.0,
            skeleton_attr.feet_f.1 + foothorir * -3.5,
            skeleton_attr.feet_f.2 + ((footvertr * -0.6).max(-1.0)),
        );
        next.foot_fr.ori = Quaternion::rotation_x(-0.2 + footrotr * -0.5);
        next.foot_fr.scale = Vec3::one();

        next.foot_bl.offset = Vec3::new(
            -skeleton_attr.feet_b.0,
            skeleton_attr.feet_b.1 + foothorilb * -3.5,
            skeleton_attr.feet_b.2 + ((footvertlb * -0.6).max(-1.0)),
        );
        next.foot_bl.ori = Quaternion::rotation_x(-0.2 + footrotlb * -0.5);
        next.foot_bl.scale = Vec3::one();

        next.foot_br.offset = Vec3::new(
            skeleton_attr.feet_b.0,
            skeleton_attr.feet_b.1 + foothorirb * -3.5,
            skeleton_attr.feet_b.2 + ((footvertrb * -0.6).max(-1.0)),
        );
        next.foot_br.ori = Quaternion::rotation_x(-0.2 + footrotrb * -0.5);
        next.foot_br.scale = Vec3::one();

        next
    }
}
