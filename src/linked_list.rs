use crate::{node::Node, CacheError};
use std::hash::Hash;
use std::sync::{Arc, Mutex};

pub(crate) trait LinkedListOps<K, V> {
    fn insert_node(&mut self, key: K, value: V) -> Result<(), CacheError>;
    fn unlink_node(&mut self, node: Arc<Mutex<Node<K, V>>>) -> Result<(), CacheError>;
}

impl<K, V> LinkedListOps<K, V> for super::SieveCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn insert_node(&mut self, key: K, value: V) -> Result<(), CacheError> {
        let new_node = Node::new(key.clone(), value);
        let new_node = Arc::new(Mutex::new(new_node));

        // set the next pointer
        {
            let mut node = new_node
                .lock()
                .map_err(|e| CacheError::LockError(e.to_string()))?;
            node.next = self.head.clone();
        }

        // update the prev pointer of the old head
        if let Some(head) = &self.head {
            let mut head_guard = head
                .lock()
                .map_err(|e| CacheError::LockError(e.to_string()))?;

            head_guard.prev = Some(new_node.clone());
        }

        // set the new head
        self.head = Some(new_node.clone());

        // if theres no tail, this is the first node
        if self.tail.is_none() {
            self.tail = Some(new_node.clone());
        }

        self.cache.insert(key, new_node);
        self.size += 1;
        Ok(())
    }

    fn unlink_node(&mut self, node: Arc<Mutex<Node<K, V>>>) -> Result<(), CacheError> {
        let (next, prev) = {
            let node_guard = node
                .lock()
                .map_err(|e| CacheError::LockError(e.to_string()))?;
            (node_guard.next.clone(), node_guard.prev.clone())
        };

        if let Some(prev_node) = &prev {
            let mut prev_guard = prev_node
                .lock()
                .map_err(|e| CacheError::LockError(e.to_string()))?;
            prev_guard.next = next.clone();
        } else {
            self.head = next.clone();
        }

        if let Some(next_node) = next {
            let mut next_guard = next_node
                .lock()
                .map_err(|e| CacheError::LockError(e.to_string()))?;
            next_guard.prev = prev.clone();
        } else {
            self.tail = prev;
        }

        Ok(())
    }
}
