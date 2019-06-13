use super::{block::Block, TerrainChunkMeta, TerrainChunkSize};
use crate::{
    vol::{BaseVol, ReadVol, WriteVol},
    volumes::chunk::{Chunk, ChunkErr},
};
use fxhash::FxHashMap;
use serde_derive::{Deserialize, Serialize};
use std::ops::Add;
use vek::*;

#[derive(Debug)]
pub enum ChonkError {
    ChunkError(ChunkErr),
    OutOfBounds,
}

const SUB_CHUNK_HEIGHT: u32 = 16;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chonk {
    z_offset: i32,
    sub_chunks: Vec<SubChunk>,
    below: Block,
    above: Block,
    meta: TerrainChunkMeta,
}

impl Chonk {
    pub fn new(z_offset: i32, below: Block, above: Block, meta: TerrainChunkMeta) -> Self {
        Self {
            z_offset,
            sub_chunks: Vec::new(),
            below,
            above,
            meta,
        }
    }

    pub fn meta(&self) -> &TerrainChunkMeta {
        &self.meta
    }

    pub fn get_min_z(&self) -> i32 {
        self.z_offset
    }

    pub fn get_max_z(&self) -> i32 {
        self.z_offset + (self.sub_chunks.len() as u32 * SUB_CHUNK_HEIGHT) as i32
    }

    pub fn get_metrics(&self) -> ChonkMetrics {
        ChonkMetrics {
            chonks: 1,
            homogeneous: self
                .sub_chunks
                .iter()
                .filter(|s| match s {
                    SubChunk::Homogeneous(_) => true,
                    _ => false,
                })
                .count(),
            hash: self
                .sub_chunks
                .iter()
                .filter(|s| match s {
                    SubChunk::Hash(_, _) => true,
                    _ => false,
                })
                .count(),
            heterogeneous: self
                .sub_chunks
                .iter()
                .filter(|s| match s {
                    SubChunk::Heterogeneous(_) => true,
                    _ => false,
                })
                .count(),
        }
    }

    fn sub_chunk_idx(&self, z: i32) -> usize {
        ((z - self.z_offset) as u32 / SUB_CHUNK_HEIGHT as u32) as usize
    }
}

impl BaseVol for Chonk {
    type Vox = Block;
    type Err = ChonkError;
}

impl ReadVol for Chonk {
    #[inline(always)]
    fn get(&self, pos: Vec3<i32>) -> Result<&Block, ChonkError> {
        if pos.z < self.z_offset {
            // Below the terrain
            Ok(&self.below)
        } else if pos.z >= self.z_offset + SUB_CHUNK_HEIGHT as i32 * self.sub_chunks.len() as i32 {
            // Above the terrain
            Ok(&self.above)
        } else {
            // Within the terrain

            let sub_chunk_idx = self.sub_chunk_idx(pos.z);

            match &self.sub_chunks[sub_chunk_idx] {
                // Can't fail
                SubChunk::Homogeneous(block) => Ok(block),
                SubChunk::Hash(cblock, map) => {
                    let rpos = pos
                        - Vec3::unit_z()
                            * (self.z_offset + sub_chunk_idx as i32 * SUB_CHUNK_HEIGHT as i32);

                    Ok(map.get(&rpos.map(|e| e as u8)).unwrap_or(cblock))
                }
                SubChunk::Heterogeneous(chunk) => {
                    let rpos = pos
                        - Vec3::unit_z()
                            * (self.z_offset + sub_chunk_idx as i32 * SUB_CHUNK_HEIGHT as i32);

                    chunk.get(rpos).map_err(|err| ChonkError::ChunkError(err))
                }
            }
        }
    }

    #[inline(always)]
    unsafe fn get_unchecked(&self, pos: Vec3<i32>) -> &Block {
        if pos.z < self.z_offset {
            // Below the terrain
            &self.below
        } else if pos.z >= self.z_offset + SUB_CHUNK_HEIGHT as i32 * self.sub_chunks.len() as i32 {
            // Above the terrain
            &self.above
        } else {
            // Within the terrain

            let sub_chunk_idx = self.sub_chunk_idx(pos.z);

            match &self.sub_chunks[sub_chunk_idx] {
                // Can't fail
                SubChunk::Homogeneous(block) => block,
                SubChunk::Hash(cblock, map) => {
                    let rpos = pos
                        - Vec3::unit_z()
                            * (self.z_offset + sub_chunk_idx as i32 * SUB_CHUNK_HEIGHT as i32);

                    map.get(&rpos.map(|e| e as u8)).unwrap_or(cblock)
                }
                SubChunk::Heterogeneous(chunk) => {
                    let rpos = pos
                        - Vec3::unit_z()
                            * (self.z_offset + sub_chunk_idx as i32 * SUB_CHUNK_HEIGHT as i32);

                    chunk.get_unchecked(rpos)
                }
            }
        }
    }
}

impl WriteVol for Chonk {
    #[inline(always)]
    fn set(&mut self, pos: Vec3<i32>, block: Block) -> Result<(), ChonkError> {
        while pos.z < self.z_offset {
            self.sub_chunks.insert(0, SubChunk::Homogeneous(self.below));
            self.z_offset -= SUB_CHUNK_HEIGHT as i32;
        }

        let sub_chunk_idx = self.sub_chunk_idx(pos.z);

        while self.sub_chunks.get(sub_chunk_idx).is_none() {
            self.sub_chunks.push(SubChunk::Homogeneous(self.above));
        }

        let rpos =
            pos - Vec3::unit_z() * (self.z_offset + sub_chunk_idx as i32 * SUB_CHUNK_HEIGHT as i32);

        match &mut self.sub_chunks[sub_chunk_idx] {
            // Can't fail
            SubChunk::Homogeneous(cblock) if block == *cblock => Ok(()),
            SubChunk::Homogeneous(cblock) => {
                let mut map = FxHashMap::default();
                map.insert(rpos.map(|e| e as u8), block);

                self.sub_chunks[sub_chunk_idx] = SubChunk::Hash(*cblock, map);
                Ok(())
            }
            SubChunk::Hash(cblock, _map) if block == *cblock => Ok(()),
            SubChunk::Hash(_cblock, map) if map.len() < 4096 => {
                map.insert(rpos.map(|e| e as u8), block);
                Ok(())
            }
            SubChunk::Hash(cblock, map) => {
                let mut new_chunk = Chunk::filled(*cblock, ());
                new_chunk.set(rpos, block).unwrap(); // Can't fail (I hope)

                for (map_pos, map_block) in map {
                    new_chunk
                        .set(map_pos.map(|e| e as i32), *map_block)
                        .unwrap(); // Can't fail (I hope!)
                }

                self.sub_chunks[sub_chunk_idx] = SubChunk::Heterogeneous(new_chunk);
                Ok(())
            }

            /*
            SubChunk::Homogeneous(cblock) => {
                let mut new_chunk = Chunk::filled(*cblock, ());

                new_chunk.set(rpos, block).unwrap(); // Can't fail (I hope!)

                self.sub_chunks[sub_chunk_idx] = SubChunk::Heterogeneous(new_chunk);
                Ok(())
            }
            */
            SubChunk::Heterogeneous(chunk) => chunk
                .set(rpos, block)
                .map_err(|err| ChonkError::ChunkError(err)),
            //_ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubChunk {
    Homogeneous(Block),
    Hash(Block, FxHashMap<Vec3<u8>, Block>),
    Heterogeneous(Chunk<Block, TerrainChunkSize, ()>),
}

impl SubChunk {
    pub fn filled(block: Block) -> Self {
        SubChunk::Homogeneous(block)
    }
}

#[derive(Debug)]
pub struct ChonkMetrics {
    chonks: usize,
    homogeneous: usize,
    hash: usize,
    heterogeneous: usize,
}

impl Default for ChonkMetrics {
    fn default() -> Self {
        ChonkMetrics {
            chonks: 0,
            homogeneous: 0,
            hash: 0,
            heterogeneous: 0,
        }
    }
}

impl Add for ChonkMetrics {
    type Output = Self;

    fn add(self, other: Self::Output) -> Self {
        Self::Output {
            chonks: self.chonks + other.chonks,
            homogeneous: self.homogeneous + other.homogeneous,
            hash: self.hash + other.hash,
            heterogeneous: self.heterogeneous + other.heterogeneous,
        }
    }
}
