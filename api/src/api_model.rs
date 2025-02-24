use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::model::Graph;

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Node {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct SearchNodeQuery {
    pub related_node_id: Option<String>,
    pub hint_node_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AppState {
    pub graph: Graph,
    pub node_id_to_name: HashMap<String, String>,
    pub edge_from_to: HashMap<String, HashSet<String>>,
    pub node_id_to_new_name: HashMap<String, String>,
    pub node_set: HashSet<String>,
}
