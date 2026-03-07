use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use petgraph::{
    Direction,
    graph::NodeIndex,
    prelude::StableGraph,
    visit::{DfsPostOrder, EdgeRef, Reversed},
};

use crate::node::{NodeResult, NodeType};
use crate::storage::ResultStorage;

pub struct NodeManager {
    graph: StableGraph<NodeType, usize>,
    storage: ResultStorage,
}

impl NodeManager {
    pub fn new(protected_cap: usize, standard_cap: usize, max_bytes: usize) -> Self {
        NodeManager {
            graph: StableGraph::new(),
            storage: ResultStorage::new(protected_cap, standard_cap, max_bytes),
        }
    }

    pub fn add_node(&mut self, new_node: NodeType, input_indexes: Vec<NodeIndex>) {
        let new_index = self.graph.add_node(new_node);

        input_indexes.iter().enumerate().for_each(|(port, &index)| {
            self.graph.add_edge(index, new_index, port);
        });
    }

    pub fn update_node(
        &mut self,
        target_index: NodeIndex,
        new_node: NodeType,
        input_indexes: Vec<NodeIndex>,
    ) {
        if let Some(node) = self.graph.node_weight_mut(target_index) {
            *node = new_node;
        } else {
            return;
        }

        let incoming_edge_ids = self
            .graph
            .edges_directed(target_index, Direction::Incoming)
            .map(|edge| edge.id())
            .collect::<Vec<_>>();

        incoming_edge_ids.iter().for_each(|&id| {
            self.graph.remove_edge(id);
        });

        input_indexes.iter().enumerate().for_each(|(port, &index)| {
            self.graph.add_edge(index, target_index, port);
        });
    }

    pub fn remove_node(&mut self, target_index: NodeIndex) {
        self.graph.remove_node(target_index);
    }

    pub fn get_result(&mut self, target_index: NodeIndex) -> Option<NodeResult> {
        if !self.graph.contains_node(target_index) {
            return None;
        }

        if petgraph::algo::is_cyclic_directed(&self.graph) {
            return None;
        }

        let mut dfs = DfsPostOrder::new(Reversed(&self.graph), target_index);
        let mut parent_indexes = Vec::new();
        while let Some(nx) = dfs.next(Reversed(&self.graph)) {
            parent_indexes.push(nx);
        }

        let mut hash_cache = HashMap::new();

        for index in parent_indexes {
            let hash = self.get_node_hash(&mut hash_cache, index);

            if self.storage.contains(&hash) {
                continue;
            }

            let inputs = self.get_inputs(&mut hash_cache, index);

            let Some(node) = self.graph.node_weight(index) else {
                return None;
            };
            let result = node.process(inputs);

            self.storage.insert(hash, result, node.is_protected());
        }

        let final_hash = self.get_node_hash(&mut hash_cache, target_index);
        self.storage.get(&final_hash)
    }

    fn get_node_hash(&self, cache: &mut HashMap<NodeIndex, u64>, index: NodeIndex) -> u64 {
        if let Some(&hash) = cache.get(&index) {
            return hash;
        }

        let mut hasher = DefaultHasher::new();
        self.graph[index].hash(&mut hasher);

        let mut edges = self
            .graph
            .edges_directed(index, Direction::Incoming)
            .collect::<Vec<_>>();

        edges.sort_by_key(|e| e.weight());

        for edge in edges {
            let hash = self.get_node_hash(cache, edge.source());
            hash.hash(&mut hasher);
        }

        let final_hash = hasher.finish();
        cache.insert(index, final_hash);
        final_hash
    }

    fn get_inputs(
        &mut self,
        cache: &mut HashMap<NodeIndex, u64>,
        index: NodeIndex,
    ) -> Vec<NodeResult> {
        let mut input_info = self
            .graph
            .edges_directed(index, Direction::Incoming)
            .map(|edge| (*edge.weight(), edge.source()))
            .collect::<Vec<_>>();

        input_info.sort_by_key(|(weight, _)| *weight);

        input_info
            .iter()
            .map(|(_, index)| {
                let input_hash = self.get_node_hash(cache, *index);
                self.storage.get(&input_hash).unwrap_or(NodeResult::None)
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn add_and_get() {
        let mut manager = NodeManager::new(1000, 1000, 1000);
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
        let mut manager = NodeManager::new(1000, 1000, 1000);
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
        let mut manager = NodeManager::new(1000, 1000, 1000);
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
