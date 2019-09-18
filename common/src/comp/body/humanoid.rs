use rand::{seq::SliceRandom, thread_rng, Rng};
use vek::Rgb;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Body {
    pub race: Race,
    pub body_type: BodyType,
    pub chest: Chest,
    pub belt: Belt,
    pub pants: Pants,
    pub hand: Hand,
    pub foot: Foot,
    pub shoulder: Shoulder,
    pub hair_style: u8,
    pub beard: u8,
    pub eyebrows: Eyebrows,
    pub accessory: u8,
    pub hair_color: u8,
    pub skin: u8,
    pub eye_color: u8,
}

impl Body {
    pub fn random() -> Self {
        let mut rng = thread_rng();
        let race = *(&ALL_RACES).choose(&mut rng).unwrap();
        let body_type = *(&ALL_BODY_TYPES).choose(&mut rng).unwrap();
        Self {
            race,
            body_type,
            chest: *(&ALL_CHESTS).choose(&mut rng).unwrap(),
            belt: *(&ALL_BELTS).choose(&mut rng).unwrap(),
            pants: *(&ALL_PANTS).choose(&mut rng).unwrap(),
            hand: *(&ALL_HANDS).choose(&mut rng).unwrap(),
            foot: *(&ALL_FEET).choose(&mut rng).unwrap(),
            shoulder: *(&ALL_SHOULDERS).choose(&mut rng).unwrap(),
            hair_style: rng.gen_range(0, race.num_hair_styles(body_type)),
            beard: rng.gen_range(0, race.num_beards(body_type)),
            eyebrows: *(&ALL_EYEBROWS).choose(&mut rng).unwrap(),
            accessory: rng.gen_range(0, race.num_accessories(body_type)),
            hair_color: rng.gen_range(0, race.num_hair_colors()) as u8,
            skin: rng.gen_range(0, race.num_skin_colors()) as u8,
            eye_color: rng.gen_range(0, race.num_eye_colors()) as u8,
        }
    }

    pub fn validate(&mut self) {
        self.hair_style = self
            .hair_style
            .min(self.race.num_hair_styles(self.body_type) - 1);
        self.beard = self.beard.min(self.race.num_beards(self.body_type) - 1);
        self.hair_color = self.hair_color.min(self.race.num_hair_colors() - 1);
        self.skin = self.skin.min(self.race.num_skin_colors() - 1);
        self.eye_color = self.hair_style.min(self.race.num_eye_colors() - 1);
        self.accessory = self
            .accessory
            .min(self.race.num_accessories(self.body_type) - 1);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Race {
    Danari,
    Dwarf,
    Elf,
    Human,
    Orc,
    Undead,
}
pub const ALL_RACES: [Race; 6] = [
    Race::Danari,
    Race::Dwarf,
    Race::Elf,
    Race::Human,
    Race::Orc,
    Race::Undead,
];

// Hair Colors
pub const DANARI_HAIR_COLORS: [(u8, u8, u8); 17] = [
    (198, 169, 113), // Philosopher's Grey
    (245, 232, 175), // Cream Blonde
    (228, 208, 147), // Gold Blonde
    (228, 223, 141), // Platinum Blonde
    (199, 131, 58),  // Summer Blonde
    (107, 76, 51),   // Oak Brown
    (203, 154, 98),  // Light Brown
    (64, 32, 18),    // Chocolate Brown
    (86, 72, 71),    // Ash Brown
    (57, 56, 61),    // Raven Black
    (101, 83, 95),   // Matte Purple
    (101, 57, 90),   // Witch Purple
    (107, 32, 60),   // Grape Purple
    (168, 45, 47),   // Lobster Red
    (135, 38, 39),   // Dark Red
    (88, 26, 29),    // Wine Red
    (146, 32, 32),   // Autumn Red
];
pub const DWARF_HAIR_COLORS: [(u8, u8, u8); 21] = [
    (245, 232, 175), // Cream Blonde
    (228, 208, 147), // Gold Blonde
    (228, 223, 141), // Platinum Blonde
    (199, 131, 58),  // Summer Blonde
    (107, 76, 51),   // Oak Brown
    (203, 154, 98),  // Light Brown
    (64, 32, 18),    // Chocolate Brown
    (86, 72, 71),    // Ash Brown
    (57, 56, 61),    // Raven Black
    (101, 83, 95),   // Matte Purple
    (101, 57, 90),   // Witch Purple
    (168, 45, 47),   // Lobster Red
    (135, 38, 39),   // Dark Red
    (88, 26, 29),    // Wine Red
    (191, 228, 254), // Ice NobleBlue
    (92, 80, 144),   // Kingfisher Blue
    (146, 198, 238), // Lagoon Blue
    (174, 148, 161), // Matte Pink
    (163, 186, 192), // Matte Green
    (84, 139, 107),  // Grass Green
    (48, 61, 52),    // Dark Green
];
pub const ELF_HAIR_COLORS: [(u8, u8, u8); 24] = [
    (66, 83, 113),   // Mysterious Blue
    (13, 76, 41),    // Rainforest Green
    (245, 232, 175), // Cream Blonde
    (228, 208, 147), // Gold Blonde
    (228, 223, 141), // Platinum Blonde
    (199, 131, 58),  // Summer Blonde
    (107, 76, 51),   // Oak Brown
    (203, 154, 98),  // Light Brown
    (64, 32, 18),    // Chocolate Brown
    (86, 72, 71),    // Ash Brown
    (57, 56, 61),    // Raven Black
    (101, 83, 95),   // Matte Purple
    (101, 57, 90),   // Witch Purple
    (168, 45, 47),   // Lobster Red
    (135, 38, 39),   // Dark Red
    (88, 26, 29),    // Wine Red
    (191, 228, 254), // Ice Blue
    (92, 80, 144),   // Kingfisher Blue
    (146, 198, 238), // Lagoon Blue
    (224, 182, 184), // Candy Pink
    (174, 148, 161), // Matte Pink
    (163, 186, 192), // Matte Green
    (84, 139, 107),  // Grass Green
    (48, 61, 52),    // Dark Green
];
pub const HUMAN_HAIR_COLORS: [(u8, u8, u8); 22] = [
    (245, 232, 175), // Cream Blonde
    (228, 208, 147), // Gold Blonde
    (228, 223, 141), // Platinum Blonde
    (199, 131, 58),  // Summer Blonde
    (107, 76, 51),   // Oak Brown
    (203, 154, 98),  // Light Brown
    (64, 32, 18),    // Chocolate Brown
    (86, 72, 71),    // Ash Brown
    (57, 56, 61),    // Raven Black
    (101, 83, 95),   // Matte Purple
    (101, 57, 90),   // Witch Purple
    (168, 45, 47),   // Lobster Red
    (135, 38, 39),   // Dark Red
    (88, 26, 29),    // Wine Red
    (191, 228, 254), // Ice Blue
    (92, 80, 144),   // Kingfisher Blue
    (146, 198, 238), // Lagoon Blue
    (224, 182, 184), // Candy Pink
    (174, 148, 161), // Matte Pink
    (163, 186, 192), // Matte Green
    (84, 139, 107),  // Grass Green
    (48, 61, 52),    // Dark Green
];
pub const ORC_HAIR_COLORS: [(u8, u8, u8); 15] = [
    (66, 66, 59),   // Wise Grey
    (125, 111, 51), // Muddy Blonde
    (199, 131, 58), // Summer Blonde
    (107, 76, 51),  // Oak Brown
    (203, 154, 98), // Light Brown
    (64, 32, 18),   // Chocolate Brown
    (54, 30, 26),   // Dark Chocolate
    (86, 72, 71),   // Ash Brown
    (57, 56, 61),   // Raven Black
    (101, 83, 95),  // Matte Purple
    (101, 57, 90),  // Witch Purple
    (168, 45, 47),  // Lobster Red
    (135, 38, 39),  // Dark Red
    (88, 26, 29),   // Wine Red
    (66, 83, 113),  // Mysterious Blue
];
pub const UNDEAD_HAIR_COLORS: [(u8, u8, u8); 25] = [
    (245, 232, 175), // Cream Blonde
    (228, 208, 147), // Gold Blonde
    (228, 223, 141), // Platinum Blonde
    (199, 131, 58),  // Summer Blonde
    (107, 76, 51),   // Oak Brown
    (203, 154, 98),  // Light Brown
    (64, 32, 18),    // Chocolate Brown
    (86, 72, 71),    // Ash Brown
    (57, 56, 61),    // Raven Black
    (101, 83, 95),   // Matte Purple
    (101, 57, 90),   // Witch Purple
    (111, 54, 117),  // Punky Purple
    (168, 45, 47),   // Lobster Red
    (135, 38, 39),   // Dark Red
    (88, 26, 29),    // Wine Red
    (191, 228, 254), // Ice Blue
    (92, 80, 144),   // Kingfisher Blue
    (146, 198, 238), // Lagoon Blue
    (66, 66, 59),    // Decayed Grey
    (224, 182, 184), // Candy Pink
    (174, 148, 161), // Matte Pink
    (0, 131, 122),   // Rotten Green
    (163, 186, 192), // Matte Green
    (84, 139, 107),  // Grass Green
    (48, 61, 52),    // Dark Green
];

// Skin colors
pub const DANARI_SKIN_COLORS: [Skin; 4] = [
    Skin::DanariOne,
    Skin::DanariTwo,
    Skin::DanariThree,
    Skin::DanariFour,
];
pub const DWARF_SKIN_COLORS: [Skin; 7] = [
    Skin::White,
    Skin::Tanned,
    Skin::Brown,
    Skin::TannedBrown,
    Skin::TannedDarkBrown,
    Skin::Iron,
    Skin::Steel,
];
pub const ELF_SKIN_COLORS: [Skin; 7] = [
    Skin::ElfOne,
    Skin::ElfTwo,
    Skin::ElfThree,
    Skin::White,
    Skin::Tanned,
    Skin::Brown,
    Skin::TannedBrown,
];
pub const HUMAN_SKIN_COLORS: [Skin; 9] = [
    Skin::Pale,
    Skin::White,
    Skin::Tanned,
    Skin::Brown,
    Skin::TannedBrown,
    Skin::TannedDarkBrown,
    Skin::Black,
    Skin::Blacker,
    Skin::TannedBlack,
];
pub const ORC_SKIN_COLORS: [Skin; 3] = [Skin::OrcOne, Skin::OrcTwo, Skin::OrcThree];
pub const UNDEAD_SKIN_COLORS: [Skin; 3] = [Skin::UndeadOne, Skin::UndeadTwo, Skin::UndeadThree];

// Eye colors
pub const DANARI_EYE_COLORS: [EyeColor; 3] = [
    EyeColor::CuriousGreen,
    EyeColor::LoyalBrown,
    EyeColor::ViciousRed,
];
pub const DWARF_EYE_COLORS: [EyeColor; 3] = [
    EyeColor::CuriousGreen,
    EyeColor::LoyalBrown,
    EyeColor::NobleBlue,
];
pub const ELF_EYE_COLORS: [EyeColor; 3] = [
    EyeColor::NobleBlue,
    EyeColor::CuriousGreen,
    EyeColor::LoyalBrown,
];
pub const HUMAN_EYE_COLORS: [EyeColor; 3] = [
    EyeColor::NobleBlue,
    EyeColor::CuriousGreen,
    EyeColor::LoyalBrown,
];
pub const ORC_EYE_COLORS: [EyeColor; 2] = [EyeColor::LoyalBrown, EyeColor::ViciousRed];
pub const UNDEAD_EYE_COLORS: [EyeColor; 5] = [
    EyeColor::ViciousRed,
    EyeColor::PumpkinOrange,
    EyeColor::GhastlyYellow,
    EyeColor::MagicPurple,
    EyeColor::ToxicGreen,
];

impl Race {
    fn hair_colors(self) -> &'static [(u8, u8, u8)] {
        match self {
            Race::Danari => &DANARI_HAIR_COLORS,
            Race::Dwarf => &DWARF_HAIR_COLORS,
            Race::Elf => &ELF_HAIR_COLORS,
            Race::Human => &HUMAN_HAIR_COLORS,
            Race::Orc => &ORC_HAIR_COLORS,
            Race::Undead => &UNDEAD_HAIR_COLORS,
        }
    }
    fn skin_colors(self) -> &'static [Skin] {
        match self {
            Race::Danari => &DANARI_SKIN_COLORS,
            Race::Dwarf => &DWARF_SKIN_COLORS,
            Race::Elf => &ELF_SKIN_COLORS,
            Race::Human => &HUMAN_SKIN_COLORS,
            Race::Orc => &ORC_SKIN_COLORS,
            Race::Undead => &UNDEAD_SKIN_COLORS,
        }
    }
    fn eye_colors(self) -> &'static [EyeColor] {
        match self {
            Race::Danari => &DANARI_EYE_COLORS,
            Race::Dwarf => &DWARF_EYE_COLORS,
            Race::Elf => &ELF_EYE_COLORS,
            Race::Human => &HUMAN_EYE_COLORS,
            Race::Orc => &ORC_EYE_COLORS,
            Race::Undead => &UNDEAD_EYE_COLORS,
        }
    }
    pub fn hair_color(self, val: u8) -> Rgb<u8> {
        self.hair_colors()
            .get(val as usize)
            .copied()
            .unwrap_or((0, 0, 0))
            .into()
    }
    pub fn num_hair_colors(self) -> u8 {
        self.hair_colors().len() as u8
    }
    pub fn skin_color(self, val: u8) -> Skin {
        self.skin_colors()
            .get(val as usize)
            .copied()
            .unwrap_or(Skin::Tanned)
    }
    pub fn num_skin_colors(self) -> u8 {
        self.skin_colors().len() as u8
    }
    pub fn eye_color(self, val: u8) -> EyeColor {
        self.eye_colors()
            .get(val as usize)
            .copied()
            .unwrap_or(EyeColor::NobleBlue)
    }
    pub fn num_eye_colors(self) -> u8 {
        self.eye_colors().len() as u8
    }
    pub fn num_hair_styles(self, body_type: BodyType) -> u8 {
        match (self, body_type) {
            (Race::Danari, BodyType::Female) => 2,
            (Race::Danari, BodyType::Male) => 2,
            (Race::Dwarf, BodyType::Female) => 2,
            (Race::Dwarf, BodyType::Male) => 3,
            (Race::Elf, BodyType::Female) => 21,
            (Race::Elf, BodyType::Male) => 1,
            (Race::Human, BodyType::Female) => 19,
            (Race::Human, BodyType::Male) => 3,
            (Race::Orc, BodyType::Female) => 1,
            (Race::Orc, BodyType::Male) => 2,
            (Race::Undead, BodyType::Female) => 4,
            (Race::Undead, BodyType::Male) => 3,
        }
    }
    pub fn num_accessories(self, body_type: BodyType) -> u8 {
        match (self, body_type) {
            (Race::Danari, BodyType::Female) => 1,
            (Race::Danari, BodyType::Male) => 1,
            (Race::Dwarf, BodyType::Female) => 1,
            (Race::Dwarf, BodyType::Male) => 1,
            (Race::Elf, BodyType::Female) => 1,
            (Race::Elf, BodyType::Male) => 1,
            (Race::Human, BodyType::Female) => 1,
            (Race::Human, BodyType::Male) => 1,
            (Race::Orc, BodyType::Female) => 3,
            (Race::Orc, BodyType::Male) => 3,
            (Race::Undead, BodyType::Female) => 1,
            (Race::Undead, BodyType::Male) => 1,
        }
    }
    pub fn num_beards(self, body_type: BodyType) -> u8 {
        match (self, body_type) {
            (Race::Danari, BodyType::Female) => 1,
            (Race::Danari, BodyType::Male) => 1,
            (Race::Dwarf, BodyType::Female) => 1,
            (Race::Dwarf, BodyType::Male) => 20,
            (Race::Elf, BodyType::Female) => 1,
            (Race::Elf, BodyType::Male) => 1,
            (Race::Human, BodyType::Female) => 1,
            (Race::Human, BodyType::Male) => 2,
            (Race::Orc, BodyType::Female) => 1,
            (Race::Orc, BodyType::Male) => 2,
            (Race::Undead, BodyType::Female) => 1,
            (Race::Undead, BodyType::Male) => 1,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BodyType {
    Female,
    Male,
}
pub const ALL_BODY_TYPES: [BodyType; 2] = [BodyType::Female, BodyType::Male];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Chest {
    Blue,
    Brown,
    Dark,
    Green,
    Orange,
}
pub const ALL_CHESTS: [Chest; 5] = [
    Chest::Blue,
    Chest::Brown,
    Chest::Dark,
    Chest::Green,
    Chest::Orange,
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Belt {
    Dark,
}
pub const ALL_BELTS: [Belt; 1] = [
    //Belt::Default,
    Belt::Dark,
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pants {
    Blue,
    Brown,
    Dark,
    Green,
    Orange,
}
pub const ALL_PANTS: [Pants; 5] = [
    Pants::Blue,
    Pants::Brown,
    Pants::Dark,
    Pants::Green,
    Pants::Orange,
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hand {
    Bare,
    Dark,
}
pub const ALL_HANDS: [Hand; 2] = [Hand::Bare, Hand::Dark];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Foot {
    Bare,
    Dark,
}
pub const ALL_FEET: [Foot; 2] = [Foot::Bare, Foot::Dark];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Shoulder {
    None,
    Brown1,
}
pub const ALL_SHOULDERS: [Shoulder; 2] = [Shoulder::None, Shoulder::Brown1];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Eyebrows {
    Yup,
}
pub const ALL_EYEBROWS: [Eyebrows; 1] = [Eyebrows::Yup];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EyeColor {
    VigorousBlack,
    NobleBlue,
    CuriousGreen,
    LoyalBrown,
    ViciousRed,
    PumpkinOrange,
    GhastlyYellow,
    MagicPurple,
    ToxicGreen,
}
impl EyeColor {
    pub fn light_rgb(self) -> Rgb<u8> {
        match self {
            EyeColor::VigorousBlack => Rgb::new(71, 59, 49),
            EyeColor::NobleBlue => Rgb::new(75, 158, 191),
            EyeColor::CuriousGreen => Rgb::new(110, 167, 113),
            EyeColor::LoyalBrown => Rgb::new(73, 42, 36),
            EyeColor::ViciousRed => Rgb::new(182, 0, 0),
            EyeColor::PumpkinOrange => Rgb::new(220, 156, 19),
            EyeColor::GhastlyYellow => Rgb::new(221, 225, 31),
            EyeColor::MagicPurple => Rgb::new(137, 4, 177),
            EyeColor::ToxicGreen => Rgb::new(1, 223, 1),
        }
    }
    pub fn dark_rgb(self) -> Rgb<u8> {
        match self {
            EyeColor::VigorousBlack => Rgb::new(32, 32, 32),
            EyeColor::NobleBlue => Rgb::new(62, 130, 159),
            EyeColor::CuriousGreen => Rgb::new(81, 124, 84),
            EyeColor::LoyalBrown => Rgb::new(54, 30, 26),
            EyeColor::ViciousRed => Rgb::new(148, 0, 0),
            EyeColor::PumpkinOrange => Rgb::new(209, 145, 18),
            EyeColor::GhastlyYellow => Rgb::new(205, 212, 29),
            EyeColor::MagicPurple => Rgb::new(110, 3, 143),
            EyeColor::ToxicGreen => Rgb::new(1, 185, 1),
        }
    }
    pub fn white_rgb(self) -> Rgb<u8> {
        Rgb::new(255, 255, 255)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Accessory {
    Nothing,
    Some,
}
pub const ALL_ACCESSORIES: [Accessory; 2] = [Accessory::Nothing, Accessory::Some];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Skin {
    Pale,
    White,
    Tanned,
    Brown,
    TannedBrown,
    TannedDarkBrown,
    Black,
    Blacker,
    TannedBlack,
    Iron,
    Steel,
    DanariOne,
    DanariTwo,
    DanariThree,
    DanariFour,
    ElfOne,
    ElfTwo,
    ElfThree,
    OrcOne,
    OrcTwo,
    OrcThree,
    UndeadOne,
    UndeadTwo,
    UndeadThree,
}
impl Skin {
    pub fn rgb(self) -> Rgb<u8> {
        let color = match self {
            Self::Pale => (252, 211, 179),
            Self::White => (253, 195, 164),
            Self::Tanned => (253, 206, 150),
            Self::Brown => (225, 177, 128),
            Self::TannedBrown => (219, 165, 131),
            Self::TannedDarkBrown => (189, 131, 93),
            Self::Black => (168, 109, 79),
            Self::Blacker => (123, 68, 55),
            Self::TannedBlack => (118, 60, 36),
            Self::Iron => (135, 113, 95),
            Self::Steel => (108, 94, 86),
            Self::DanariOne => (104, 168, 196),
            Self::DanariTwo => (30, 149, 201),
            Self::DanariThree => (57, 120, 148),
            Self::DanariFour => (40, 85, 105),
            Self::ElfOne => (176, 161, 181),
            Self::ElfTwo => (132, 139, 161),
            Self::ElfThree => (138, 119, 201),
            Self::OrcOne => (67, 141, 46),
            Self::OrcTwo => (82, 117, 36),
            Self::OrcThree => (71, 94, 42),
            Self::UndeadOne => (255, 255, 255),
            Self::UndeadTwo => (178, 178, 178),
            Self::UndeadThree => (145, 135, 121),
        };
        Rgb::from(color)
    }
    pub fn light_rgb(self) -> Rgb<u8> {
        let color = match self {
            Self::Pale => (255, 165, 165),
            Self::White => (255, 165, 165),
            Self::Tanned => (253, 206, 150),
            Self::Brown => (225, 177, 128),
            Self::TannedBrown => (219, 165, 131),
            Self::TannedDarkBrown => (189, 131, 93),
            Self::Black => (168, 109, 79),
            Self::Blacker => (123, 68, 55),
            Self::TannedBlack => (118, 60, 36),
            Self::Iron => (135, 113, 95),
            Self::Steel => (108, 94, 86),
            Self::DanariOne => (104, 168, 196),
            Self::DanariTwo => (30, 149, 201),
            Self::DanariThree => (57, 120, 148),
            Self::DanariFour => (40, 85, 105),
            Self::ElfOne => (176, 161, 181),
            Self::ElfTwo => (132, 139, 161),
            Self::ElfThree => (138, 119, 201),
            Self::OrcOne => (77, 150, 51),
            Self::OrcTwo => (85, 124, 37),
            Self::OrcThree => (73, 100, 43),
            Self::UndeadOne => (255, 255, 255),
            Self::UndeadTwo => (178, 178, 178),
            Self::UndeadThree => (145, 135, 121),
        };
        Rgb::from(color)
    }
    pub fn dark_rgb(self) -> Rgb<u8> {
        let color = match self {
            Self::Pale => (207, 173, 147),
            Self::White => (212, 162, 138),
            Self::Tanned => (207, 167, 123),
            Self::Brown => (187, 147, 107),
            Self::TannedBrown => (219, 165, 131),
            Self::TannedDarkBrown => (157, 108, 77),
            Self::Black => (168, 109, 79),
            Self::Blacker => (123, 68, 55),
            Self::TannedBlack => (118, 60, 36),
            Self::Iron => (135, 113, 95),
            Self::Steel => (108, 94, 86),
            Self::DanariOne => (104, 168, 196),
            Self::DanariTwo => (30, 149, 201),
            Self::DanariThree => (57, 120, 148),
            Self::DanariFour => (40, 85, 105),
            Self::ElfOne => (176, 161, 181),
            Self::ElfTwo => (132, 139, 161),
            Self::ElfThree => (138, 119, 201),
            Self::OrcOne => (68, 129, 44),
            Self::OrcTwo => (77, 111, 34),
            Self::OrcThree => (68, 91, 40),
            Self::UndeadOne => (255, 255, 255),
            Self::UndeadTwo => (178, 178, 178),
            Self::UndeadThree => (145, 135, 121),
        };
        Rgb::from(color)
    }
}
