use std::sync::atomic::Ordering;
use crate::linked_list::LinkedListOps;
use std::hash::Hash;

pub(crate) trait EvictionPolicy<K, V> {
    fn evict(&mut self);
}

impl<K, V> EvictionPolicy<K, V> for super::SieveCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn evict(&mut self) {
        if self.hand.is_none() {
            self.hand = self.tail.clone();
        }

        while let Some(current) = &self.hand {
            let curr_guard = current.lock().unwrap();

            if !curr_guard.visited.load(Ordering::SeqCst) {
                let key = curr_guard.key.clone();
                let prev = curr_guard.prev.clone();

                drop(curr_guard);

                self.cache.remove(&key);
                self.unlink_node(current.clone());
                self.hand = prev;
                self.size -= 1;
                return;
            }

            curr_guard.visited.store(false, Ordering::SeqCst);
            let prev = curr_guard.prev.clone();
            drop(curr_guard);

            self.hand = prev;

            if self.hand.is_none() {
                self.hand = self.tail.clone();
            }
        }
    }
}