use std::sync::{Arc, Mutex};
use crate::node::Node;

pub struct CacheIterator<'a, K, V> {
    pub(crate) current: Option<Arc<Mutex<Node<K, V>>>>,
    pub(crate) _phantom: std::marker::PhantomData<&'a (K, V)>,
}

impl<'a, K: Clone, V: Clone> Iterator for CacheIterator<'a, K, V> {
    type Item = (K, V);
    
    fn next(&mut self) -> Option<Self::Item> {
        // Take ownership of current value and replace with None
        let current = self.current.take()?;
        
        // If we can't acquire the lock, end iteration
        let guard = match current.lock() {
            Ok(guard) => guard,
            Err(_) => return None,
        };

        let result = (guard.key.clone(), guard.value.clone());
        // Store the next node before dropping the guard
        self.current = guard.next.clone();
        Some(result)
        
    }
}