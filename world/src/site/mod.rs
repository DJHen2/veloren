mod dungeon;
mod settlement;

// Reexports
pub use self::{dungeon::Dungeon, settlement::Settlement};

use crate::column::ColumnSample;
use common::{
    generation::ChunkSupplement,
    terrain::Block,
    vol::{BaseVol, ReadVol, RectSizedVol, Vox, WriteVol},
};
use rand::Rng;
use std::{fmt, sync::Arc};
use vek::*;

#[derive(Copy, Clone)]
pub struct BlockMask {
    block: Block,
    priority: i32,
}

impl BlockMask {
    pub fn new(block: Block, priority: i32) -> Self { Self { block, priority } }

    pub fn nothing() -> Self {
        Self {
            block: Block::empty(),
            priority: 0,
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn resolve_with(self, other: Self) -> Self {
        if self.priority >= other.priority {
            self
        } else {
            other
        }
    }

    pub fn finish(self) -> Option<Block> {
        if self.priority > 0 {
            Some(self.block)
        } else {
            None
        }
    }
}

pub struct SpawnRules {
    pub trees: bool,
}

impl Default for SpawnRules {
    fn default() -> Self { Self { trees: true } }
}

#[derive(Clone)]
pub enum Site {
    Settlement(Arc<Settlement>),
    Dungeon(Arc<Dungeon>),
}

impl Site {
    pub fn radius(&self) -> f32 {
        match self {
            Site::Settlement(settlement) => settlement.radius(),
            Site::Dungeon(dungeon) => dungeon.radius(),
        }
    }

    pub fn get_origin(&self) -> Vec2<i32> {
        match self {
            Site::Settlement(s) => s.get_origin(),
            Site::Dungeon(d) => d.get_origin(),
        }
    }

    pub fn spawn_rules(&self, wpos: Vec2<i32>) -> SpawnRules {
        match self {
            Site::Settlement(s) => s.spawn_rules(wpos),
            Site::Dungeon(d) => d.spawn_rules(wpos),
        }
    }

    pub fn apply_to<'a>(
        &'a self,
        wpos2d: Vec2<i32>,
        get_column: impl FnMut(Vec2<i32>) -> Option<&'a ColumnSample<'a>>,
        vol: &mut (impl BaseVol<Vox = Block> + RectSizedVol + ReadVol + WriteVol),
    ) {
        match self {
            Site::Settlement(settlement) => settlement.apply_to(wpos2d, get_column, vol),
            Site::Dungeon(dungeon) => dungeon.apply_to(wpos2d, get_column, vol),
        }
    }

    pub fn apply_supplement<'a>(
        &'a self,
        rng: &mut impl Rng,
        wpos2d: Vec2<i32>,
        get_column: impl FnMut(Vec2<i32>) -> Option<&'a ColumnSample<'a>>,
        supplement: &mut ChunkSupplement,
    ) {
        match self {
            Site::Settlement(settlement) => {
                settlement.apply_supplement(rng, wpos2d, get_column, supplement)
            },
            Site::Dungeon(dungeon) => dungeon.apply_supplement(rng, wpos2d, get_column, supplement),
        }
    }
}

impl From<Settlement> for Site {
    fn from(settlement: Settlement) -> Self { Site::Settlement(Arc::new(settlement)) }
}

impl From<Dungeon> for Site {
    fn from(dungeon: Dungeon) -> Self { Site::Dungeon(Arc::new(dungeon)) }
}

impl fmt::Debug for Site {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Site::Settlement(_) => write!(f, "Settlement"),
            Site::Dungeon(_) => write!(f, "Dungeon"),
        }
    }
}
