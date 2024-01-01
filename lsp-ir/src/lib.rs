use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct DebugInfo {
    pub file: String,
    pub line: i32,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "name")]
pub enum SignalBehavior {
    Persist,
    Reset { default_expr: String },
}

impl Default for SignalBehavior {
    fn default() -> Self {
        Self::Persist
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SchemaField {
    #[serde(rename = "type")]
    pub type_name: String,
    pub clock_companion: String,
    pub input_key: String,
    #[serde(default)]
    pub signal_behavior: SignalBehavior,
    #[serde(default)]
    pub debug_info: Option<DebugInfo>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Schema {
    pub type_name: String,
    pub patch_timestamp_key: String,
    pub members: HashMap<String, SchemaField>,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum NodeInput {
    InputBag,
    InputSignal { id: String },
    Component { id: usize },
    Constant { value: String, type_name: String },
    Tuple { values: Vec<NodeInput> },
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Node {
    pub id: usize,
    pub is_measurement: bool,
    pub node_decl: String,
    pub upstreams: Vec<NodeInput>,
    pub package: String,
    pub namespace: String,
    #[serde(default)]
    pub moved: bool,
    #[serde(default)]
    pub debug_info: Option<DebugInfo>,
}

#[derive(Deserialize, Serialize, Clone)]
pub enum MetricsDrainType {
    #[serde(rename = "json")]
    Json,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MetricSpec {
    #[serde(rename = "type")]
    pub typename: String,
    pub source: NodeInput,
}

fn default_measure_trigger_signal() -> NodeInput {
    NodeInput::Constant {
        value: "0i32".to_string(),
        type_name: "i32".to_string(),
    }
}

fn default_measure_left_side_limit_signal() -> NodeInput {
    NodeInput::Constant {
        value: "false".to_string(),
        type_name: "bool".to_string(),
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ProcessingPolicy {
    pub merge_simultaneous_moments: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MeasurementPolicy {
    pub measure_at_event_filter: String,
    #[serde(default = "default_measure_trigger_signal")]
    pub measure_trigger_signal: NodeInput,
    #[serde(default = "default_measure_left_side_limit_signal")]
    pub measure_left_side_limit_signal: NodeInput,
    pub metrics_drain: MetricsDrainType,
    pub output_schema: HashMap<String, MetricSpec>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LspIr {
    pub schema: Schema,
    pub nodes: Vec<Node>,
    pub processing_policy: ProcessingPolicy,
    pub measurement_policy: MeasurementPolicy,
}

impl LspIr {
    pub fn normalize(&mut self) -> LspIr {
        // Node ID -> Node
        let mut lookup = HashMap::new();

        for node in &mut self.nodes {
            node.upstreams = node
                .upstreams
                .iter()
                .flat_map(|ni| LspIr::traceback(ni, &lookup))
                .collect();
            lookup.insert(node.id, node);
        }
        self.clone()
    }

    fn traceback(node_input: &NodeInput, lookup: &HashMap<usize, &Node>) -> Vec<NodeInput> {
        match node_input {
            NodeInput::Component { id } => {
                let from_node = lookup.get(id).unwrap();
                if from_node.is_measurement {
                    from_node.upstreams.clone()
                } else {
                    vec![node_input.clone()]
                }
            }
            NodeInput::Tuple { values } => {
                let normalized = values
                    .iter()
                    .flat_map(|ni| LspIr::traceback(ni, lookup))
                    .collect();
                vec![NodeInput::Tuple { values: normalized }]
            }
            _ => vec![node_input.clone()],
        }
    }
}
