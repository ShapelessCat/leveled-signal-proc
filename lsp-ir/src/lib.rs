use std::collections::HashMap;

use serde::Deserialize;


#[derive(Deserialize, Debug)]
pub struct SchemaField {
    #[serde(rename = "type")]
    pub type_name: String,
    pub clock_companion: String,
    pub input_key: String,
}

#[derive(Deserialize)]
#[serde(transparent)]
pub struct Schema {
    pub schema_items: HashMap<String, SchemaField>
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum NodeInput {
    InputBag,
    InputSignal { id: String },
    Component { id: usize },
    Constant { value: String, type_name: String },
    Tuple { values: Vec<NodeInput> },
}

#[derive(Deserialize)]
pub struct Node {
    pub id: usize,
    pub is_measurement: bool,
    pub node_decl: String,
    pub upstreams: Vec<NodeInput>,
    pub package: String,
    pub namespace: String,
}

#[derive(Deserialize)]
pub enum MetricsDrainType {
    #[serde(rename = "json")]
    Json,
}

#[derive(Deserialize)]
pub struct MeasurementPolicy {
    pub measure_at_event_filter: String,
    pub measure_periodically_interval: i64,
    pub metrics_drait: MetricsDrainType,
    pub output_items: HashMap<String, NodeInput>,
}

#[derive(Deserialize)]
pub struct LspIr {
    pub schema: Schema,
    pub nodes: Vec<Node>,
    pub measurement_policy: MeasurementPolicy,
}