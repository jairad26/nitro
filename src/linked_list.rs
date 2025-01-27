use std::sync::{Arc, Mutex};
use crate::node::Node;
use std::hash::Hash;

pub(crate) trait LinkedListOps<K, V> {
    fn insert_node(&mut self, key: K, value: V);
    fn unlink_node(&mut self, node: Arc<Mutex<Node<K, V>>>);
}

impl<K, V> LinkedListOps<K, V> for super::SieveCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn insert_node(&mut self, key: K, value: V) {
        let new_node = Node::new(key.clone(), value);
        let new_node = Arc::new(Mutex::new(new_node));

        // set the next pointer
        {
            let mut node = new_node.lock().unwrap();
            node.next = self.head.clone();
        }

        // update the prev pointer of the old head
        if let Some(head) = &self.head {
            head.lock().unwrap().prev = Some(new_node.clone());
        }

        // set the new head
        self.head = Some(new_node.clone());

        // if theres no tail, this is the first node
        if self.tail.is_none() {
            self.tail = Some(new_node.clone());
        }

        self.cache.insert(key, new_node);
        self.size += 1;
    }

    fn unlink_node(&mut self, node: Arc<Mutex<Node<K, V>>>) {
        let (next, prev) = {
            let node_guard = node.lock().unwrap();
            (node_guard.next.clone(), node_guard.prev.clone())
        };

        if let Some(prev_node) = &prev {
            let mut prev_guard = prev_node.lock().unwrap();
            prev_guard.next = next.clone();
        } else {
            self.head = next.clone();
        }

        if let Some(next_node) = next {
            let mut next_guard = next_node.lock().unwrap();
            next_guard.prev = prev.clone();
        } else {
            self.tail = prev;
        }
    }
}