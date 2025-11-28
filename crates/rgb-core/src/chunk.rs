pub trait Chunk: Clone + Send + Sync + Default {
    fn is_empty(&self) -> bool;
}
