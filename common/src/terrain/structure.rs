use super::Block;
use crate::{
    assets::{self, Asset},
    vol::{BaseVol, ReadVol, Vox, WriteVol},
    volumes::dyna::{Dyna, DynaErr},
};
use dot_vox::DotVoxData;
use std::io::{BufReader, Read};
use vek::*;

#[derive(Copy, Clone)]
pub enum StructureBlock {
    TemperateLeaves,
    PineLeaves,
    PalmLeaves,
    Block(Block),
}

impl Vox for StructureBlock {
    fn empty() -> Self {
        StructureBlock::Block(Block::empty())
    }

    fn is_empty(&self) -> bool {
        match self {
            StructureBlock::Block(block) => block.is_empty(),
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum StructureError {}

#[derive(Clone)]
pub struct Structure {
    center: Vec3<i32>,
    vol: Dyna<StructureBlock, ()>,
    empty: StructureBlock,
}

impl Structure {
    pub fn with_center(mut self, center: Vec3<i32>) -> Self {
        self.center = center;
        self
    }
}

impl BaseVol for Structure {
    type Vox = StructureBlock;
    type Err = StructureError;
}

impl ReadVol for Structure {
    #[inline(always)]
    fn get(&self, pos: Vec3<i32>) -> Result<&Self::Vox, StructureError> {
        match self.vol.get(pos + self.center) {
            Ok(block) => Ok(block),
            Err(DynaErr::OutOfBounds) => Ok(&self.empty),
        }
    }
}

impl Asset for Structure {
    fn load(buf_reader: BufReader<impl Read>) -> Result<Self, assets::Error> {
        let dot_vox_data = DotVoxData::load(buf_reader)?;

        if let Some(model) = dot_vox_data.models.get(0) {
            let palette = dot_vox_data
                .palette
                .iter()
                .map(|col| Rgba::from(col.to_ne_bytes()).into())
                .collect::<Vec<_>>();

            let mut vol = Dyna::filled(
                Vec3::new(model.size.x, model.size.y, model.size.z),
                StructureBlock::empty(),
                (),
            );

            for voxel in &model.voxels {
                let block = match voxel.i {
                    0 => StructureBlock::TemperateLeaves,
                    1 => StructureBlock::PineLeaves,
                    2 => StructureBlock::PalmLeaves,
                    index => {
                        let color = palette
                            .get(index as usize)
                            .copied()
                            .unwrap_or_else(|| Rgb::broadcast(0));
                        StructureBlock::Block(Block::new(1, color))
                    }
                };

                let _ = vol.set(
                    Vec3::new(voxel.x, voxel.y, voxel.z).map(|e| i32::from(e)),
                    block,
                );
            }

            Ok(Structure {
                center: Vec3::zero(),
                vol,
                empty: StructureBlock::empty(),
            })
        } else {
            Ok(Self {
                center: Vec3::zero(),
                vol: Dyna::filled(Vec3::zero(), StructureBlock::empty(), ()),
                empty: StructureBlock::empty(),
            })
        }
    }
}
