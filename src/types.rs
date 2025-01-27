use std::fmt;
use std::error::Error;

#[derive(Debug, Default)]
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

// Implement Display for CacheError
impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::LockError(msg) => write!(f, "Lock error: {}", msg),
            CacheError::CapacityError(msg) => write!(f, "Capacity error: {}", msg),
        }
    }
}

// Implement Error trait for CacheError
impl Error for CacheError {}
