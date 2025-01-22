use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::Debug;

// Node represents a cache entry in the doubly-linked list
#[derive(Clone)]  // Added Clone derive
struct Node<K, V> {
    key: K,
    value: V,
    visited: bool,
    // Using raw pointers instead of Box for the linked list
    next: *mut Node<K, V>,
    prev: *mut Node<K, V>,
}

impl<K, V> Node<K, V> {
    fn new(key: K, value: V) -> Self {
        Node {
            key,
            value,
            visited: false,
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
        }
    }
}

// SieveCache is the main cache structure
pub struct SieveCache<K, V> {
    cache: HashMap<K, Box<Node<K, V>>>,
    head: *mut Node<K, V>,
    tail: *mut Node<K, V>,
    hand: *mut Node<K, V>,
    size: usize,
    capacity: usize,
}

impl<K, V> SieveCache<K, V> 
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        SieveCache {
            cache: HashMap::with_capacity(capacity),
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
            hand: std::ptr::null_mut(),
            size: 0,
            capacity,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(node) = self.cache.get_mut(key) {
            node.visited = true;
            Some(&node.value)
        } else {
            None
        }
    }

    pub fn add(&mut self, key: K, value: V) -> bool {
        if let Some(node) = self.cache.get_mut(&key) {
            node.visited = true;
            node.value = value;
            true
        } else {
            self.insert(key, value);
            false
        }
    }

    pub fn probe(&mut self, key: K, value: V) -> (V, bool) {
        if let Some(node) = self.cache.get(&key) {
            (node.value.clone(), true)
        } else {
            self.insert(key, value.clone());
            (value, false)
        }
    }

    pub fn delete(&mut self, key: &K) -> bool {
        if let Some(node) = self.cache.remove(key) {
            unsafe {
                self.unlink_node(Box::into_raw(node));
            }
            self.size -= 1;
            true
        } else {
            false
        }
    }

    pub fn purge(&mut self) {
        self.cache.clear();
        self.head = std::ptr::null_mut();
        self.tail = std::ptr::null_mut();
        self.hand = std::ptr::null_mut();
        self.size = 0;
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // Private helper methods
    fn insert(&mut self, key: K, value: V) {
        if self.size == self.capacity {
            self.evict();
        }

        let new_node = Box::new(Node::new(key.clone(), value));
        let node_ptr = Box::into_raw(new_node);

        unsafe {
            // Link at the head
            (*node_ptr).next = self.head;
            if !self.head.is_null() {
                (*self.head).prev = node_ptr;
            }
            
            self.head = node_ptr;
            
            if self.tail.is_null() {
                self.tail = node_ptr;
            }
        }

        self.cache.insert(key, unsafe { Box::from_raw(node_ptr) });
        self.size += 1;
    }

    fn evict(&mut self) {
        if self.hand.is_null() {
            self.hand = self.tail;
        }

        unsafe {
            while !self.hand.is_null() {
                let current = self.hand;
                if !(*current).visited {
                    self.cache.remove(&(*current).key);
                    let prev = (*current).prev;
                    self.unlink_node(current);
                    self.hand = prev;
                    self.size -= 1;
                    return;
                }

                (*current).visited = false;
                self.hand = (*current).prev;

                if self.hand.is_null() {
                    self.hand = self.tail;
                }
            }
        }
    }

    unsafe fn unlink_node(&mut self, node: *mut Node<K, V>) {
        if !(*node).prev.is_null() {
            (*(*node).prev).next = (*node).next;
        } else {
            self.head = (*node).next;
        }

        if !(*node).next.is_null() {
            (*(*node).next).prev = (*node).prev;
        } else {
            self.tail = (*node).prev;
        }
    }
}

impl<K, V> Debug for SieveCache<K, V>
where
    K: Debug + Eq + Hash,
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SieveCache {{ size: {}, capacity: {} }}", 
            self.size,
            self.capacity
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache() {
        let cache: SieveCache<String, i32> = SieveCache::new(5);
        assert_eq!(cache.size, 0);
        assert_eq!(cache.capacity(), 5);
    }

    #[test]
    fn test_add_and_get() {
        let mut cache: SieveCache<String, i32> = SieveCache::new(3);
        
        // Add a new item
        assert_eq!(cache.add(String::from("key1"), 1), false); // Returns false for new addition
        assert_eq!(cache.get(&String::from("key1")), Some(&1));
        
        // Update existing item
        assert_eq!(cache.add(String::from("key1"), 2), true); // Returns true for update
        assert_eq!(cache.get(&String::from("key1")), Some(&2));
    }

    #[test]
    fn test_capacity_and_eviction() {
        let mut cache: SieveCache<String, i32> = SieveCache::new(2);
        
        // Fill the cache
        cache.add(String::from("key1"), 1);
        cache.add(String::from("key2"), 2);
        assert_eq!(cache.len(), 2);
        
        // Access key1 to mark it as visited
        assert_eq!(cache.get(&String::from("key1")), Some(&1));
        
        // Add another item, should evict key2 (not visited)
        cache.add(String::from("key3"), 3);
        
        assert_eq!(cache.get(&String::from("key1")), Some(&1)); // Should still exist
        assert_eq!(cache.get(&String::from("key2")), None);     // Should be evicted
        assert_eq!(cache.get(&String::from("key3")), Some(&3)); // Should exist
    }

    #[test]
    fn test_probe() {
        let mut cache: SieveCache<String, i32> = SieveCache::new(2);
        
        // Probe new item
        let (val, exists) = cache.probe(String::from("key1"), 1);
        assert_eq!(val, 1);
        assert_eq!(exists, false);
        
        // Probe existing item
        let (val, exists) = cache.probe(String::from("key1"), 2);
        assert_eq!(val, 1);  // Should get existing value
        assert_eq!(exists, true);
    }

    #[test]
    fn test_delete() {
        let mut cache: SieveCache<String, i32> = SieveCache::new(2);
        
        cache.add(String::from("key1"), 1);
        assert_eq!(cache.delete(&String::from("key1")), true);
        assert_eq!(cache.get(&String::from("key1")), None);
        assert_eq!(cache.delete(&String::from("key1")), false); // Already deleted
    }

    #[test]
    fn test_purge() {
        let mut cache: SieveCache<String, i32> = SieveCache::new(2);
        
        cache.add(String::from("key1"), 1);
        cache.add(String::from("key2"), 2);
        
        cache.purge();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.get(&String::from("key1")), None);
        assert_eq!(cache.get(&String::from("key2")), None);
    }

    #[test]
    fn test_eviction_policy() {
        let mut cache: SieveCache<String, i32> = SieveCache::new(3);
        
        // Add three items
        cache.add(String::from("key1"), 1);
        cache.add(String::from("key2"), 2);
        cache.add(String::from("key3"), 3);
        
        // Access key1 and key2 to mark them as visited
        cache.get(&String::from("key1"));
        cache.get(&String::from("key2"));
        
        // Add new item, should evict key3 (not visited)
        cache.add(String::from("key4"), 4);
        
        assert!(cache.get(&String::from("key1")).is_some());
        assert!(cache.get(&String::from("key2")).is_some());
        assert!(cache.get(&String::from("key3")).is_none());
        assert!(cache.get(&String::from("key4")).is_some());
    }

    #[test]
    fn test_with_different_types() {
        let mut cache: SieveCache<i32, String> = SieveCache::new(2);
        
        cache.add(1, String::from("one"));
        cache.add(2, String::from("two"));
        
        assert_eq!(cache.get(&1), Some(&String::from("one")));
        assert_eq!(cache.get(&2), Some(&String::from("two")));
    }
}
