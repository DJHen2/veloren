pub mod house;
pub mod keep;

use vek::*;
use rand::prelude::*;
use common::terrain::Block;
use super::skeleton::*;

pub trait Archetype {
    type Attr: Default;

    fn generate<R: Rng>(rng: &mut R) -> Self where Self: Sized;
    fn draw(
        &self,
        dist: i32,
        offset: Vec2<i32>,
        z: i32,
        branch: &Branch<Self::Attr>,
    ) -> Option<Block>;
}
