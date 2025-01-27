use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// Node represents a cache entry in the doubly-linked list
#[derive(Debug)] // Added Debug derive
pub(crate) struct Node<K, V> {
    pub(crate) key: K,
    pub(crate) value: V,
    pub(crate) visited: AtomicBool,
    // Using raw pointers instead of Box for the linked list
    pub(crate) next: Option<Arc<Mutex<Node<K, V>>>>,
    pub(crate) prev: Option<Arc<Mutex<Node<K, V>>>>,
}

impl<K: Clone, V: Clone> Clone for Node<K, V> {
    fn clone(&self) -> Self {
        Node {
            key: self.key.clone(),
            value: self.value.clone(),
            visited: AtomicBool::new(self.visited.load(Ordering::SeqCst)),
            next: self.next.clone(),
            prev: self.prev.clone(),
        }
    }
}

impl<K, V> Node<K, V> {
    pub(crate) fn new(key: K, value: V) -> Self {
        Node {
            key,
            value,
            visited: AtomicBool::new(false),
            next: None,
            prev: None,
        }
    }
}
