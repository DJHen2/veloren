use crate::{all::ForestKind, sim::LocationInfo, util::Sampler, World, CONFIG};
use common::{terrain::TerrainChunkSize, vol::VolSize};
use noise::NoiseFn;
use std::{
    f32,
    ops::{Add, Div, Mul, Neg, Sub},
};
use vek::*;

pub struct ColumnGen<'a> {
    world: &'a World,
}

impl<'a> ColumnGen<'a> {
    pub fn new(world: &'a World) -> Self {
        Self { world }
    }
}

impl<'a> Sampler for ColumnGen<'a> {
    type Index = Vec2<i32>;
    type Sample = Option<ColumnSample<'a>>;

    fn get(&self, wpos: Vec2<i32>) -> Option<ColumnSample<'a>> {
        let wposf = wpos.map(|e| e as f64);
        let chunk_pos = wpos.map2(Vec2::from(TerrainChunkSize::SIZE), |e, sz: u32| {
            e / sz as i32
        });

        let sim = self.world.sim();

        let turb = Vec2::new(
            sim.gen_ctx.turb_x_nz.get((wposf.div(48.0)).into_array()) as f32,
            sim.gen_ctx.turb_y_nz.get((wposf.div(48.0)).into_array()) as f32,
        ) * 12.0;
        let wposf_turb = wposf + turb.map(|e| e as f64);

        let alt_base = sim.get_interpolated(wpos, |chunk| chunk.alt_base)?;
        let chaos = sim.get_interpolated(wpos, |chunk| chunk.chaos)?;
        let temp = sim.get_interpolated(wpos, |chunk| chunk.temp)?;
        let dryness = sim.get_interpolated(wpos, |chunk| chunk.dryness)?;
        let rockiness = sim.get_interpolated(wpos, |chunk| chunk.rockiness)?;
        let tree_density = sim.get_interpolated(wpos, |chunk| chunk.tree_density)?;
        let spawn_rate = sim.get_interpolated(wpos, |chunk| chunk.spawn_rate)?;

        let sim_chunk = sim.get(chunk_pos)?;

        const RIVER_PROPORTION: f32 = 0.025;

        /*
        let river = dryness
            .abs()
            .neg()
            .add(RIVER_PROPORTION)
            .div(RIVER_PROPORTION)
            .max(0.0)
            .mul((1.0 - (chaos - 0.15) * 20.0).max(0.0).min(1.0));
        */
        let river = 0.0;

        let cliff_hill =
            (sim.gen_ctx.small_nz.get((wposf.div(128.0)).into_array()) as f32).mul(16.0);

        let riverless_alt = sim.get_interpolated(wpos, |chunk| chunk.alt)?
            + (sim.gen_ctx.small_nz.get((wposf.div(256.0)).into_array()) as f32)
                .abs()
                .mul(chaos.max(0.15))
                .mul(64.0);

        let is_cliffs = sim_chunk.is_cliffs;
        let near_cliffs = sim_chunk.near_cliffs;

        let alt = riverless_alt
            - (1.0 - river)
                .mul(f32::consts::PI)
                .cos()
                .add(1.0)
                .mul(0.5)
                .mul(24.0);

        let water_level = riverless_alt - 4.0 - 5.0 * chaos;

        let rock = (sim.gen_ctx.small_nz.get(
            Vec3::new(wposf.x, wposf.y, alt as f64)
                .div(100.0)
                .into_array(),
        ) as f32)
            .mul(rockiness)
            .sub(0.4)
            .max(0.0)
            .mul(8.0);

        let wposf3d = Vec3::new(wposf.x, wposf.y, alt as f64);

        let marble_small = (sim.gen_ctx.hill_nz.get((wposf3d.div(3.0)).into_array()) as f32)
            .add(1.0)
            .mul(0.5);
        let marble = (sim.gen_ctx.hill_nz.get((wposf3d.div(48.0)).into_array()) as f32)
            .mul(0.75)
            .add(1.0)
            .mul(0.5)
            .add(marble_small.sub(0.5).mul(0.25));

        // Colours
        let cold_grass = Rgb::new(0.0, 0.25, 0.13);
        let warm_grass = Rgb::new(0.18, 0.65, 0.0);
        let cold_stone = Rgb::new(0.55, 0.7, 0.75);
        let warm_stone = Rgb::new(0.65, 0.65, 0.35);
        let beach_sand = Rgb::new(0.9, 0.85, 0.3);
        let desert_sand = Rgb::new(1.0, 0.7, 0.15);
        let snow = Rgb::broadcast(1.0);

        let dirt = Lerp::lerp(Rgb::new(0.2, 0.1, 0.05), Rgb::new(0.4, 0.25, 0.0), marble);
        let cliff = Rgb::lerp(cold_stone, warm_stone, marble);

        let grass = Rgb::lerp(cold_grass, warm_grass, marble);
        let sand = Rgb::lerp(beach_sand, desert_sand, marble);

        let tropical = Rgb::lerp(
            grass,
            Rgb::new(0.85, 0.4, 0.2),
            marble_small.sub(0.5).mul(0.2).add(0.75),
        );

        let ground = Rgb::lerp(
            Rgb::lerp(
                snow,
                grass,
                temp.sub(CONFIG.snow_temp)
                    .sub((marble - 0.5) * 0.05)
                    .mul(256.0),
            ),
            Rgb::lerp(tropical, sand, temp.sub(CONFIG.desert_temp).mul(32.0)),
            temp.sub(CONFIG.tropical_temp).mul(16.0),
        );

        // Work out if we're on a path or near a town
        let dist_to_path = match &sim_chunk.location {
            Some(loc) => {
                let this_loc = &sim.locations[loc.loc_idx];
                this_loc
                    .neighbours
                    .iter()
                    .map(|j| {
                        let other_loc = &sim.locations[*j];

                        // Find the two location centers
                        let near_0 = this_loc.center.map(|e| e as f32);
                        let near_1 = other_loc.center.map(|e| e as f32);

                        // Calculate distance to path between them
                        (0.0 + (near_1.y - near_0.y) * wposf_turb.x as f32
                            - (near_1.x - near_0.x) * wposf_turb.y as f32
                            + near_1.x * near_0.y
                            - near_0.x * near_1.y)
                            .abs()
                            .div(near_0.distance(near_1))
                    })
                    .filter(|x| x.is_finite())
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(f32::INFINITY)
            }
            None => f32::INFINITY,
        };

        let on_path = dist_to_path < 5.0 && !sim_chunk.near_cliffs; // || near_0.distance(wposf_turb.map(|e| e as f32)) < 150.0;

        let (alt, ground) = if on_path {
            (alt - 1.0, dirt)
        } else {
            (alt, ground)
        };

        // Cities
        // TODO: In a later MR
        let building = match &sim_chunk.location {
            Some(loc) => {
                let loc = &sim.locations[loc.loc_idx];
                let rpos = wposf.map2(loc.center, |a, b| a as f32 - b as f32) / 256.0 + 0.5;

                if rpos.map(|e| e >= 0.0 && e < 1.0).reduce_and() {
                    (loc.settlement
                        .get_at(rpos)
                        .map(|b| b.seed % 20 + 10)
                        .unwrap_or(0)) as f32
                } else {
                    0.0
                }
            }
            None => 0.0,
        };

        let alt = alt + building;

        // Caves
        let cave_at = |wposf: Vec2<f64>| {
            (sim.gen_ctx.cave_0_nz.get(
                Vec3::new(wposf.x, wposf.y, alt as f64 * 8.0)
                    .div(800.0)
                    .into_array(),
            ) as f32)
                .powf(2.0)
                .neg()
                .add(1.0)
                .mul((1.15 - chaos).min(1.0))
        };
        let cave_xy = cave_at(wposf);
        let cave_alt = alt - 24.0
            + (sim
                .gen_ctx
                .cave_1_nz
                .get(Vec2::new(wposf.x, wposf.y).div(48.0).into_array()) as f32)
                * 8.0
            + (sim
                .gen_ctx
                .cave_1_nz
                .get(Vec2::new(wposf.x, wposf.y).div(500.0).into_array()) as f32)
                .add(1.0)
                .mul(0.5)
                .powf(15.0)
                .mul(150.0);

        Some(ColumnSample {
            alt,
            chaos,
            water_level,
            river,
            surface_color: Rgb::lerp(
                sand,
                // Land
                Rgb::lerp(
                    ground,
                    // Mountain
                    Rgb::lerp(
                        cliff,
                        snow,
                        (alt - CONFIG.sea_level
                            - 0.4 * CONFIG.mountain_scale
                            - alt_base
                            - temp * 96.0
                            - marble * 24.0)
                            / 12.0,
                    ),
                    (alt - CONFIG.sea_level - 0.25 * CONFIG.mountain_scale + marble * 128.0)
                        / 100.0,
                ),
                // Beach
                ((alt - CONFIG.sea_level - 1.0) / 2.0).min(1.0 - river * 2.0),
            ),
            sub_surface_color: dirt,
            tree_density,
            forest_kind: sim_chunk.forest_kind,
            close_trees: sim.gen_ctx.tree_gen.get(wpos),
            cave_xy,
            cave_alt,
            rock,
            is_cliffs,
            near_cliffs,
            cliff_hill,
            close_cliffs: sim.gen_ctx.cliff_gen.get(wpos),
            temp,
            spawn_rate,
            location: sim_chunk.location.as_ref(),
        })
    }
}

#[derive(Clone)]
pub struct ColumnSample<'a> {
    pub alt: f32,
    pub chaos: f32,
    pub water_level: f32,
    pub river: f32,
    pub surface_color: Rgb<f32>,
    pub sub_surface_color: Rgb<f32>,
    pub tree_density: f32,
    pub forest_kind: ForestKind,
    pub close_trees: [(Vec2<i32>, u32); 9],
    pub cave_xy: f32,
    pub cave_alt: f32,
    pub rock: f32,
    pub is_cliffs: bool,
    pub near_cliffs: bool,
    pub cliff_hill: f32,
    pub close_cliffs: [(Vec2<i32>, u32); 9],
    pub temp: f32,
    pub spawn_rate: f32,
    pub location: Option<&'a LocationInfo>,
}
