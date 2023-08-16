use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct DebugInfo {
    pub file: String,
    pub line: i32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SchemaField {
    #[serde(rename = "type")]
    pub type_name: String,
    pub clock_companion: String,
    pub input_key: String,
    #[serde(default)]
    pub debug_info: Option<DebugInfo>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Schema {
    pub type_name: String,
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
pub struct MeasurementPolicy {
    pub measure_at_event_filter: String,
    pub measure_periodically_interval: i64,
    pub metrics_drain: MetricsDrainType,
    pub output_schema: HashMap<String, MetricSpec>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LspIr {
    pub schema: Schema,
    pub nodes: Vec<Node>,
    pub measurement_policy: MeasurementPolicy,
}
