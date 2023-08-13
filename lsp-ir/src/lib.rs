use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct DebugInfo {
    pub file: String,
    pub line: i32,
}

#[derive(Deserialize, Clone)]
pub struct SchemaField {
    #[serde(rename = "type")]
    pub type_name: String,
    pub clock_companion: String,
    pub input_key: String,
    #[serde(default)]
    pub debug_info: Option<DebugInfo>,
}

#[derive(Deserialize, Clone)]
pub struct Schema {
    pub type_name: String,
    pub members: HashMap<String, SchemaField>,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum NodeInput {
    InputBag,
    InputSignal { id: String },
    Component { id: usize },
    Constant { value: String, type_name: String },
    Tuple { values: Vec<NodeInput> },
}

#[derive(Deserialize, Clone)]
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

#[derive(Deserialize, Clone)]
pub enum MetricsDrainType {
    #[serde(rename = "json")]
    Json,
}

#[derive(Deserialize, Clone)]
pub struct MeasurementPolicy {
    pub measure_at_event_filter: String,
    pub measure_periodically_interval: i64,
    pub metrics_drain: MetricsDrainType,
    pub output_schema: HashMap<String, NodeInput>,
}

#[derive(Deserialize, Clone)]
pub struct LspIr {
    pub schema: Schema,
    pub nodes: Vec<Node>,
    pub measurement_policy: MeasurementPolicy,
}
