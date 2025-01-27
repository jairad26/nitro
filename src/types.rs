pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
}

#[derive(Debug)]
pub enum CacheError {
    LockError(String),
    CapacityError(String),
    // Other error types as needed
}