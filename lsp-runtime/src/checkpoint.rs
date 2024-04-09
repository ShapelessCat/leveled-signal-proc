use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Deserialize, Serialize)]
pub struct Checkpoint {
    pub context_state: String, // serialize `LspContext` or `UpdateContext`
    pub input_state: String,   // serialize generated `InputSignalBag`
    pub entries: HashMap<usize, String>,
}
