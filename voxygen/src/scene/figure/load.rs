use crate::{
    mesh::Meshable,
    render::{FigurePipeline, Mesh},
};
use common::comp::humanoid::Body;
use common::{
    assets::{self, watch::ReloadIndicator, Asset},
    comp::{
        biped_large, bird_medium, bird_small, dragon, fish_medium, fish_small,
        humanoid::{
            Belt, BodyType, Chest, EyeColor, Eyebrows, Foot, Hand, Pants, Race, Shoulder, Skin,
        },
        item::Tool,
        object, quadruped_medium, quadruped_small, Item, ItemKind,
    },
    figure::{DynaUnionizer, MatSegment, Material, Segment},
};
use dot_vox::DotVoxData;
use hashbrown::HashMap;
use log::{error, warn};
use serde_derive::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, sync::Arc};
use vek::*;

fn load_segment(mesh_name: &str) -> Segment {
    let full_specifier: String = ["voxygen.voxel.", mesh_name].concat();
    Segment::from(assets::load_expect::<DotVoxData>(full_specifier.as_str()).as_ref())
}
fn graceful_load_vox(mesh_name: &str) -> Arc<DotVoxData> {
    let full_specifier: String = ["voxygen.voxel.", mesh_name].concat();
    match assets::load::<DotVoxData>(full_specifier.as_str()) {
        Ok(dot_vox) => dot_vox,
        Err(_) => {
            error!("Could not load vox file for figure: {}", full_specifier);
            assets::load_expect::<DotVoxData>("voxygen.voxel.not_found")
        }
    }
}
fn graceful_load_segment(mesh_name: &str) -> Segment {
    Segment::from(graceful_load_vox(mesh_name).as_ref())
}
fn graceful_load_mat_segment(mesh_name: &str) -> MatSegment {
    MatSegment::from(graceful_load_vox(mesh_name).as_ref())
}

fn generate_mesh(segment: &Segment, offset: Vec3<f32>) -> Mesh<FigurePipeline> {
    Meshable::<FigurePipeline, FigurePipeline>::generate_mesh(segment, offset).0
}

pub fn load_mesh(mesh_name: &str, position: Vec3<f32>) -> Mesh<FigurePipeline> {
    generate_mesh(&load_segment(mesh_name), position)
}

fn color_segment(
    mat_segment: MatSegment,
    skin: Skin,
    hair_color: Rgb<u8>,
    eye_color: EyeColor,
) -> Segment {
    // TODO move some of the colors to common
    mat_segment.to_segment(|mat| match mat {
        Material::Skin => skin.rgb(),
        Material::SkinDark => skin.dark_rgb(),
        Material::SkinLight => skin.light_rgb(),
        Material::Hair => hair_color,
        // TODO add back multiple colors
        Material::EyeLight => eye_color.light_rgb(),
        Material::EyeDark => eye_color.dark_rgb(),
        Material::EyeWhite => eye_color.white_rgb(),
    })
}

fn recolor_grey(rgb: Rgb<u8>, color: Rgb<u8>) -> Rgb<u8> {
    use common::util::{linear_to_srgb, srgb_to_linear};

    const BASE_GREY: f32 = 178.0;
    if rgb.r == rgb.g && rgb.g == rgb.b {
        let c1 = srgb_to_linear(rgb.map(|e| e as f32 / BASE_GREY));
        let c2 = srgb_to_linear(color.map(|e| e as f32 / 255.0));

        linear_to_srgb(c1 * c2).map(|e| (e.min(1.0).max(0.0) * 255.0) as u8)
    } else {
        rgb
    }
}

// All offsets should be relative to an initial origin that doesn't change when combining segments
#[derive(Serialize, Deserialize)]
struct VoxSpec<T>(String, [T; 3]);

// Armor can have the color modified.
#[derive(Serialize, Deserialize)]
struct ArmorVoxSpec {
    vox_spec: VoxSpec<f32>,
    color: Option<[u8; 3]>,
}

// For use by armor with a left and right component
#[derive(Serialize, Deserialize)]
struct SidedArmorVoxSpec {
    left: ArmorVoxSpec,
    right: ArmorVoxSpec,
}

// All reliant on humanoid::Race and humanoid::BodyType
#[derive(Serialize, Deserialize)]
struct HumHeadSubSpec {
    offset: [f32; 3], // Should be relative to initial origin
    head: VoxSpec<i32>,
    eyes: VoxSpec<i32>,
    hair: Vec<Option<VoxSpec<i32>>>,
    beard: Vec<Option<VoxSpec<i32>>>,
    accessory: Vec<Option<VoxSpec<i32>>>,
}
#[derive(Serialize, Deserialize)]
pub struct HumHeadSpec(HashMap<(Race, BodyType), HumHeadSubSpec>);

impl Asset for HumHeadSpec {
    const ENDINGS: &'static [&'static str] = &["ron"];
    fn parse(buf_reader: BufReader<File>) -> Result<Self, assets::Error> {
        Ok(ron::de::from_reader(buf_reader).expect("Error parsing humanoid head spec"))
    }
}

impl HumHeadSpec {
    pub fn load_watched(indicator: &mut ReloadIndicator) -> Arc<Self> {
        assets::load_watched::<Self>("voxygen.voxel.humanoid_head_manifest", indicator).unwrap()
    }
    pub fn mesh_head(
        &self,
        race: Race,
        body_type: BodyType,
        hair_color: u8,
        hair_style: u8,
        beard: u8,
        eye_color: u8,
        skin: u8,
        _eyebrows: Eyebrows,
        accessory: u8,
    ) -> Mesh<FigurePipeline> {
        let spec = match self.0.get(&(race, body_type)) {
            Some(spec) => spec,
            None => {
                error!(
                    "No head specification exists for the combination of {:?} and {:?}",
                    race, body_type
                );
                return load_mesh("not_found", Vec3::new(-5.0, -5.0, -2.5));
            }
        };

        let hair_rgb = race.hair_color(hair_color);
        let skin = race.skin_color(skin);
        let eye_rgb = race.eye_color(eye_color);

        // Load segment pieces
        let bare_head = graceful_load_mat_segment(&spec.head.0);
        let eyes = color_segment(
            graceful_load_mat_segment(&spec.eyes.0).map_rgb(|rgb| recolor_grey(rgb, hair_rgb)),
            skin,
            hair_rgb,
            eye_rgb,
        );
        let hair = match spec.hair.get(hair_style as usize) {
            Some(Some(spec)) => Some((
                graceful_load_segment(&spec.0).map_rgb(|rgb| recolor_grey(rgb, hair_rgb)),
                Vec3::from(spec.1),
            )),
            Some(None) => None,
            None => {
                warn!("No specification for hair style {}", hair_style);
                None
            }
        };
        let beard = match spec.beard.get(beard as usize) {
            Some(Some(spec)) => Some((
                graceful_load_segment(&spec.0).map_rgb(|rgb| recolor_grey(rgb, hair_rgb)),
                Vec3::from(spec.1),
            )),
            Some(None) => None,
            None => {
                warn!("No specification for this beard: {:?}", beard);
                None
            }
        };
        let accessory = match spec.accessory.get(accessory as usize) {
            Some(Some(spec)) => Some((graceful_load_segment(&spec.0), Vec3::from(spec.1))),
            Some(None) => None,
            None => {
                warn!("No specification for this accessory: {:?}", accessory);
                None
            }
        };

        let (head, origin_offset) = DynaUnionizer::new()
            .add(
                color_segment(bare_head, skin, hair_rgb, eye_rgb),
                spec.head.1.into(),
            )
            .add(eyes, spec.eyes.1.into())
            .maybe_add(hair)
            .maybe_add(beard)
            .maybe_add(accessory)
            .unify();

        generate_mesh(
            &head,
            Vec3::from(spec.offset) + origin_offset.map(|e| e as f32 * -1.0),
        )
    }
}
// Armor spects should be in the same order, top to bottom.
// These seem overly split up, but wanted to keep the armor seperated
// unlike head which is done above.

#[derive(Serialize, Deserialize)]
pub struct HumArmorShoulderSpec(HashMap<Shoulder, SidedArmorVoxSpec>);
#[derive(Serialize, Deserialize)]
pub struct HumArmorChestSpec(HashMap<Chest, ArmorVoxSpec>);
#[derive(Serialize, Deserialize)]
pub struct HumArmorHandSpec(HashMap<Hand, SidedArmorVoxSpec>);
#[derive(Serialize, Deserialize)]
pub struct HumArmorBeltSpec(HashMap<Belt, ArmorVoxSpec>);
#[derive(Serialize, Deserialize)]
pub struct HumArmorPantsSpec(HashMap<Pants, ArmorVoxSpec>);
#[derive(Serialize, Deserialize)]
pub struct HumArmorFootSpec(HashMap<Foot, ArmorVoxSpec>);

impl Asset for HumArmorShoulderSpec {
    const ENDINGS: &'static [&'static str] = &["ron"];
    fn parse(buf_reader: BufReader<File>) -> Result<Self, assets::Error> {
        Ok(ron::de::from_reader(buf_reader).expect("Error parsing humanoid armor shoulder spec"))
    }
}
impl Asset for HumArmorChestSpec {
    const ENDINGS: &'static [&'static str] = &["ron"];
    fn parse(buf_reader: BufReader<File>) -> Result<Self, assets::Error> {
        Ok(ron::de::from_reader(buf_reader).expect("Error parsing humanoid armor chest spec"))
    }
}
impl Asset for HumArmorHandSpec {
    const ENDINGS: &'static [&'static str] = &["ron"];
    fn parse(buf_reader: BufReader<File>) -> Result<Self, assets::Error> {
        Ok(ron::de::from_reader(buf_reader).expect("Error parsing humanoid armor hand spec"))
    }
}
impl Asset for HumArmorBeltSpec {
    const ENDINGS: &'static [&'static str] = &["ron"];
    fn parse(buf_reader: BufReader<File>) -> Result<Self, assets::Error> {
        Ok(ron::de::from_reader(buf_reader).expect("Error parsing humanoid armor belt spec"))
    }
}
impl Asset for HumArmorPantsSpec {
    const ENDINGS: &'static [&'static str] = &["ron"];
    fn parse(buf_reader: BufReader<File>) -> Result<Self, assets::Error> {
        Ok(ron::de::from_reader(buf_reader).expect("Error parsing humanoid armor pants spec"))
    }
}
impl Asset for HumArmorFootSpec {
    const ENDINGS: &'static [&'static str] = &["ron"];
    fn parse(buf_reader: BufReader<File>) -> Result<Self, assets::Error> {
        Ok(ron::de::from_reader(buf_reader).expect("Error parsing humanoid armor foot spec"))
    }
}

impl HumArmorShoulderSpec {
    pub fn load_watched(indicator: &mut ReloadIndicator) -> Arc<Self> {
        assets::load_watched::<Self>("voxygen.voxel.humanoid_armor_shoulder_manifest", indicator)
            .unwrap()
    }

    pub fn mesh_left_shoulder(&self, body: &Body) -> Mesh<FigurePipeline> {
        let spec = match self.0.get(&body.shoulder) {
            Some(spec) => spec,
            None => {
                error!("No shoulder specification exists for {:?}", body.shoulder);
                return load_mesh("not_found", Vec3::new(-3.0, -3.5, 0.1));
            }
        };

        let shoulder_segment = color_segment(
            graceful_load_mat_segment(&spec.left.vox_spec.0),
            body.race.skin_color(body.skin),
            body.race.hair_color(body.hair_color),
            body.race.eye_color(body.eye_color),
        );

        generate_mesh(&shoulder_segment, Vec3::from(spec.left.vox_spec.1))
    }

    pub fn mesh_right_shoulder(&self, body: &Body) -> Mesh<FigurePipeline> {
        let spec = match self.0.get(&body.shoulder) {
            Some(spec) => spec,
            None => {
                error!("No shoulder specification exists for {:?}", body.shoulder);
                return load_mesh("not_found", Vec3::new(-2.0, -3.5, 0.1));
            }
        };

        let shoulder_segment = color_segment(
            graceful_load_mat_segment(&spec.right.vox_spec.0),
            body.race.skin_color(body.skin),
            body.race.hair_color(body.hair_color),
            body.race.eye_color(body.eye_color),
        );

        generate_mesh(&shoulder_segment, Vec3::from(spec.right.vox_spec.1))
    }
}

impl HumArmorChestSpec {
    pub fn load_watched(indicator: &mut ReloadIndicator) -> Arc<Self> {
        assets::load_watched::<Self>("voxygen.voxel.humanoid_armor_chest_manifest", indicator)
            .unwrap()
    }

    pub fn mesh_chest(&self, body: &Body) -> Mesh<FigurePipeline> {
        let spec = match self.0.get(&body.chest) {
            Some(spec) => spec,
            None => {
                error!("No chest specification exists for {:?}", body.chest);
                return load_mesh("not_found", Vec3::new(-7.0, -3.5, 2.0));
            }
        };

        let color = |mat_segment| {
            color_segment(
                mat_segment,
                body.race.skin_color(body.skin),
                body.race.hair_color(body.hair_color),
                body.race.eye_color(body.eye_color),
            )
        };

        let bare_chest = graceful_load_mat_segment("armor.chest.none");

        let mut chest_armor = graceful_load_mat_segment(&spec.vox_spec.0);

        if let Some(color) = spec.color {
            let chest_color = Vec3::from(color);
            chest_armor = chest_armor.map_rgb(|rgb| recolor_grey(rgb, Rgb::from(chest_color)));
        }

        let chest = DynaUnionizer::new()
            .add(color(bare_chest), Vec3::new(0, 0, 0))
            .add(color(chest_armor), Vec3::new(0, 0, 0))
            .unify()
            .0;

        generate_mesh(&chest, Vec3::from(spec.vox_spec.1))
    }
}

impl HumArmorHandSpec {
    pub fn load_watched(indicator: &mut ReloadIndicator) -> Arc<Self> {
        assets::load_watched::<Self>("voxygen.voxel.humanoid_armor_hand_manifest", indicator)
            .unwrap()
    }

    pub fn mesh_left_hand(&self, body: &Body) -> Mesh<FigurePipeline> {
        let spec = match self.0.get(&body.hand) {
            Some(spec) => spec,
            None => {
                error!("No hand specification exists for {:?}", body.hand);
                return load_mesh("not_found", Vec3::new(-1.5, -1.5, -7.0));
            }
        };

        let hand_segment = color_segment(
            graceful_load_mat_segment(&spec.left.vox_spec.0),
            body.race.skin_color(body.skin),
            body.race.hair_color(body.hair_color),
            body.race.eye_color(body.eye_color),
        );

        generate_mesh(&hand_segment, Vec3::from(spec.left.vox_spec.1))
    }

    pub fn mesh_right_hand(&self, body: &Body) -> Mesh<FigurePipeline> {
        let spec = match self.0.get(&body.hand) {
            Some(spec) => spec,
            None => {
                error!("No hand specification exists for {:?}", body.hand);
                return load_mesh("not_found", Vec3::new(-1.5, -1.5, -7.0));
            }
        };

        let hand_segment = color_segment(
            graceful_load_mat_segment(&spec.right.vox_spec.0),
            body.race.skin_color(body.skin),
            body.race.hair_color(body.hair_color),
            body.race.eye_color(body.eye_color),
        );

        generate_mesh(&hand_segment, Vec3::from(spec.right.vox_spec.1))
    }
}

impl HumArmorBeltSpec {
    pub fn load_watched(indicator: &mut ReloadIndicator) -> Arc<Self> {
        assets::load_watched::<Self>("voxygen.voxel.humanoid_armor_belt_manifest", indicator)
            .unwrap()
    }

    pub fn mesh_belt(&self, body: &Body) -> Mesh<FigurePipeline> {
        let spec = match self.0.get(&body.belt) {
            Some(spec) => spec,
            None => {
                error!("No belt specification exists for {:?}", body.belt);
                return load_mesh("not_found", Vec3::new(-4.0, -3.5, 2.0));
            }
        };

        let belt_segment = color_segment(
            graceful_load_mat_segment(&spec.vox_spec.0),
            body.race.skin_color(body.skin),
            body.race.hair_color(body.hair_color),
            body.race.eye_color(body.eye_color),
        );

        generate_mesh(&belt_segment, Vec3::from(spec.vox_spec.1))
    }
}

impl HumArmorPantsSpec {
    pub fn load_watched(indicator: &mut ReloadIndicator) -> Arc<Self> {
        assets::load_watched::<Self>("voxygen.voxel.humanoid_armor_pants_manifest", indicator)
            .unwrap()
    }

    pub fn mesh_pants(&self, body: &Body) -> Mesh<FigurePipeline> {
        let spec = match self.0.get(&body.pants) {
            Some(spec) => spec,
            None => {
                error!("No pants specification exists for {:?}", body.pants);
                return load_mesh("not_found", Vec3::new(-5.0, -3.5, 1.0));
            }
        };

        let color = |mat_segment| {
            color_segment(
                mat_segment,
                body.race.skin_color(body.skin),
                body.race.hair_color(body.hair_color),
                body.race.eye_color(body.eye_color),
            )
        };

        let bare_pants = graceful_load_mat_segment("armor.pants.grayscale");

        let mut pants_armor = graceful_load_mat_segment(&spec.vox_spec.0);

        if let Some(color) = spec.color {
            let pants_color = Vec3::from(color);
            pants_armor = pants_armor.map_rgb(|rgb| recolor_grey(rgb, Rgb::from(pants_color)));
        }

        let pants = DynaUnionizer::new()
            .add(color(bare_pants), Vec3::new(0, 0, 0))
            .add(color(pants_armor), Vec3::new(0, 0, 0))
            .unify()
            .0;

        generate_mesh(&pants, Vec3::from(spec.vox_spec.1))
    }
}

impl HumArmorFootSpec {
    pub fn load_watched(indicator: &mut ReloadIndicator) -> Arc<Self> {
        assets::load_watched::<Self>("voxygen.voxel.humanoid_armor_foot_manifest", indicator)
            .unwrap()
    }

    pub fn mesh_left_foot(&self, body: &Body) -> Mesh<FigurePipeline> {
        let spec = match self.0.get(&body.foot) {
            Some(spec) => spec,
            None => {
                error!("No foot specification exists for {:?}", body.foot);
                return load_mesh("not_found", Vec3::new(-2.5, -3.5, -9.0));
            }
        };

        let foot_segment = color_segment(
            graceful_load_mat_segment(&spec.vox_spec.0),
            body.race.skin_color(body.skin),
            body.race.hair_color(body.hair_color),
            body.race.eye_color(body.eye_color),
        );

        generate_mesh(&foot_segment, Vec3::from(spec.vox_spec.1))
    }

    pub fn mesh_right_foot(&self, body: &Body) -> Mesh<FigurePipeline> {
        let spec = match self.0.get(&body.foot) {
            Some(spec) => spec,
            None => {
                error!("No foot specification exists for {:?}", body.foot);
                return load_mesh("not_found", Vec3::new(-2.5, -3.5, -9.0));
            }
        };

        let foot_segment = color_segment(
            graceful_load_mat_segment(&spec.vox_spec.0),
            body.race.skin_color(body.skin),
            body.race.hair_color(body.hair_color),
            body.race.eye_color(body.eye_color),
        );

        generate_mesh(&foot_segment, Vec3::from(spec.vox_spec.1))
    }
}

pub fn mesh_main(item: Option<&Item>) -> Mesh<FigurePipeline> {
    if let Some(item) = item {
        let (name, offset) = match item.kind {
            ItemKind::Tool { kind, .. } => match kind {
                Tool::Sword => ("weapon.sword.rusty_2h", Vec3::new(-1.5, -6.5, -4.0)),
                Tool::Axe => ("weapon.axe.rusty_2h", Vec3::new(-1.5, -5.0, -4.0)),
                Tool::Hammer => ("weapon.hammer.rusty_2h", Vec3::new(-2.5, -5.5, -4.0)),
                Tool::Dagger => ("weapon.hammer.rusty_2h", Vec3::new(-2.5, -5.5, -4.0)),
                Tool::Shield => ("weapon.axe.rusty_2h", Vec3::new(-2.5, -6.5, -2.0)),
                Tool::Bow => ("weapon.bow.simple-bow", Vec3::new(-1.0, -6.0, -2.0)),
                Tool::Staff => ("weapon.staff.wood-fire", Vec3::new(-1.0, -6.0, -3.0)),
                Tool::Debug(_) => ("weapon.debug_wand", Vec3::new(-1.5, -9.5, -4.0)),
            },
            _ => return Mesh::new(),
        };
        load_mesh(name, offset)
    } else {
        Mesh::new()
    }
}

// TODO: Inventory
pub fn mesh_draw() -> Mesh<FigurePipeline> {
    load_mesh("object.glider", Vec3::new(-26.0, -26.0, -5.0))
}

/////////
pub fn mesh_quadruped_small_head(head: quadruped_small::Head) -> Mesh<FigurePipeline> {
    load_mesh(
        match head {
            quadruped_small::Head::Default => "npc.pig_purple.head",
        },
        Vec3::new(-6.0, 4.5, 3.0),
    )
}

pub fn mesh_quadruped_small_chest(chest: quadruped_small::Chest) -> Mesh<FigurePipeline> {
    load_mesh(
        match chest {
            quadruped_small::Chest::Default => "npc.pig_purple.chest",
        },
        Vec3::new(-5.0, 4.5, 0.0),
    )
}

pub fn mesh_quadruped_small_leg_lf(leg_l: quadruped_small::LegL) -> Mesh<FigurePipeline> {
    load_mesh(
        match leg_l {
            quadruped_small::LegL::Default => "npc.pig_purple.leg_l",
        },
        Vec3::new(0.0, -1.0, -1.5),
    )
}

pub fn mesh_quadruped_small_leg_rf(leg_r: quadruped_small::LegR) -> Mesh<FigurePipeline> {
    load_mesh(
        match leg_r {
            quadruped_small::LegR::Default => "npc.pig_purple.leg_r",
        },
        Vec3::new(0.0, -1.0, -1.5),
    )
}

pub fn mesh_quadruped_small_leg_lb(leg_l: quadruped_small::LegL) -> Mesh<FigurePipeline> {
    load_mesh(
        match leg_l {
            quadruped_small::LegL::Default => "npc.pig_purple.leg_l",
        },
        Vec3::new(0.0, -1.0, -1.5),
    )
}

pub fn mesh_quadruped_small_leg_rb(leg_r: quadruped_small::LegR) -> Mesh<FigurePipeline> {
    load_mesh(
        match leg_r {
            quadruped_small::LegR::Default => "npc.pig_purple.leg_r",
        },
        Vec3::new(0.0, -1.0, -1.5),
    )
}
//////
pub fn mesh_quadruped_medium_head_upper(
    upper_head: quadruped_medium::HeadUpper,
) -> Mesh<FigurePipeline> {
    load_mesh(
        match upper_head {
            quadruped_medium::HeadUpper::Default => "npc.wolf.head_upper",
        },
        Vec3::new(-7.0, -6.0, -5.5),
    )
}

pub fn mesh_quadruped_medium_jaw(jaw: quadruped_medium::Jaw) -> Mesh<FigurePipeline> {
    load_mesh(
        match jaw {
            quadruped_medium::Jaw::Default => "npc.wolf.jaw",
        },
        Vec3::new(-3.0, -3.0, -2.5),
    )
}

pub fn mesh_quadruped_medium_head_lower(
    head_lower: quadruped_medium::HeadLower,
) -> Mesh<FigurePipeline> {
    load_mesh(
        match head_lower {
            quadruped_medium::HeadLower::Default => "npc.wolf.head_lower",
        },
        Vec3::new(-7.0, -6.0, -5.5),
    )
}

pub fn mesh_quadruped_medium_tail(tail: quadruped_medium::Tail) -> Mesh<FigurePipeline> {
    load_mesh(
        match tail {
            quadruped_medium::Tail::Default => "npc.wolf.tail",
        },
        Vec3::new(-2.0, -12.0, -5.0),
    )
}

pub fn mesh_quadruped_medium_torso_back(
    torso_back: quadruped_medium::TorsoBack,
) -> Mesh<FigurePipeline> {
    load_mesh(
        match torso_back {
            quadruped_medium::TorsoBack::Default => "npc.wolf.torso_back",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_quadruped_medium_torso_mid(
    torso_mid: quadruped_medium::TorsoMid,
) -> Mesh<FigurePipeline> {
    load_mesh(
        match torso_mid {
            quadruped_medium::TorsoMid::Default => "npc.wolf.torso_mid",
        },
        Vec3::new(-8.0, -5.5, -6.0),
    )
}

pub fn mesh_quadruped_medium_ears(ears: quadruped_medium::Ears) -> Mesh<FigurePipeline> {
    load_mesh(
        match ears {
            quadruped_medium::Ears::Default => "npc.wolf.ears",
        },
        Vec3::new(-4.0, -1.0, -1.0),
    )
}

pub fn mesh_quadruped_medium_foot_lf(foot_lf: quadruped_medium::FootLF) -> Mesh<FigurePipeline> {
    load_mesh(
        match foot_lf {
            quadruped_medium::FootLF::Default => "npc.wolf.foot_lf",
        },
        Vec3::new(-2.5, -4.0, -2.5),
    )
}

pub fn mesh_quadruped_medium_foot_rf(foot_rf: quadruped_medium::FootRF) -> Mesh<FigurePipeline> {
    load_mesh(
        match foot_rf {
            quadruped_medium::FootRF::Default => "npc.wolf.foot_rf",
        },
        Vec3::new(-2.5, -4.0, -2.5),
    )
}

pub fn mesh_quadruped_medium_foot_lb(foot_lb: quadruped_medium::FootLB) -> Mesh<FigurePipeline> {
    load_mesh(
        match foot_lb {
            quadruped_medium::FootLB::Default => "npc.wolf.foot_lb",
        },
        Vec3::new(-2.5, -4.0, -2.5),
    )
}

pub fn mesh_quadruped_medium_foot_rb(foot_rb: quadruped_medium::FootRB) -> Mesh<FigurePipeline> {
    load_mesh(
        match foot_rb {
            quadruped_medium::FootRB::Default => "npc.wolf.foot_rb",
        },
        Vec3::new(-2.5, -4.0, -2.5),
    )
}
////
pub fn mesh_bird_medium_head(head: bird_medium::Head) -> Mesh<FigurePipeline> {
    load_mesh(
        match head {
            bird_medium::Head::Default => "npc.duck_m.head",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_bird_medium_torso(torso: bird_medium::Torso) -> Mesh<FigurePipeline> {
    load_mesh(
        match torso {
            bird_medium::Torso::Default => "npc.duck_m.body",
        },
        Vec3::new(-8.0, -5.5, -6.0),
    )
}

pub fn mesh_bird_medium_tail(tail: bird_medium::Tail) -> Mesh<FigurePipeline> {
    load_mesh(
        match tail {
            bird_medium::Tail::Default => "npc.duck_m.tail",
        },
        Vec3::new(-4.0, -1.0, -1.0),
    )
}

pub fn mesh_bird_medium_wing_l(wing_l: bird_medium::WingL) -> Mesh<FigurePipeline> {
    load_mesh(
        match wing_l {
            bird_medium::WingL::Default => "npc.duck_m.wing",
        },
        Vec3::new(-2.5, -4.0, -2.5),
    )
}

pub fn mesh_bird_medium_wing_r(wing_r: bird_medium::WingR) -> Mesh<FigurePipeline> {
    load_mesh(
        match wing_r {
            bird_medium::WingR::Default => "npc.duck_m.wing",
        },
        Vec3::new(-2.5, -4.0, -2.5),
    )
}

pub fn mesh_bird_medium_leg_l(leg_l: bird_medium::LegL) -> Mesh<FigurePipeline> {
    load_mesh(
        match leg_l {
            bird_medium::LegL::Default => "npc.duck_m.leg_l",
        },
        Vec3::new(-2.5, -4.0, -2.5),
    )
}

pub fn mesh_bird_medium_leg_r(leg_r: bird_medium::LegR) -> Mesh<FigurePipeline> {
    load_mesh(
        match leg_r {
            bird_medium::LegR::Default => "npc.duck_m.leg_r",
        },
        Vec3::new(-2.5, -4.0, -2.5),
    )
}
////
pub fn mesh_fish_medium_head(head: fish_medium::Head) -> Mesh<FigurePipeline> {
    load_mesh(
        match head {
            fish_medium::Head::Default => "npc.marlin.head",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_fish_medium_torso(torso: fish_medium::Torso) -> Mesh<FigurePipeline> {
    load_mesh(
        match torso {
            fish_medium::Torso::Default => "npc.marlin.torso",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_fish_medium_rear(rear: fish_medium::Rear) -> Mesh<FigurePipeline> {
    load_mesh(
        match rear {
            fish_medium::Rear::Default => "npc.marlin.rear",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_fish_medium_tail(tail: fish_medium::Tail) -> Mesh<FigurePipeline> {
    load_mesh(
        match tail {
            fish_medium::Tail::Default => "npc.marlin.tail",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_fish_medium_fin_l(fin_l: fish_medium::FinL) -> Mesh<FigurePipeline> {
    load_mesh(
        match fin_l {
            fish_medium::FinL::Default => "npc.marlin.fin_l",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_fish_medium_fin_r(fin_r: fish_medium::FinR) -> Mesh<FigurePipeline> {
    load_mesh(
        match fin_r {
            fish_medium::FinR::Default => "npc.marlin.fin_r",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}
////
pub fn mesh_dragon_head(head: dragon::Head) -> Mesh<FigurePipeline> {
    load_mesh(
        match head {
            dragon::Head::Default => "npc.dragon.head",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_dragon_chest_front(chest_front: dragon::ChestFront) -> Mesh<FigurePipeline> {
    load_mesh(
        match chest_front {
            dragon::ChestFront::Default => "npc.dragon.chest_front",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_dragon_chest_rear(chest_rear: dragon::ChestRear) -> Mesh<FigurePipeline> {
    load_mesh(
        match chest_rear {
            dragon::ChestRear::Default => "npc.dragon.chest_rear",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_dragon_tail_front(tail_front: dragon::TailFront) -> Mesh<FigurePipeline> {
    load_mesh(
        match tail_front {
            dragon::TailFront::Default => "npc.dragon.tail_front",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_dragon_tail_rear(tail_rear: dragon::TailRear) -> Mesh<FigurePipeline> {
    load_mesh(
        match tail_rear {
            dragon::TailRear::Default => "npc.dragon.tail_rear",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_dragon_wing_in_l(wing_in_l: dragon::WingInL) -> Mesh<FigurePipeline> {
    load_mesh(
        match wing_in_l {
            dragon::WingInL::Default => "npc.dragon.wing_in_l",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_dragon_wing_in_r(wing_in_r: dragon::WingInR) -> Mesh<FigurePipeline> {
    load_mesh(
        match wing_in_r {
            dragon::WingInR::Default => "npc.dragon.wing_in_r",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_dragon_wing_out_l(wing_out_l: dragon::WingOutL) -> Mesh<FigurePipeline> {
    load_mesh(
        match wing_out_l {
            dragon::WingOutL::Default => "npc.dragon.wing_out_l",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_dragon_wing_out_r(wing_out_r: dragon::WingOutR) -> Mesh<FigurePipeline> {
    load_mesh(
        match wing_out_r {
            dragon::WingOutR::Default => "npc.dragon.wing_out_r",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_dragon_foot_fl(foot_fl: dragon::FootFL) -> Mesh<FigurePipeline> {
    load_mesh(
        match foot_fl {
            dragon::FootFL::Default => "npc.dragon.foot_fl",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_dragon_foot_fr(foot_fr: dragon::FootFR) -> Mesh<FigurePipeline> {
    load_mesh(
        match foot_fr {
            dragon::FootFR::Default => "npc.dragon.foot_fr",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_dragon_foot_bl(foot_bl: dragon::FootBL) -> Mesh<FigurePipeline> {
    load_mesh(
        match foot_bl {
            dragon::FootBL::Default => "npc.dragon.foot_bl",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_dragon_foot_br(foot_br: dragon::FootBR) -> Mesh<FigurePipeline> {
    load_mesh(
        match foot_br {
            dragon::FootBR::Default => "npc.dragon.foot_br",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

////
pub fn mesh_bird_small_head(head: bird_small::Head) -> Mesh<FigurePipeline> {
    load_mesh(
        match head {
            bird_small::Head::Default => "npc.crow.head",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_bird_small_torso(torso: bird_small::Torso) -> Mesh<FigurePipeline> {
    load_mesh(
        match torso {
            bird_small::Torso::Default => "npc.crow.torso",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_bird_small_wing_l(wing_l: bird_small::WingL) -> Mesh<FigurePipeline> {
    load_mesh(
        match wing_l {
            bird_small::WingL::Default => "npc.crow.wing_l",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_bird_small_wing_r(wing_r: bird_small::WingR) -> Mesh<FigurePipeline> {
    load_mesh(
        match wing_r {
            bird_small::WingR::Default => "npc.crow.wing_r",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}
////
pub fn mesh_fish_small_torso(torso: fish_small::Torso) -> Mesh<FigurePipeline> {
    load_mesh(
        match torso {
            fish_small::Torso::Default => "npc.cardinalfish.torso",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_fish_small_tail(tail: fish_small::Tail) -> Mesh<FigurePipeline> {
    load_mesh(
        match tail {
            fish_small::Tail::Default => "npc.cardinalfish.tail",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}
////
pub fn mesh_biped_large_head(head: biped_large::Head) -> Mesh<FigurePipeline> {
    load_mesh(
        match head {
            biped_large::Head::Default => "npc.knight.head",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_biped_large_upper_torso(upper_torso: biped_large::UpperTorso) -> Mesh<FigurePipeline> {
    load_mesh(
        match upper_torso {
            biped_large::UpperTorso::Default => "npc.knight.upper_torso",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_biped_large_lower_torso(lower_torso: biped_large::LowerTorso) -> Mesh<FigurePipeline> {
    load_mesh(
        match lower_torso {
            biped_large::LowerTorso::Default => "npc.knight.lower_torso",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_biped_large_shoulder_l(shoulder_l: biped_large::ShoulderL) -> Mesh<FigurePipeline> {
    load_mesh(
        match shoulder_l {
            biped_large::ShoulderL::Default => "npc.knight.shoulder_l",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_biped_large_shoulder_r(shoulder_r: biped_large::ShoulderR) -> Mesh<FigurePipeline> {
    load_mesh(
        match shoulder_r {
            biped_large::ShoulderR::Default => "npc.knight.shoulder_r",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_biped_large_hand_l(hand_l: biped_large::HandL) -> Mesh<FigurePipeline> {
    load_mesh(
        match hand_l {
            biped_large::HandL::Default => "npc.knight.hand_l",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_biped_large_hand_r(hand_r: biped_large::HandR) -> Mesh<FigurePipeline> {
    load_mesh(
        match hand_r {
            biped_large::HandR::Default => "npc.knight.hand_r",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_biped_large_leg_l(leg_l: biped_large::LegL) -> Mesh<FigurePipeline> {
    load_mesh(
        match leg_l {
            biped_large::LegL::Default => "npc.knight.leg_l",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_biped_large_leg_r(leg_r: biped_large::LegR) -> Mesh<FigurePipeline> {
    load_mesh(
        match leg_r {
            biped_large::LegR::Default => "npc.knight.leg_r",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_biped_large_foot_l(foot_l: biped_large::FootL) -> Mesh<FigurePipeline> {
    load_mesh(
        match foot_l {
            biped_large::FootL::Default => "npc.knight.foot_l",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

pub fn mesh_biped_large_foot_r(foot_r: biped_large::FootR) -> Mesh<FigurePipeline> {
    load_mesh(
        match foot_r {
            biped_large::FootR::Default => "npc.knight.foot_r",
        },
        Vec3::new(-7.0, -6.0, -6.0),
    )
}

////
pub fn mesh_object(obj: object::Body) -> Mesh<FigurePipeline> {
    use object::Body;

    let (name, offset) = match obj {
        Body::Arrow => ("weapon.projectile.simple-arrow", Vec3::new(-5.5, -5.5, 0.0)),
        Body::Bomb => ("object.bomb", Vec3::new(-5.5, -5.5, 0.0)),
        Body::Scarecrow => ("object.scarecrow", Vec3::new(-9.5, -4.0, 0.0)),
        Body::Cauldron => ("object.cauldron", Vec3::new(-10.0, -10.0, 0.0)),
        Body::ChestVines => ("object.chest_vines", Vec3::new(-7.5, -6.0, 0.0)),
        Body::Chest => ("object.chest", Vec3::new(-7.5, -6.0, 0.0)),
        Body::ChestDark => ("object.chest_dark", Vec3::new(-7.5, -6.0, 0.0)),
        Body::ChestDemon => ("object.chest_demon", Vec3::new(-7.5, -6.0, 0.0)),
        Body::ChestGold => ("object.chest_gold", Vec3::new(-7.5, -6.0, 0.0)),
        Body::ChestLight => ("object.chest_light", Vec3::new(-7.5, -6.0, 0.0)),
        Body::ChestOpen => ("object.chest_open", Vec3::new(-7.5, -6.0, 0.0)),
        Body::ChestSkull => ("object.chest_skull", Vec3::new(-7.5, -6.0, 0.0)),
        Body::Pumpkin => ("object.pumpkin", Vec3::new(-5.5, -4.0, 0.0)),
        Body::Pumpkin2 => ("object.pumpkin_2", Vec3::new(-5.0, -4.0, 0.0)),
        Body::Pumpkin3 => ("object.pumpkin_3", Vec3::new(-5.0, -4.0, 0.0)),
        Body::Pumpkin4 => ("object.pumpkin_4", Vec3::new(-5.0, -4.0, 0.0)),
        Body::Pumpkin5 => ("object.pumpkin_5", Vec3::new(-4.0, -5.0, 0.0)),
        Body::Campfire => ("object.campfire", Vec3::new(-9.0, -10.0, 0.0)),
        Body::LanternGround => ("object.lantern_ground", Vec3::new(-3.5, -3.5, 0.0)),
        Body::LanternGroundOpen => ("object.lantern_ground_open", Vec3::new(-3.5, -3.5, 0.0)),
        Body::LanternStanding => ("object.lantern_standing", Vec3::new(-7.5, -3.5, 0.0)),
        Body::LanternStanding2 => ("object.lantern_standing_2", Vec3::new(-11.5, -3.5, 0.0)),
        Body::PotionRed => ("object.potion_red", Vec3::new(-2.0, -2.0, 0.0)),
        Body::PotionBlue => ("object.potion_blue", Vec3::new(-2.0, -2.0, 0.0)),
        Body::PotionGreen => ("object.potion_green", Vec3::new(-2.0, -2.0, 0.0)),
        Body::Crate => ("object.crate", Vec3::new(-7.0, -7.0, 0.0)),
        Body::Tent => ("object.tent", Vec3::new(-18.5, -19.5, 0.0)),
        Body::WindowSpooky => ("object.window_spooky", Vec3::new(-15.0, -1.5, -1.0)),
        Body::DoorSpooky => ("object.door_spooky", Vec3::new(-15.0, -4.5, 0.0)),
        Body::Table => ("object.table", Vec3::new(-12.0, -8.0, 0.0)),
        Body::Table2 => ("object.table_2", Vec3::new(-8.0, -8.0, 0.0)),
        Body::Table3 => ("object.table_3", Vec3::new(-10.0, -10.0, 0.0)),
        Body::Drawer => ("object.drawer", Vec3::new(-11.0, -7.5, 0.0)),
        Body::BedBlue => ("object.bed_human_blue", Vec3::new(-11.0, -15.0, 0.0)),
        Body::Anvil => ("object.anvil", Vec3::new(-3.0, -7.0, 0.0)),
        Body::Gravestone => ("object.gravestone", Vec3::new(-5.0, -2.0, 0.0)),
        Body::Gravestone2 => ("object.gravestone_2", Vec3::new(-8.5, -3.0, 0.0)),
        Body::Chair => ("object.chair", Vec3::new(-5.0, -4.5, 0.0)),
        Body::Chair2 => ("object.chair_2", Vec3::new(-5.0, -4.5, 0.0)),
        Body::Chair3 => ("object.chair_3", Vec3::new(-5.0, -4.5, 0.0)),
        Body::Bench => ("object.bench", Vec3::new(-8.8, -5.0, 0.0)),
        Body::Carpet => ("object.carpet", Vec3::new(-14.0, -14.0, -0.5)),
        Body::Bedroll => ("object.bedroll", Vec3::new(-11.0, -19.5, -0.5)),
        Body::CarpetHumanRound => ("object.carpet_human_round", Vec3::new(-14.0, -14.0, -0.5)),
        Body::CarpetHumanSquare => ("object.carpet_human_square", Vec3::new(-13.5, -14.0, -0.5)),
        Body::CarpetHumanSquare2 => (
            "object.carpet_human_square_2",
            Vec3::new(-13.5, -14.0, -0.5),
        ),
        Body::CarpetHumanSquircle => (
            "object.carpet_human_squircle",
            Vec3::new(-21.0, -21.0, -0.5),
        ),
        Body::Pouch => ("object.pouch", Vec3::new(-5.5, -4.5, 0.0)),
        Body::CraftingBench => ("object.crafting_bench", Vec3::new(-9.5, -7.0, 0.0)),
        Body::ArrowSnake => ("weapon.projectile.snake-arrow", Vec3::new(-1.5, -6.5, 0.0)),
        Body::BoltFire => ("weapon.projectile.fire-bolt", Vec3::new(-3.0, -5.5, -3.0)),
    };
    load_mesh(name, offset)
}
