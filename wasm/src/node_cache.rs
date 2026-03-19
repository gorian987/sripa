use crate::node_value::NodeValue;
use std::num::NonZeroUsize;

pub struct NodeCache {
    protected: lru::LruCache<u64, NodeValue>,
    standard: lru::LruCache<u64, NodeValue>,
    max_bytes: usize,
    usage_bytes: usize,
}

impl NodeCache {
    pub fn new(protected_cap: usize, standard_cap: usize, max_bytes: usize) -> Self {
        Self {
            protected: lru::LruCache::new(NonZeroUsize::new(protected_cap).unwrap()),
            standard: lru::LruCache::new(NonZeroUsize::new(standard_cap).unwrap()),
            max_bytes,
            usage_bytes: 0,
        }
    }

    pub fn get(&mut self, hash: &u64) -> Option<NodeValue> {
        if let Some(result) = self.protected.get(hash) {
            return Some(result.clone());
        }

        if let Some(result) = self.standard.get(hash) {
            return Some(result.clone());
        }

        None
    }

    pub fn contains(&self, hash: &u64) -> bool {
        self.protected.contains(hash) || self.standard.contains(hash)
    }

    pub fn insert(&mut self, hash: u64, result: NodeValue, is_protected: bool) {
        if let Some(old) = self.protected.pop(&hash) {
            self.usage_bytes -= old.size_bytes();
        } else if let Some(old) = self.standard.pop(&hash) {
            self.usage_bytes -= old.size_bytes();
        }

        self.usage_bytes += result.size_bytes();

        if self.max_bytes < self.usage_bytes {
            self.collect_garbage();
        }

        if is_protected {
            if let Some((old_hash, demoted)) = self.protected.push(hash, result) {
                if let Some((_, removed)) = self.standard.push(old_hash, demoted) {
                    self.usage_bytes -= removed.size_bytes();
                }
            }
        } else {
            if let Some((_, removed)) = self.standard.push(hash, result) {
                self.usage_bytes -= removed.size_bytes();
            }
        }
    }

    fn collect_garbage(&mut self) {
        let target_usage = (self.max_bytes as f64 * 0.8) as usize;

        while target_usage < self.usage_bytes {
            if let Some((_, oldest)) = self.standard.pop_lru() {
                self.usage_bytes -= oldest.size_bytes();
            } else if let Some((_, oldest)) = self.protected.pop_lru() {
                self.usage_bytes -= oldest.size_bytes();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut storage = NodeCache::new(1000, 1000, 1000);
        storage.insert(0, NodeValue::None, false);

        assert!(storage.contains(&0));
        assert!(!storage.contains(&1));
    }

    #[test]
    fn contains() {
        let mut storage = NodeCache::new(1000, 1000, 1000);
        storage.insert(0, NodeValue::None, false);

        assert!(matches!(storage.get(&0).unwrap(), NodeValue::None));
        assert!(matches!(storage.get(&1), None));
    }

    #[test]
    fn over_capacity() {
        let mut storage = NodeCache::new(1, 1, 1000);
        storage.insert(0, NodeValue::None, true);
        storage.insert(1, NodeValue::None, false);
        storage.insert(2, NodeValue::None, true);

        assert!(storage.contains(&0));
        assert!(!storage.contains(&1));
        assert!(storage.contains(&2));
    }

    #[test]
    fn over_max_bytes() {
        let mut storage = NodeCache::new(1000, 1000, 10);
        storage.insert(0, NodeValue::Value(0.0), true);
        storage.insert(1, NodeValue::Value(0.0), false);
        storage.insert(2, NodeValue::Value(0.0), true);

        assert!(storage.contains(&0));
        assert!(!storage.contains(&1));
        assert!(storage.contains(&2));
    }
}
