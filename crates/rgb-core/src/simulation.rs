use crate::chunk::Chunk;
use crate::slice::WorldSlice;

pub trait Simulation: Send + Sync {
    type Chunk: Chunk;

    fn step_region(&self, slice: &mut WorldSlice<Self::Chunk>);
}
