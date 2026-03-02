use std::{
    collections::{HashMap, VecDeque},
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

use image::{self, DynamicImage};

use petgraph::{
    Direction,
    graph::NodeIndex,
    prelude::StableGraph,
    visit::{DfsPostOrder, EdgeRef, Reversed},
};

use crate::record::TaggedImage;

#[derive(Clone, Debug, Hash, serde::Deserialize, serde::Serialize, specta::Type)]
#[serde(tag = "type", content = "parameter")]
pub enum NodeType {
    // Read
    Read,

    // Filter
    Grayscale { color: ColorType },
    GaussianBlur { kernel: u32, sigma: u32 },
    Sobel { kernel: u32 },
    Binarization { threshold: u32 },

    // Calculation
    Center { rect: ImageRect },
    Max { rect: ImageRect },
    Min { rect: ImageRect },
    Average { rect: ImageRect },
    Mode { rect: ImageRect },
}

impl NodeType {
    fn process(
        &self,
        image_queue: &mut VecDeque<TaggedImage>,
        inputs: Vec<Arc<NodeResult>>,
    ) -> NodeResult {
        match self {
            Self::Read => {
                if let Some(tagged_image) = image_queue.pop_front() {
                    return NodeResult::Image(tagged_image.image);
                }
                NodeResult::None
            }
            Self::Grayscale { color } => NodeResult::None,
            Self::GaussianBlur { kernel, sigma } => NodeResult::None,
            Self::Sobel { kernel } => NodeResult::None,
            Self::Binarization { threshold } => NodeResult::None,
            Self::Center { rect } => NodeResult::None,
            Self::Max { rect } => NodeResult::None,
            Self::Min { rect } => NodeResult::None,
            Self::Average { rect } => NodeResult::None,
            Self::Mode { rect } => NodeResult::None,
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, serde::Deserialize, serde::Serialize, specta::Type)]
pub enum ColorType {
    Gray,
    Red,
    Green,
    Blue,
}

#[derive(Clone, Debug, Hash, serde::Deserialize, serde::Serialize, specta::Type)]
pub struct ImageRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Clone, Debug)]
pub enum NodeResult {
    Image(Arc<DynamicImage>),
    Point(f32, f32),
    Value(f32),
    None,
}

// enum InputValue {
//     Int(i32),
//     Float(Hashablef32),
//     Connection(Vec<u32>),
// }

// #[derive(Debug, Clone, Copy)]
// struct Hashablef32(pub f32);

// impl Hash for Hashablef32 {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         let bits = if self.0.is_nan() {
//             f32::NAN.to_bits()
//         } else if self.0 == 0.0 {
//             0.0f32.to_bits()
//         } else {
//             self.0.to_bits()
//         };
//         bits.hash(state);
//     }
// }

pub struct NodeManager {
    graph: StableGraph<NodeType, InputPort>,
    storage: HashMap<u64, Arc<NodeResult>>,
}

impl NodeManager {
    pub fn add_node(&mut self, new_node: NodeType, input_indexes: Vec<NodeIndex>) {
        let new_index = self.graph.add_node(new_node);

        input_indexes.iter().enumerate().for_each(|(port, &index)| {
            self.graph.add_edge(index, new_index, InputPort(port));
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
            self.graph.add_edge(index, target_index, InputPort(port));
        });
    }

    pub fn remove_node(&mut self, target_index: NodeIndex) {
        self.graph.remove_node(target_index);
    }

    pub fn process(
        &mut self,
        image_queue: &mut VecDeque<TaggedImage>,
        target_index: NodeIndex,
    ) -> Arc<NodeResult> {
        if petgraph::algo::is_cyclic_directed(&self.graph) {
            return Arc::new(NodeResult::None);
        }

        self.storage.clear();

        let mut dfs = DfsPostOrder::new(Reversed(&self.graph), target_index);
        let mut parent_indexes = Vec::new();
        while let Some(nx) = dfs.next(Reversed(&self.graph)) {
            parent_indexes.push(nx);
        }

        let mut hash_cache = HashMap::new();

        for index in parent_indexes {
            let hash = self.get_node_hash(&mut hash_cache, index);

            if self.storage.contains_key(&hash) {
                continue;
            }

            let Some(node) = self.graph.node_weight(index) else {
                return Arc::new(NodeResult::None);
            };

            let inputs = self.get_inputs(&mut hash_cache, index);
            let result = Arc::new(node.process(image_queue, inputs));
            self.storage.insert(hash, result);
        }

        let final_hash = self.get_node_hash(&mut hash_cache, target_index);
        self.storage
            .get(&final_hash)
            .cloned()
            .unwrap_or_else(|| Arc::new(NodeResult::None))
    }

    pub fn collect_garbage(&mut self) {}

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

        edges.sort_by_key(|e| e.weight().0);

        for edge in edges {
            let hash = self.get_node_hash(cache, edge.source());
            hash.hash(&mut hasher);
        }

        let final_hash = hasher.finish();
        cache.insert(index, final_hash);
        final_hash
    }

    fn get_inputs(
        &self,
        cache: &mut HashMap<NodeIndex, u64>,
        index: NodeIndex,
    ) -> Vec<Arc<NodeResult>> {
        let mut edges = self
            .graph
            .edges_directed(index, Direction::Incoming)
            .collect::<Vec<_>>();

        edges.sort_by_key(|edge| edge.weight().0);

        edges
            .iter()
            .map(|edge| {
                let input_hash = self.get_node_hash(cache, edge.source());
                self.storage
                    .get(&input_hash)
                    .cloned()
                    .unwrap_or(Arc::new(NodeResult::None))
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Clone, Copy, Debug)]
struct InputPort(pub usize);
