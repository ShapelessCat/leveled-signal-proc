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
pub struct EnumVariantInfo {
    pub variant_name: String,
    pub input_value: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SchemaField {
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub enum_variants: Vec<EnumVariantInfo>,
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

#[derive(Deserialize, Serialize, Clone)]
pub struct DerivedMetricSpec {
    #[serde(rename = "type")]
    pub typename: String,
    pub source: NodeInput,
    pub source_metric_name: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ResetSwitch {
    pub metric_name: String,
    pub source: NodeInput,
    pub initial_value: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ComplementaryOutputConfig {
    pub schema: HashMap<String, DerivedMetricSpec>,
    pub reset_switch: Option<ResetSwitch>,
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
    #[serde(default)]
    pub output_control_measurement_ids: Vec<usize>,
    pub output_schema: HashMap<String, MetricSpec>,
    pub complementary_output_config: Option<ComplementaryOutputConfig>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LspIr {
    pub schema: Schema,
    pub nodes: Vec<Node>,
    pub processing_policy: ProcessingPolicy,
    pub measurement_policy: MeasurementPolicy,
}

impl LspIr {
    pub fn normalize(&mut self) {
        let mut buffer = vec![];
        for i in 0..self.nodes.len() {
            let node = &self.nodes[i];
            node.upstreams
                .iter()
                .for_each(|ni| Self::traceback(ni, &self.nodes, &mut buffer));
            self.nodes[i].upstreams = buffer.clone();
            buffer.clear();
        }
    }

    fn traceback(node_input: &NodeInput, lookup: &Vec<Node>, buffer: &mut Vec<NodeInput>) {
        match node_input {
            NodeInput::Component { id } => {
                let from_node = &lookup[*id];
                let updated_node_input = if from_node.is_measurement {
                    match &from_node.upstreams[..] {
                        [ni] => ni.clone(),
                        _ => NodeInput::Tuple {
                            values: from_node.upstreams.clone(),
                        },
                    }
                } else {
                    node_input.clone()
                };
                buffer.push(updated_node_input)
            }
            NodeInput::Tuple { values } => {
                let len = buffer.len();
                values
                    .iter()
                    .for_each(|ni| Self::traceback(ni, lookup, buffer));
                let updated = buffer.split_off(len);
                buffer.push(NodeInput::Tuple { values: updated })
            }
            _ => buffer.push(node_input.clone()),
        }
    }
}
