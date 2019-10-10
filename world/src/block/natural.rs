use super::{BlockGen, StructureInfo, StructureMeta};
use crate::{
    all::ForestKind,
    column::{ColumnGen, ColumnSample},
    util::{RandomPerm, Sampler, SmallCache, UnitChooser},
    CONFIG,
};
use common::assets::Asset;
use common::{assets, terrain::Structure};
use lazy_static::lazy_static;
use ron;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::u32;
use vek::*;

static VOLUME_RAND: RandomPerm = RandomPerm::new(0xDB21C052);
static UNIT_CHOOSER: UnitChooser = UnitChooser::new(0x700F4EC7);
static QUIRKY_RAND: RandomPerm = RandomPerm::new(0xA634460F);

pub fn structure_gen<'a>(
    column_gen: &ColumnGen<'a>,
    column_cache: &mut SmallCache<Option<ColumnSample<'a>>>,
    idx: usize,
    st_pos: Vec2<i32>,
    st_seed: u32,
    structure_samples: &[Option<ColumnSample>; 9],
) -> Option<StructureInfo> {
    let st_sample = &structure_samples[idx].as_ref()?;

    // Assuming it's a tree... figure out when it SHOULDN'T spawn
    let random_seed = (st_seed as f64) / (u32::MAX as f64);
    if (st_sample.tree_density as f64) < random_seed
        || st_sample.alt < st_sample.water_level
        || st_sample.spawn_rate < 0.5
        || !st_sample.spawn_rules.trees
    {
        return None;
    }

    let cliff_height = BlockGen::get_cliff_height(
        column_gen,
        column_cache,
        st_pos.map(|e| e as f32),
        &st_sample.close_cliffs,
        st_sample.cliff_hill,
        0.0,
    );

    let wheight = st_sample.alt.max(cliff_height);
    let st_pos3d = Vec3::new(st_pos.x, st_pos.y, wheight as i32);

    let volumes: &'static [_] = if QUIRKY_RAND.get(st_seed) % 512 == 17 {
        if st_sample.temp > CONFIG.desert_temp {
            &QUIRKY_DRY
        } else {
            &QUIRKY
        }
    } else {
        match st_sample.forest_kind {
            ForestKind::Palm => &PALMS,
            ForestKind::Savannah => &ACACIAS,
            ForestKind::Oak if QUIRKY_RAND.get(st_seed) % 16 == 7 => &OAK_STUMPS,
            ForestKind::Oak if QUIRKY_RAND.get(st_seed) % 8 == 7 => &FRUIT_TREES,
            ForestKind::Oak => &OAKS,
            ForestKind::Pine => &PINES,
            ForestKind::SnowPine => &SNOW_PINES,
            ForestKind::Mangrove => &MANGROVE_TREES,
        }
    };

    Some(StructureInfo {
        pos: st_pos3d,
        seed: st_seed,
        meta: StructureMeta::Volume {
            units: UNIT_CHOOSER.get(st_seed),
            volume: &volumes[(VOLUME_RAND.get(st_seed) / 13) as usize % volumes.len()],
        },
    })
}

#[derive(Deserialize)]
struct StructureSpec {
    specifier: String,
    center: [i32; 3],
}
#[derive(Deserialize)]
struct StructuresSpec(Vec<StructureSpec>);

impl Asset for StructuresSpec {
    const ENDINGS: &'static [&'static str] = &["ron"];
    fn parse(buf_reader: BufReader<File>) -> Result<Self, assets::Error> {
        Ok(ron::de::from_reader(buf_reader).expect("Error parsing structure specs"))
    }
}

fn convert(specifications: &StructuresSpec) -> Vec<Arc<Structure>> {
    return specifications
        .0
        .iter()
        .map(|sp| {
            assets::load_map(&sp.specifier[..], |s: Structure| {
                s.with_center(Vec3::from(sp.center))
            })
            .unwrap()
        })
        .collect();
}

fn load_structures(specifier: &str) -> Vec<Arc<Structure>> {
    dbg!(specifier);
    let spec = assets::load::<StructuresSpec>(&["world.manifests.", specifier].concat());
    return convert(&spec.unwrap());
}

lazy_static! {
    pub static ref OAKS: Vec<Arc<Structure>> = load_structures("oaks");

    pub static ref OAK_STUMPS: Vec<Arc<Structure>> = load_structures("oak_stumps");

    pub static ref PINES: Vec<Arc<Structure>> = load_structures("pines");

    pub static ref PALMS: Vec<Arc<Structure>> = load_structures("palms");

    pub static ref SNOW_PINES: Vec<Arc<Structure>> = load_structures("snow_pines");

    pub static ref ACACIAS: Vec<Arc<Structure>> = load_structures("acacias");

    pub static ref FRUIT_TREES: Vec<Arc<Structure>> = load_structures("fruit_trees");

    pub static ref MANGROVE_TREES: Vec<Arc<Structure>> = load_structures("mangrove_trees");

    pub static ref QUIRKY: Vec<Arc<Structure>> = load_structures("quirky");

    pub static ref QUIRKY_DRY: Vec<Arc<Structure>> = load_structures("quirky_dry");

     /*
        // green pines 2
         assets::load_map("world/tree/pine_green_2/1", |s: Structure| s
            .with_center(Vec3::new(15, 15, 14)))
        .unwrap(),
        assets::load_map("world/tree/pine_green_2/2", |s: Structure| s
            .with_center(Vec3::new(15, 15, 14)))
        .unwrap(),
        assets::load_map("world/tree/pine_green_2/3", |s: Structure| s
            .with_center(Vec3::new(17, 15, 12)))
        .unwrap(),
        assets::load_map("world/tree/pine_green_2/4", |s: Structure| s
            .with_center(Vec3::new(10, 8, 12)))
        .unwrap(),
        assets::load_map("world/tree/pine_green_2/5", |s: Structure| s
            .with_center(Vec3::new(12, 12, 12)))
        .unwrap(),
        assets::load_map("world/tree/pine_green_2/6", |s: Structure| s
            .with_center(Vec3::new(11, 10, 12)))
        .unwrap(),
        assets::load_map("world/tree/pine_green_2/7", |s: Structure| s
            .with_center(Vec3::new(16, 15, 12)))
        .unwrap(),
        assets::load_map("world/tree/pine_green_2/8", |s: Structure| s
            .with_center(Vec3::new(12, 10, 12)))
        .unwrap(),
        // blue pines
        assets::load_map("world/tree/pine_blue/1", |s: Structure| s
            .with_center(Vec3::new(15, 15, 14)))
        .unwrap(),
        assets::load_map("world/tree/pine_blue/2", |s: Structure| s
            .with_center(Vec3::new(15, 15, 14)))
        .unwrap(),
        assets::load_map("world/tree/pine_blue/3", |s: Structure| s
            .with_center(Vec3::new(17, 15, 12)))
        .unwrap(),
        assets::load_map("world/tree/pine_blue/4", |s: Structure| s
            .with_center(Vec3::new(10, 8, 12)))
        .unwrap(),
        assets::load_map("world/tree/pine_blue/5", |s: Structure| s
            .with_center(Vec3::new(12, 12, 12)))
        .unwrap(),
        assets::load_map("world/tree/pine_blue/6", |s: Structure| s
            .with_center(Vec3::new(11, 10, 12)))
        .unwrap(),
        assets::load_map("world/tree/pine_blue/7", |s: Structure| s
            .with_center(Vec3::new(16, 15, 12)))
        .unwrap(),
        assets::load_map("world/tree/pine_blue/8", |s: Structure| s
            .with_center(Vec3::new(12, 10, 12)))
        .unwrap(),
        */
      /*
        // temperate small
        assets::load_map("world/tree/temperate_small/1", |s: Structure| s
            .with_center(Vec3::new(4, 4, 7)))
        .unwrap(),
        assets::load_map("world/tree/temperate_small/2", |s: Structure| s
            .with_center(Vec3::new(4, 4, 7)))
        .unwrap(),
        assets::load_map("world/tree/temperate_small/3", |s: Structure| s
            .with_center(Vec3::new(4, 4, 7)))
        .unwrap(),
        assets::load_map("world/tree/temperate_small/4", |s: Structure| s
            .with_center(Vec3::new(4, 4, 7)))
        .unwrap(),
        assets::load_map("world/tree/temperate_small/5", |s: Structure| s
            .with_center(Vec3::new(4, 4, 7)))
        .unwrap(),
        assets::load_map("world/tree/temperate_small/6", |s: Structure| s
            .with_center(Vec3::new(4, 4, 7)))
        .unwrap(),
        // birch
        assets::load_map("world/tree/birch/1", |s: Structure| s
            .with_center(Vec3::new(12, 9, 10)))
        .unwrap(),
        assets::load_map("world/tree/birch/2", |s: Structure| s
            .with_center(Vec3::new(11, 10, 10)))
        .unwrap(),
        assets::load_map("world/tree/birch/3", |s: Structure| s
            .with_center(Vec3::new(9, 10, 10)))
        .unwrap(),
        assets::load_map("world/tree/birch/4", |s: Structure| s
            .with_center(Vec3::new(9, 10, 10)))
        .unwrap(),
        assets::load_map("world/tree/birch/5", |s: Structure| s
            .with_center(Vec3::new(9, 11, 10)))
        .unwrap(),
        assets::load_map("world/tree/birch/6", |s: Structure| s
            .with_center(Vec3::new(9, 9, 10)))
        .unwrap(),
        assets::load_map("world/tree/birch/7", |s: Structure| s
            .with_center(Vec3::new(10, 10, 10)))
        .unwrap(),
        assets::load_map("world/tree/birch/8", |s: Structure| s
            .with_center(Vec3::new(9, 9, 10)))
        .unwrap(),
        assets::load_map("world/tree/birch/9", |s: Structure| s
            .with_center(Vec3::new(9, 10, 10)))
        .unwrap(),
        assets::load_map("world/tree/birch/10", |s: Structure| s
            .with_center(Vec3::new(10, 9, 10)))
        .unwrap(),
        assets::load_map("world/tree/birch/11", |s: Structure| s
            .with_center(Vec3::new(9, 10, 10)))
        .unwrap(),
        assets::load_map("world/tree/birch/12", |s: Structure| s
            .with_center(Vec3::new(10, 9, 10)))
        .unwrap(),
        // poplar
        assets::load_map("world/tree/poplar/1", |s: Structure| s
            .with_center(Vec3::new(6, 6, 10)))
        .unwrap(),
        assets::load_map("world/tree/poplar/2", |s: Structure| s
            .with_center(Vec3::new(6, 6, 10)))
        .unwrap(),
        assets::load_map("world/tree/poplar/3", |s: Structure| s
            .with_center(Vec3::new(6, 6, 10)))
        .unwrap(),
        assets::load_map("world/tree/poplar/4", |s: Structure| s
            .with_center(Vec3::new(6, 6, 10)))
        .unwrap(),
        assets::load_map("world/tree/poplar/5", |s: Structure| s
            .with_center(Vec3::new(6, 6, 10)))
        .unwrap(),
        assets::load_map("world/tree/poplar/6", |s: Structure| s
            .with_center(Vec3::new(6, 6, 10)))
        .unwrap(),
        assets::load_map("world/tree/poplar/7", |s: Structure| s
            .with_center(Vec3::new(6, 6, 10)))
        .unwrap(),
        assets::load_map("world/tree/poplar/8", |s: Structure| s
            .with_center(Vec3::new(6, 6, 10)))
        .unwrap(),
        assets::load_map("world/tree/poplar/9", |s: Structure| s
            .with_center(Vec3::new(6, 6, 10)))
        .unwrap(),
        assets::load_map("world/tree/poplar/10", |s: Structure| s
            .with_center(Vec3::new(7, 7, 10)))
        .unwrap(),
        */
        /*
        // snow birches -> need roots!
        assets::load_map("world/tree/snow_birch/1", |s: Structure| s
            .with_center(Vec3::new(12, 9, 4)))
        .unwrap(),
        assets::load_map("world/tree/snow_birch/2", |s: Structure| s
            .with_center(Vec3::new(11, 10, 4)))
        .unwrap(),
        assets::load_map("world/tree/snow_birch/3", |s: Structure| s
            .with_center(Vec3::new(9, 10, 4)))
        .unwrap(),
        assets::load_map("world/tree/snow_birch/4", |s: Structure| s
            .with_center(Vec3::new(9, 10, 4)))
        .unwrap(),
        assets::load_map("world/tree/snow_birch/5", |s: Structure| s
            .with_center(Vec3::new(9, 11, 4)))
        .unwrap(),
        assets::load_map("world/tree/snow_birch/6", |s: Structure| s
            .with_center(Vec3::new(9, 9, 4)))
        .unwrap(),
        assets::load_map("world/tree/snow_birch/7", |s: Structure| s
            .with_center(Vec3::new(10, 10, 4)))
        .unwrap(),
        assets::load_map("world/tree/snow_birch/8", |s: Structure| s
            .with_center(Vec3::new(9, 9, 4)))
        .unwrap(),
        assets::load_map("world/tree/snow_birch/9", |s: Structure| s
            .with_center(Vec3::new(9, 10, 4)))
        .unwrap(),
        assets::load_map("world/tree/snow_birch/10", |s: Structure| s
            .with_center(Vec3::new(10, 9, 4)))
        .unwrap(),
        assets::load_map("world/tree/snow_birch/11", |s: Structure| s
            .with_center(Vec3::new(9, 10, 4)))
        .unwrap(),
        assets::load_map("world/tree/snow_birch/12", |s: Structure| s
            .with_center(Vec3::new(10, 9, 4)))
        .unwrap(),
        // willows
        assets::load_map("world/tree/willow/1", |s: Structure| s
            .with_center(Vec3::new(15, 14, 1)))
        .unwrap(),
        assets::load_map("world/tree/willow/2", |s: Structure| s
            .with_center(Vec3::new(11, 12, 1)))
        .unwrap(),
    ];
    */
}
