use crate::vol::Vox;
use vek::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Material {
    Skin,
    SkinDark,
    SkinLight,
    Hair,
    EyeDark,
    EyeLight,
    EyeWhite,
    /*HairLight,
     *HairDark,
     *Clothing, */
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MatCell {
    None,
    Mat(Material),
    Normal(Rgb<u8>),
}

impl Vox for MatCell {
    fn empty() -> Self { MatCell::None }

    fn is_empty(&self) -> bool { matches!(self, MatCell::None) }
}
