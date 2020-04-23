use vek::*;
use rand::prelude::*;
use common::{
    terrain::{Block, BlockKind},
    vol::Vox,
};
use crate::util::{RandomField, Sampler};
use super::{
    Archetype,
    BlockMask,
    super::skeleton::*,
};

pub struct House {
    roof_color: Rgb<u8>,
    noise: RandomField,
    roof_ribbing: bool,
    roof_ribbing_diagonal: bool,
}

enum Pillar {
    None,
    Chimney(i32),
    Tower(i32),
}

enum RoofStyle {
    Hip,
    Gable,
    Rounded,
}

enum StoreyFill {
    None,
    Upper,
    All,
}

impl StoreyFill {
    fn has_lower(&self) -> bool { if let StoreyFill::All = self { true } else { false } }
    fn has_upper(&self) -> bool { if let StoreyFill::None = self { false } else { true } }
}

pub struct Attr {
    central_supports: bool,
    storey_fill: StoreyFill,
    roof_style: RoofStyle,
    mansard: i32,
    pillar: Pillar,
}

impl Attr {
    fn generate<R: Rng>(rng: &mut R, locus: i32) -> Self {
        Self {
            central_supports: rng.gen(),
            storey_fill: match rng.gen_range(0, 2) {
                //0 => StoreyFill::None,
                0 => StoreyFill::Upper,
                _ => StoreyFill::All,
            },
            roof_style: match rng.gen_range(0, 3) {
                0 => RoofStyle::Hip,
                1 => RoofStyle::Gable,
                _ => RoofStyle::Rounded,
            },
            mansard: rng.gen_range(-7, 4).max(0),
            pillar: match rng.gen_range(0, 4) {
                0 => Pillar::Chimney(9 + locus + rng.gen_range(0, 4)),
                _ => Pillar::None,
            },
        }
    }
}

impl Archetype for House {
    type Attr = Attr;

    fn generate<R: Rng>(rng: &mut R) -> (Self, Skeleton<Self::Attr>) {
        let len = rng.gen_range(-8, 24).clamped(0, 20);
        let locus = 6 + rng.gen_range(0, 5);
        let branches_per_side = 1 + len as usize / 20;
        let skel = Skeleton {
            offset: -rng.gen_range(0, len + 7).clamped(0, len),
            ori: if rng.gen() { Ori::East } else { Ori::North },
            root: Branch {
                len,
                attr: Attr {
                    storey_fill: StoreyFill::All,
                    mansard: 0,
                    pillar: match rng.gen_range(0, 3) {
                        0 => Pillar::Chimney(9 + locus + rng.gen_range(0, 4)),
                        1 => Pillar::Tower(15 + locus + rng.gen_range(0, 4)),
                        _ => Pillar::None,
                    },
                    ..Attr::generate(rng, locus)
                },
                locus,
                border: 4,
                children: [1, -1]
                    .iter()
                    .map(|flip| (0..branches_per_side).map(move |i| (i, *flip)))
                    .flatten()
                    .filter_map(|(i, flip)| if rng.gen() {
                        Some((i as i32 * len / (branches_per_side - 1).max(1) as i32, Branch {
                            len: rng.gen_range(8, 16) * flip,
                            attr: Attr::generate(rng, locus),
                            locus: (6 + rng.gen_range(0, 3)).min(locus),
                            border: 4,
                            children: Vec::new(),
                        }))
                    } else {
                        None
                    })
                    .collect(),
            },
        };

        let this = Self {
            roof_color: Rgb::new(
                rng.gen_range(50, 200),
                rng.gen_range(50, 200),
                rng.gen_range(50, 200),
            ),
            noise: RandomField::new(rng.gen()),
            roof_ribbing: rng.gen(),
            roof_ribbing_diagonal: rng.gen(),
        };

        (this, skel)
    }

    fn draw(
        &self,
        dist: i32,
        bound_offset: Vec2<i32>,
        center_offset: Vec2<i32>,
        z: i32,
        branch: &Branch<Self::Attr>,
    ) -> BlockMask {
        let profile = Vec2::new(bound_offset.x, z);

        let make_block = |r, g, b| {
            let nz = self.noise.get(Vec3::new(center_offset.x, center_offset.y, z * 8));
            BlockMask::new(Block::new(BlockKind::Normal, Rgb::new(r, g, b) + (nz & 0x0F) as u8 - 8), 2)
        };

        let facade_layer = 3;
        let structural_layer = facade_layer + 1;
        let foundation_layer = structural_layer + 1;
        let floor_layer = foundation_layer + 1;

        let foundation = make_block(100, 100, 100).with_priority(foundation_layer);
        let log = make_block(60, 45, 30);
        let floor = make_block(100, 75, 50);
        let wall = make_block(200, 180, 150).with_priority(facade_layer);
        let roof = make_block(self.roof_color.r, self.roof_color.g, self.roof_color.b).with_priority(facade_layer);
        let empty = BlockMask::nothing();
        let internal = BlockMask::new(Block::empty(), structural_layer);
        let fire = BlockMask::new(Block::new(BlockKind::Ember, Rgb::white()), foundation_layer);

        let ceil_height = 6;
        let lower_width = branch.locus - 1;
        let upper_width = branch.locus;
        let width = if profile.y >= ceil_height { upper_width } else { lower_width };
        let foundation_height = 0 - (dist - width - 1).max(0);
        let roof_top = 8 + width;

        if let Pillar::Chimney(chimney_top) = branch.attr.pillar {
            // Chimney shaft
            if center_offset.map(|e| e.abs()).reduce_max() == 0 && profile.y >= foundation_height + 1 {
                return if profile.y == foundation_height + 1 {
                    fire
                } else {
                    internal
                };
            }

            // Chimney
            if center_offset.map(|e| e.abs()).reduce_max() <= 1 && profile.y < chimney_top {
                // Fireplace
                if center_offset.product() == 0 && profile.y > foundation_height + 1 && profile.y <= foundation_height + 3 {
                    return internal;
                } else {
                    return foundation;
                }
            }
        }

        if profile.y <= foundation_height && dist < width + 3 { // Foundations
            if branch.attr.storey_fill.has_lower() {
                if dist == width - 1 { // Floor lining
                    return log.with_priority(floor_layer);
                } else if dist < width - 1 && profile.y == foundation_height { // Floor
                    return floor.with_priority(floor_layer);
                }
            }

            if dist < width && profile.y < foundation_height && profile.y >= foundation_height - 3 { // Basement
                return internal;
            } else {
                return foundation.with_priority(1);
            }
        }

        // Roofs and walls
        let do_roof_wall = |profile: Vec2<i32>, width, dist, bound_offset: Vec2<i32>, roof_top, mansard| {
            // Roof

            let (roof_profile, roof_dist) = match &branch.attr.roof_style {
                RoofStyle::Hip => (Vec2::new(dist, profile.y), dist),
                RoofStyle::Gable => (profile, dist),
                RoofStyle::Rounded => {
                    let circular_dist = (bound_offset.map(|e| e.pow(4) as f32).sum().powf(0.25) - 0.5).ceil() as i32;
                    (Vec2::new(circular_dist, profile.y), circular_dist)
                },
            };

            let roof_level = roof_top - roof_profile.x.max(mansard);

            if profile.y > roof_level {
                return None;
            }

            // Roof
            if profile.y == roof_level && roof_dist <= width + 2 {
                let is_ribbing = ((profile.y - ceil_height) % 3 == 0 && self.roof_ribbing)
                    || (bound_offset.x == bound_offset.y && self.roof_ribbing_diagonal);
                if (roof_profile.x == 0 && mansard == 0) || roof_dist == width + 2 || is_ribbing { // Eaves
                    return Some(log);
                } else {
                    return Some(roof);
                }
            }

            // Wall

            if dist == width && profile.y < roof_level {
                if bound_offset.x == bound_offset.y || profile.y == ceil_height { // Support beams
                    return Some(log);
                } else if !branch.attr.storey_fill.has_lower() && profile.y < ceil_height {
                    return Some(empty);
                } else if !branch.attr.storey_fill.has_upper() {
                    return Some(empty);
                } else {
                    let frame_bounds = if profile.y >= ceil_height {
                        Aabr {
                            min: Vec2::new(-1, ceil_height + 2),
                            max: Vec2::new(1, ceil_height + 5),
                        }
                    } else {
                        Aabr {
                            min: Vec2::new(2, foundation_height + 2),
                            max: Vec2::new(width - 2, ceil_height - 2),
                        }
                    };
                    let window_bounds = Aabr {
                        min: (frame_bounds.min + 1).map2(frame_bounds.center(), |a, b| a.min(b)),
                        max: (frame_bounds.max - 1).map2(frame_bounds.center(), |a, b| a.max(b)),
                    };

                    // Window
                    if (frame_bounds.size() + 1).reduce_min() > 2 { // Window frame is large enough for a window
                        let surface_pos = Vec2::new(bound_offset.x, profile.y);
                        if window_bounds.contains_point(surface_pos) {
                            return Some(internal);
                        } else if frame_bounds.contains_point(surface_pos) {
                            return Some(log.with_priority(3));
                        };
                    }

                    // Wall
                    return Some(if branch.attr.central_supports && profile.x == 0 { // Support beams
                        log.with_priority(structural_layer)
                    } else {
                        wall
                    });
                }
            }

            if dist < width { // Internals
                if profile.y == ceil_height {
                    if profile.x == 0 {// Rafters
                        return Some(log);
                    } else if branch.attr.storey_fill.has_upper() { // Ceiling
                        return Some(floor);
                    }
                } else if (!branch.attr.storey_fill.has_lower() && profile.y < ceil_height)
                    || (!branch.attr.storey_fill.has_upper() && profile.y >= ceil_height)
                {
                    return Some(empty);
                } else {
                    return Some(internal);
                }
            }

            None
        };

        let mut cblock = empty;

        if let Some(block) = do_roof_wall(profile, width, dist, bound_offset, roof_top, branch.attr.mansard) {
            cblock = cblock.resolve_with(block);
        }

        if let Pillar::Tower(tower_top) = branch.attr.pillar {
            let profile = Vec2::new(center_offset.x.abs(), profile.y);
            let dist = center_offset.map(|e| e.abs()).reduce_max();

            if let Some(block) = do_roof_wall(profile, 4, dist, center_offset.map(|e| e.abs()), tower_top, branch.attr.mansard) {
                 cblock = cblock.resolve_with(block);
            }
        }

        cblock
    }
}
