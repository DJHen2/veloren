use rand::{seq::SliceRandom, thread_rng};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Body {
    pub species: Species,
    pub body_type: BodyType,
}

impl Body {
    pub fn random() -> Self {
        let mut rng = thread_rng();
        let species = *(&ALL_SPECIES).choose(&mut rng).unwrap();
        Self::random_with(&mut rng, &species)
    }

    #[inline]
    pub fn random_with(rng: &mut impl rand::Rng, &species: &Species) -> Self {
        let body_type = *(&ALL_BODY_TYPES).choose(rng).unwrap();
        Self { species, body_type }
    }
}

impl From<Body> for super::Body {
    fn from(body: Body) -> Self { super::Body::QuadrupedMedium(body) }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum Species {
    Grolgar = 0,
    Saber = 1,
    Tiger = 2,
    Tuskram = 3,
    Lion = 6,
    Tarasque = 7,
}

/// Data representing per-species generic data.
///
/// NOTE: Deliberately don't (yet?) implement serialize.
#[derive(Clone, Debug, Deserialize)]
pub struct AllSpecies<SpeciesMeta> {
    pub grolgar: SpeciesMeta,
    pub saber: SpeciesMeta,
    pub tiger: SpeciesMeta,
    pub tuskram: SpeciesMeta,
    pub lion: SpeciesMeta,
    pub tarasque: SpeciesMeta,
}

impl<'a, SpeciesMeta> core::ops::Index<&'a Species> for AllSpecies<SpeciesMeta> {
    type Output = SpeciesMeta;

    #[inline]
    fn index(&self, &index: &'a Species) -> &Self::Output {
        match index {
            Species::Grolgar => &self.grolgar,
            Species::Saber => &self.saber,
            Species::Tiger => &self.tiger,
            Species::Tuskram => &self.tuskram,
            Species::Lion => &self.lion,
            Species::Tarasque => &self.tarasque,
        }
    }
}

pub const ALL_SPECIES: [Species; 6] = [
    Species::Grolgar,
    Species::Saber,
    Species::Tiger,
    Species::Tuskram,
    Species::Lion,
    Species::Tarasque,
];

impl<'a, SpeciesMeta: 'a> IntoIterator for &'a AllSpecies<SpeciesMeta> {
    type IntoIter = std::iter::Copied<std::slice::Iter<'static, Self::Item>>;
    type Item = Species;

    fn into_iter(self) -> Self::IntoIter { ALL_SPECIES.iter().copied() }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum BodyType {
    Female = 0,
    Male = 1,
}
pub const ALL_BODY_TYPES: [BodyType; 2] = [BodyType::Female, BodyType::Male];
