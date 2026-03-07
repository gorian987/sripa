use crate::node::NodeResult;
use lru::LruCache;
use std::num::NonZeroUsize;

pub struct ResultStorage {
    protected: LruCache<u64, NodeResult>,
    standard: LruCache<u64, NodeResult>,
    max_bytes: usize,
    usage_bytes: usize,
}

impl ResultStorage {
    pub fn new(protected_cap: usize, standard_cap: usize, max_bytes: usize) -> Self {
        Self {
            protected: LruCache::new(NonZeroUsize::new(protected_cap).unwrap()),
            standard: LruCache::new(NonZeroUsize::new(standard_cap).unwrap()),
            max_bytes,
            usage_bytes: 0,
        }
    }

    pub fn get(&mut self, hash: &u64) -> Option<NodeResult> {
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

    pub fn insert(&mut self, hash: u64, result: NodeResult, is_protected: bool) {
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
        let mut storage = ResultStorage::new(1000, 1000, 1000);
        storage.insert(0, NodeResult::None, false);

        assert!(storage.contains(&0));
        assert!(!storage.contains(&1));
    }

    #[test]
    fn contains() {
        let mut storage = ResultStorage::new(1000, 1000, 1000);
        storage.insert(0, NodeResult::None, false);

        assert!(matches!(storage.get(&0).unwrap(), NodeResult::None));
        assert!(matches!(storage.get(&1), None));
    }

    #[test]
    fn over_capacity() {
        let mut storage = ResultStorage::new(1, 1, 1000);
        storage.insert(0, NodeResult::None, true);
        storage.insert(1, NodeResult::None, false);
        storage.insert(2, NodeResult::None, true);

        assert!(storage.contains(&0));
        assert!(!storage.contains(&1));
        assert!(storage.contains(&2));
    }

    #[test]
    fn over_max_bytes() {
        let mut storage = ResultStorage::new(1000, 1000, 10);
        storage.insert(0, NodeResult::Value(0.0), true);
        storage.insert(1, NodeResult::Value(0.0), false);
        storage.insert(2, NodeResult::Value(0.0), true);

        assert!(storage.contains(&0));
        assert!(!storage.contains(&1));
        assert!(storage.contains(&2));
    }
}
