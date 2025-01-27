mod eviction;
mod iter;
mod linked_list;
mod node;
mod sieve;
mod types;

pub use iter::CacheIterator;
pub use sieve::SieveCache;
pub use types::{CacheError, CacheStats};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache() {
        let cache: SieveCache<String, i32> = SieveCache::new(5).unwrap();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.capacity(), 5);
    }

    #[test]
    fn test_add_and_get() {
        let mut cache: SieveCache<String, i32> = SieveCache::new(3).unwrap();

        // Add a new item
        assert_eq!(cache.add(String::from("key1"), 1).unwrap(), false);
        assert_eq!(cache.get(&String::from("key1")).unwrap(), Some(1));

        // Update existing item
        assert_eq!(cache.add(String::from("key1"), 2).unwrap(), true);
        assert_eq!(cache.get(&String::from("key1")).unwrap(), Some(2));
    }

    #[test]
    fn test_capacity_and_eviction() {
        let mut cache: SieveCache<String, i32> = SieveCache::new(2).unwrap();

        cache.add(String::from("key1"), 1).unwrap();
        cache.add(String::from("key2"), 2).unwrap();
        assert_eq!(cache.len(), 2);

        assert_eq!(cache.get(&String::from("key1")).unwrap(), Some(1));

        cache.add(String::from("key3"), 3).unwrap();

        assert_eq!(cache.get(&String::from("key1")).unwrap(), Some(1));
        assert_eq!(cache.get(&String::from("key2")).unwrap(), None);
        assert_eq!(cache.get(&String::from("key3")).unwrap(), Some(3));
    }

    #[test]
    fn test_probe() {
        let mut cache: SieveCache<String, i32> = SieveCache::new(2).unwrap();

        let (val, exists) = cache.probe(String::from("key1"), 1).unwrap();
        assert_eq!(val, 1);
        assert_eq!(exists, false);

        let (val, exists) = cache.probe(String::from("key1"), 2).unwrap();
        assert_eq!(val, 1);
        assert_eq!(exists, true);
    }

    #[test]
    fn test_delete() {
        let mut cache: SieveCache<String, i32> = SieveCache::new(2).unwrap();

        cache.add(String::from("key1"), 1).unwrap();
        assert_eq!(cache.delete(&String::from("key1")).unwrap(), true);
        assert_eq!(cache.get(&String::from("key1")).unwrap(), None);
        assert_eq!(cache.delete(&String::from("key1")).unwrap(), false);
    }

    #[test]
    fn test_purge() {
        let mut cache: SieveCache<String, i32> = SieveCache::new(2).unwrap();

        cache.add(String::from("key1"), 1).unwrap();
        cache.add(String::from("key2"), 2).unwrap();

        cache.purge();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.get(&String::from("key1")).unwrap(), None);
        assert_eq!(cache.get(&String::from("key2")).unwrap(), None);
    }

    #[test]
    fn test_eviction_policy() {
        let mut cache: SieveCache<String, i32> = SieveCache::new(3).unwrap();

        cache.add(String::from("key1"), 1).unwrap();
        cache.add(String::from("key2"), 2).unwrap();
        cache.add(String::from("key3"), 3).unwrap();

        let _ = cache.get(&String::from("key1")).unwrap();
        cache.get(&String::from("key2")).unwrap();

        cache.add(String::from("key4"), 4).unwrap();

        assert!(cache.get(&String::from("key1")).unwrap().is_some());
        assert!(cache.get(&String::from("key2")).unwrap().is_some());
        assert!(cache.get(&String::from("key3")).unwrap().is_none());
        assert!(cache.get(&String::from("key4")).unwrap().is_some());
    }

    #[test]
    fn test_with_different_types() {
        let mut cache: SieveCache<i32, String> = SieveCache::new(2).unwrap();

        cache.add(1, String::from("one")).unwrap();
        cache.add(2, String::from("two")).unwrap();

        assert_eq!(cache.get(&1).unwrap(), Some(String::from("one")));
        assert_eq!(cache.get(&2).unwrap(), Some(String::from("two")));
    }
}
