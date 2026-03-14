use crate::node::NodeType;
use crate::node_result::NodeResult;
use crate::storage::ResultStorage;
use petgraph::{
    Direction,
    graph::NodeIndex,
    prelude::StableGraph,
    visit::{DfsPostOrder, EdgeRef, Reversed},
};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    num::NonZeroUsize,
};

pub struct NodeManager {
    graph: StableGraph<NodeType, usize>,
    hash_cache: lru::LruCache<NodeIndex, u64>,
    storage: ResultStorage,
}

impl NodeManager {
    pub fn new(
        hash_cap: usize,
        protected_cap: usize,
        standard_cap: usize,
        max_bytes: usize,
    ) -> Self {
        NodeManager {
            graph: StableGraph::new(),
            hash_cache: lru::LruCache::new(NonZeroUsize::new(hash_cap).unwrap()),
            storage: ResultStorage::new(protected_cap, standard_cap, max_bytes),
        }
    }

    pub fn add_node(&mut self, node: NodeType, input_indexes: Vec<NodeIndex>) {
        let index = self.graph.add_node(node);

        input_indexes
            .iter()
            .enumerate()
            .for_each(|(port, input_index)| {
                self.graph.add_edge(*input_index, index, port);
            });
    }

    pub fn update_node(&mut self, index: NodeIndex, node: NodeType, input_indexes: Vec<NodeIndex>) {
        if let Some(target_node) = self.graph.node_weight_mut(index) {
            *target_node = node;
        } else {
            return;
        }

        let incoming_edge_ids = self
            .graph
            .edges_directed(index, Direction::Incoming)
            .map(|edge| edge.id())
            .collect::<Vec<_>>();

        incoming_edge_ids.iter().for_each(|&id| {
            self.graph.remove_edge(id);
        });

        input_indexes
            .iter()
            .enumerate()
            .for_each(|(port, input_index)| {
                self.graph.add_edge(*input_index, index, port);
            });
    }

    pub fn remove_node(&mut self, index: NodeIndex) {
        self.graph.remove_node(index);
    }

    pub fn get_result(&mut self, index: NodeIndex) -> Option<NodeResult> {
        if !self.graph.contains_node(index) {
            return None;
        }

        if petgraph::algo::is_cyclic_directed(&self.graph) {
            return None;
        }

        let mut dfs = DfsPostOrder::new(Reversed(&self.graph), index);
        let mut parent_indexes = Vec::new();
        while let Some(nx) = dfs.next(Reversed(&self.graph)) {
            parent_indexes.push(nx);
        }

        for index in parent_indexes {
            let hash = self.get_node_hash(index);

            if self.storage.contains(&hash) {
                continue;
            }

            let Some(inputs) = self.get_inputs(index) else {
                return None;
            };

            let Some(node) = self.graph.node_weight(index) else {
                return None;
            };
            let result = node.process(inputs).unwrap_or(NodeResult::None);

            self.storage.insert(hash, result, node.is_protected());
        }

        let final_hash = self.get_node_hash(index);
        self.storage.get(&final_hash)
    }

    fn get_node_hash(&mut self, index: NodeIndex) -> u64 {
        if let Some(hash) = self.hash_cache.get(&index) {
            return *hash;
        }

        let mut hasher = DefaultHasher::new();
        self.graph[index].hash(&mut hasher);

        let mut input_info = self
            .graph
            .edges_directed(index, Direction::Incoming)
            .map(|edge| (*edge.weight(), edge.source()))
            .collect::<Vec<_>>();

        input_info.sort_by_key(|(weight, _)| *weight);

        for (_, input_index) in input_info {
            let hash = self.get_node_hash(input_index);
            hash.hash(&mut hasher);
        }

        let final_hash = hasher.finish();
        self.hash_cache.push(index, final_hash);
        final_hash
    }

    fn get_inputs(&mut self, index: NodeIndex) -> Option<Vec<NodeResult>> {
        let mut input_info = self
            .graph
            .edges_directed(index, Direction::Incoming)
            .map(|edge| (*edge.weight(), edge.source()))
            .collect::<Vec<_>>();

        input_info.sort_by_key(|(weight, _)| *weight);

        input_info
            .iter()
            .map(|(_, index)| self.get_result(*index))
            .collect::<Option<Vec<NodeResult>>>()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn add_and_get() {
        let mut manager = NodeManager::new(1000, 1000, 1000, 1000);
        manager.add_node(
            NodeType::Read {
                width: 640,
                height: 480,
                filename: "tmp.bmp".to_string(),
                last_modified: 3600,
            },
            Vec::new(),
        );

        assert!(!matches!(manager.get_result(NodeIndex::new(0)), None));
    }

    #[test]
    fn update() {
        let mut manager = NodeManager::new(1000, 1000, 1000, 1000);
        let node = NodeType::Read {
            width: 640,
            height: 480,
            filename: "tmp.bmp".to_string(),
            last_modified: 3600,
        };

        manager.add_node(node.clone(), Vec::new());
        manager.update_node(NodeIndex::new(0), node.clone(), Vec::new());
        manager.update_node(NodeIndex::new(1), node.clone(), Vec::new());

        assert!(!matches!(manager.get_result(NodeIndex::new(0)), None));
        assert!(matches!(manager.get_result(NodeIndex::new(1)), None));
    }

    #[test]
    fn remove() {
        let mut manager = NodeManager::new(1000, 1000, 1000, 1000);
        let node = NodeType::Read {
            width: 640,
            height: 480,
            filename: "tmp.bmp".to_string(),
            last_modified: 3600,
        };

        manager.add_node(node.clone(), Vec::new());
        manager.remove_node(NodeIndex::new(0));

        assert!(matches!(manager.get_result(NodeIndex::new(0)), None));
    }
}
