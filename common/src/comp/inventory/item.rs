use crate::{
    assets::{self, Asset},
    comp::{
        body::{humanoid, object},
        projectile, Body, CharacterAbility, HealthChange, HealthSource, Projectile,
    },
    effect::Effect,
    terrain::{Block, BlockKind},
};
use rand::seq::SliceRandom;
use specs::{Component, FlaggedStorage};
use specs_idvs::IDVStorage;
use std::{fs::File, io::BufReader, time::Duration, vec::Vec};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SwordKind {
    BasicSword,
    Rapier,
    Zweihander0,
    WoodTraining,
    Short0,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AxeKind {
    BasicAxe,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HammerKind {
    BasicHammer,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BowKind {
    BasicBow,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DaggerKind {
    BasicDagger,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StaffKind {
    BasicStaff,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShieldKind {
    BasicShield,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolKind {
    Sword(SwordKind),
    Axe(AxeKind),
    Hammer(HammerKind),
    Bow(BowKind),
    Dagger(DaggerKind),
    Staff(StaffKind),
    Shield(ShieldKind),
    Debug(DebugKind),
    /// This is an placeholder item, it is used by non-humanoid npcs to attack
    Empty,
}

impl ToolData {
    pub fn equip_time(&self) -> Duration { Duration::from_millis(self.equip_time_millis) }

    pub fn get_abilities(&self) -> Vec<CharacterAbility> {
        use CharacterAbility::*;
        use DebugKind::*;
        use ToolKind::*;

        match self.kind {
            Sword(_) => vec![
                BasicMelee {
                    buildup_duration: Duration::from_millis(100),
                    recover_duration: Duration::from_millis(500),
                    base_damage: 6,
                },
                DashMelee {
                    buildup_duration: Duration::from_millis(500),
                    recover_duration: Duration::from_millis(500),
                    base_damage: 20,
                },
            ],
            Axe(_) => vec![BasicMelee {
                buildup_duration: Duration::from_millis(700),
                recover_duration: Duration::from_millis(100),
                base_damage: 8,
            }],
            Hammer(_) => vec![BasicMelee {
                buildup_duration: Duration::from_millis(700),
                recover_duration: Duration::from_millis(300),
                base_damage: 10,
            }],
            Bow(_) => vec![BasicRanged {
                projectile: Projectile {
                    hit_ground: vec![projectile::Effect::Stick],
                    hit_wall: vec![projectile::Effect::Stick],
                    hit_entity: vec![
                        projectile::Effect::Damage(HealthChange {
                            // TODO: This should not be fixed (?)
                            amount: -3,
                            cause: HealthSource::Item,
                        }),
                        projectile::Effect::Vanish,
                    ],
                    time_left: Duration::from_secs(15),
                    owner: None,
                },
                projectile_body: Body::Object(object::Body::Arrow),
                recover_duration: Duration::from_millis(300),
            }],
            Dagger(_) => vec![BasicMelee {
                buildup_duration: Duration::from_millis(100),
                recover_duration: Duration::from_millis(400),
                base_damage: 5,
            }],
            Staff(_) => vec![BasicMelee {
                buildup_duration: Duration::from_millis(400),
                recover_duration: Duration::from_millis(300),
                base_damage: 7,
            }],
            Shield(_) => vec![BasicBlock],
            Debug(kind) => match kind {
                DebugKind::Boost => vec![
                    CharacterAbility::Boost {
                        duration: Duration::from_millis(50),
                        only_up: false,
                    },
                    CharacterAbility::Boost {
                        duration: Duration::from_millis(50),
                        only_up: true,
                    },
                ],
                Possess => vec![BasicRanged {
                    projectile: Projectile {
                        hit_ground: vec![projectile::Effect::Stick],
                        hit_wall: vec![projectile::Effect::Stick],
                        hit_entity: vec![projectile::Effect::Stick, projectile::Effect::Possess],
                        time_left: Duration::from_secs(10),
                        owner: None,
                    },
                    projectile_body: Body::Object(object::Body::ArrowSnake),
                    recover_duration: Duration::from_millis(300),
                }],
            },
            Empty => vec![],
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DebugKind {
    Boost,
    Possess,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Armor {
    Shoulder(humanoid::Shoulder),
    Chest(humanoid::Chest),
    Belt(humanoid::Belt),
    Hand(humanoid::Hand),
    Pants(humanoid::Pants),
    Foot(humanoid::Foot),
}

pub type ArmorStats = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Consumable {
    Apple,
    Cheese,
    Potion,
    Mushroom,
    Velorite,
    VeloriteFrag,
    PotionMinor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Utility {
    Collar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ingredient {
    Flower,
    Grass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolData {
    pub kind: ToolKind,
    equip_time_millis: u64,
    // TODO: item specific abilities
}

fn default_amount() -> u32 { 1 }

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    /// Something wieldable
    Tool(ToolData),
    Armor {
        kind: Armor,
        stats: ArmorStats,
    },
    Consumable {
        kind: Consumable,
        effect: Effect,
        #[serde(default = "default_amount")]
        amount: u32,
    },
    Utility {
        kind: Utility,
        #[serde(default = "default_amount")]
        amount: u32,
    },
    Ingredient {
        kind: Ingredient,
        #[serde(default = "default_amount")]
        amount: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Item {
    name: String,
    description: String,
    pub kind: ItemKind,
}

impl Asset for Item {
    const ENDINGS: &'static [&'static str] = &["ron"];

    fn parse(buf_reader: BufReader<File>) -> Result<Self, assets::Error> {
        Ok(ron::de::from_reader(buf_reader).unwrap())
    }
}

impl Item {
    pub fn empty() -> Self {
        Self {
            name: "Empty Item".to_owned(),
            description: "This item may grant abilities, but is invisible".to_owned(),
            kind: ItemKind::Tool(ToolData {
                kind: ToolKind::Empty,
                equip_time_millis: 0,
            }),
        }
    }

    pub fn name(&self) -> &str { &self.name }

    pub fn description(&self) -> &str { &self.description }

    pub fn try_reclaim_from_block(block: Block) -> Option<Self> {
        match block.kind() {
            BlockKind::Apple => Some(assets::load_expect_cloned("common.items.apple")),
            BlockKind::Mushroom => Some(assets::load_expect_cloned("common.items.mushroom")),
            BlockKind::Velorite => Some(assets::load_expect_cloned("common.items.velorite")),
            BlockKind::BlueFlower => Some(assets::load_expect_cloned("common.items.flowers.blue")),
            BlockKind::PinkFlower => Some(assets::load_expect_cloned("common.items.flowers.pink")),
            BlockKind::PurpleFlower => {
                Some(assets::load_expect_cloned("common.items.flowers.purple"))
            },
            BlockKind::RedFlower => Some(assets::load_expect_cloned("common.items.flowers.red")),
            BlockKind::WhiteFlower => {
                Some(assets::load_expect_cloned("common.items.flowers.white"))
            },
            BlockKind::YellowFlower => {
                Some(assets::load_expect_cloned("common.items.flowers.yellow"))
            },
            BlockKind::Sunflower => Some(assets::load_expect_cloned("common.items.flowers.sun")),
            BlockKind::LongGrass => Some(assets::load_expect_cloned("common.items.grasses.long")),
            BlockKind::MediumGrass => {
                Some(assets::load_expect_cloned("common.items.grasses.medium"))
            },
            BlockKind::ShortGrass => Some(assets::load_expect_cloned("common.items.grasses.short")),
            BlockKind::Chest => Some(assets::load_expect_cloned(
                [
                    "common.items.apple",
                    "common.items.velorite",
                    "common.items.veloritefrag",
                    "common.items.cheese",
                    "common.items.potion_minor",
                    "common.items.collar",
                    "common.items.weapons.starter_sword",
                    "common.items.weapons.starter_axe",
                    "common.items.weapons.starter_hammer",
                    "common.items.weapons.starter_bow",
                    "common.items.weapons.starter_staff",
                ]
                .choose(&mut rand::thread_rng())
                .unwrap(), // Can't fail
            )),
            _ => None,
        }
    }
}

impl Component for Item {
    type Storage = FlaggedStorage<Self, IDVStorage<Self>>;
}
