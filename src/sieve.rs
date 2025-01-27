use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::eviction::EvictionPolicy;
use crate::iter::CacheIterator;
use crate::linked_list::LinkedListOps;
use crate::node::Node;
use crate::types::{CacheError, CacheStats};

pub struct SieveCache<K, V> {
    pub(crate) cache: HashMap<K, Arc<Mutex<Node<K, V>>>>,
    pub(crate) head: Option<Arc<Mutex<Node<K, V>>>>,
    pub(crate) tail: Option<Arc<Mutex<Node<K, V>>>>,
    pub(crate) hand: Option<Arc<Mutex<Node<K, V>>>>,
    pub(crate) size: usize,
    pub(crate) capacity: usize,
    pub(crate) stats: CacheStats,
}

impl<K, V> SieveCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    // The SieveCache struct is a HashMap that stores the keys and values of the cache.
    // It also has a head, tail, and hand field that are used to implement the Sieve algorithm.
    // The size field keeps track of the number of elements in the cache, and the capacity field
    // specifies the maximum number of elements that the cache can hold.
    pub fn new(capacity: usize) -> Result<Self, CacheError> {
        if capacity < 1 {
            return Err(CacheError::CapacityError(
                "Cache capacity cannot be zero".to_string(),
            ));
        }
        Ok(SieveCache {
            cache: HashMap::with_capacity(capacity),
            head: None,
            tail: None,
            hand: None,
            size: 0,
            capacity,
            stats: CacheStats { hits: 0, misses: 0 },
        })
    }

    /// Retrieves a value from the cache if it exists.
    ///
    /// # Returns
    /// - `Ok(Some(V))` if the key exists
    /// - `Ok(None)` if the key doesn't exist
    /// - `Err(CacheError)` if there was a lock poisoning
    pub fn get(&mut self, key: &K) -> Result<Option<V>, CacheError> {
        if let Some(node) = self.cache.get_mut(key) {
            let guard = node
                .lock()
                .map_err(|e| CacheError::LockError(e.to_string()))?;
            guard.visited.store(true, Ordering::SeqCst);
            self.stats.hits += 1;
            Ok(Some(guard.value.clone()))
        } else {
            self.stats.misses += 1;
            Ok(None)
        }
    }

    /// Adds a value to the cache.
    ///
    /// # Returns
    /// - `Ok(true)` if the key already existed and the value was updated
    /// - `Ok(false)` if the key was newly inserted
    /// - `Err(CacheError)` if there was a lock poisoning
    #[must_use = "The returned value indicates whether the key already existed"]
    pub fn add(&mut self, key: K, value: V) -> Result<bool, CacheError> {
        if let Some(node) = self.cache.get_mut(&key) {
            let mut node_guard = node
                .lock()
                .map_err(|e| CacheError::LockError(e.to_string()))?;
            node_guard.visited.store(true, Ordering::SeqCst);
            node_guard.value = value;
            drop(node_guard);
            Ok(true)
        } else {
            self.insert(key, value)?;
            Ok(false)
        }
    }

    /// Probes the cache for a value, inserting it if not present.
    ///
    /// # Returns
    /// - The value associated with the key (either existing or newly inserted)
    /// - A boolean indicating whether the key already existed
    #[must_use = "This returns the probed value and whether it existed"]
    pub fn probe(&mut self, key: K, value: V) -> Result<(V, bool), CacheError> {
        match self.cache.get(&key) {
            Some(node) => {
                let guard = node
                    .lock()
                    .map_err(|e| CacheError::LockError(e.to_string()))?;
                let result = guard.value.clone();
                drop(guard);
                Ok((result, true))
            }
            None => {
                self.insert(key, value.clone())?;
                Ok((value, false))
            }
        }
    }

    pub fn delete(&mut self, key: &K) -> Result<bool, CacheError> {
        if let Some(node) = self.cache.remove(key) {
            self.unlink_node(node)?;
            self.size -= 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn purge(&mut self) {
        self.cache.clear();
        self.head = None;
        self.tail = None;
        self.hand = None;
        self.size = 0;
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    fn insert(&mut self, key: K, value: V) -> Result<(), CacheError> {
        if self.size == self.capacity {
            self.evict()?;
        }
        self.insert_node(key, value)?;
        Ok(())
    }

    pub fn get_stats(&self) -> &CacheStats {
        &self.stats
    }

    pub fn iter(&self) -> CacheIterator<K, V> {
        CacheIterator {
            current: self.head.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<K, V> Debug for SieveCache<K, V>
where
    K: Debug + Eq + Hash,
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SieveCache")
            .field("size", &self.size)
            .field("capacity", &self.capacity)
            .field(
                "cache_usage",
                &format!("{}%", (self.size * 100) / self.capacity),
            )
            .field("hits", &self.stats.hits)
            .field("misses", &self.stats.misses)
            .field(
                "hit_rate",
                &format!(
                    "{}%",
                    if self.stats.hits + self.stats.misses > 0 {
                        (self.stats.hits * 100) / (self.stats.hits + self.stats.misses)
                    } else {
                        0
                    }
                ),
            )
            .finish()
    }
}
