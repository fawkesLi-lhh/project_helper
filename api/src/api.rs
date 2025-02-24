use crate::{
    api_model::{AppState, Node, SearchNodeQuery},
    dot_parse::parse_from_dot,
    model::HtmlNode,
};
use anyhow::{Context, Result};
use axum::{
    extract::Query,
    routing::{delete, get, post, put},
    Json, Router,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

lazy_static::lazy_static! {
    pub static ref STATE: Mutex<Option<AppState>> = Mutex::new(None);
}

pub fn routes() -> Router {
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);
    Router::new()
        .route("/init_graph", get(init_graph))
        .route("/node", get(search_node))
        .route("/node", put(put_node))
        .route("/node", delete(delete_node))
        .route("/node", post(post_node))
        .route("/graph", get(gen_graph))
        .layer(cors)
}

pub async fn gen_graph() -> Json<ResponseStatus> {
    process_resp(gen_graph_inner())
}

#[auto_context::auto_context]
fn gen_graph_inner() -> Result<String> {
    let mut state = STATE
        .lock()
        .unwrap()
        .as_ref()
        .ok_or(anyhow::anyhow!("state not found"))?
        .clone();
    let mut edges = Vec::new();
    for (from, tos) in state.edge_from_to {
        for to in tos {
            if state.node_set.contains(&to) && state.node_set.contains(&from) {
                edges.push(format!("{} --> {}", from, to));
            }
        }
    }
    let mut nodes = Vec::new();
    for id in state.node_set {
        let mut name = state
            .node_id_to_name
            .remove(&id)
            .ok_or(anyhow::anyhow!("node name not found"))?;
        if let Some(new_name) = state.node_id_to_new_name.remove(&id) {
            name = new_name;
        }
        nodes.push((id, name));
    }
    let mut dot = String::new();
    dot.push_str("flowchart TD\n");
    for (id, name) in nodes {
        dot.push_str(&format!("{}[{}]\n", id, name));
    }
    for edge in edges {
        dot.push_str(&format!("{}\n", edge));
    }

    Ok(dot)
}

#[auto_context::auto_context]
fn post_node_inner(query: Node) -> Result<()> {
    let mut pre_state = STATE.lock().unwrap();
    let state = pre_state
        .as_mut()
        .ok_or(anyhow::anyhow!("state not found"))?;
    let node_name = query.name.ok_or(anyhow::anyhow!("node name is required"))?;
    state
        .node_id_to_new_name
        .insert(query.id.clone(), node_name);
    Ok(())
}

pub async fn post_node(Json(query): Json<Node>) -> Json<ResponseStatus> {
    process_resp(post_node_inner(query))
}

#[auto_context::auto_context]
fn delete_node_inner(id: String) -> Result<()> {
    let mut pre_state = STATE.lock().unwrap();
    let state = pre_state
        .as_mut()
        .ok_or(anyhow::anyhow!("state not found"))?;
    state.node_set.remove(&id);
    Ok(())
}

pub async fn delete_node(Query(query): Query<Node>) -> Json<ResponseStatus> {
    process_resp(delete_node_inner(query.id))
}

#[auto_context::auto_context]
fn put_node_inner(query: Node) -> Result<()> {
    println!("put node {:?}", query);
    let mut pre_state = STATE.lock().unwrap();
    let state = pre_state
        .as_mut()
        .ok_or(anyhow::anyhow!("state not found"))?;
    state.node_set.insert(query.id.clone());
    Ok(())
}

pub async fn put_node(Json(query): Json<Node>) -> Json<ResponseStatus> {
    process_resp(put_node_inner(query))
}

#[auto_context::auto_context]
fn search_node_inner(query: SearchNodeQuery) -> Result<Vec<Node>> {
    println!("search node {:?}", query);
    let pre_state = STATE.lock().unwrap();
    let state = pre_state
        .as_ref()
        .ok_or(anyhow::anyhow!("state not found"))?;
    let edge_filter: Option<HashSet<String>> = query.related_node_id.map(|id| {
        state
            .edge_from_to
            .get(&id)
            .map(|v| v.clone())
            .unwrap_or_default()
    });
    let mut ans = Vec::new();
    for (id, name) in state.node_id_to_name.iter() {
        if let Some(hint_node_id) = &query.hint_node_id {
            if !name.contains(hint_node_id) {
                continue;
            }
        }
        if let Some(edge_filter) = &edge_filter {
            if !edge_filter.contains(id) {
                continue;
            }
        }
        ans.push(Node {
            id: id.clone(),
            name: Some(name.clone()),
        });
    }
    // 只保留前100个
    ans.truncate(100);

    println!("search node done {:?}", ans);

    Ok(ans)
}

pub async fn search_node(Query(query): Query<SearchNodeQuery>) -> Json<ResponseStatus> {
    process_resp(search_node_inner(query))
}

#[auto_context::auto_context]
fn init_graph_inner() -> Result<()> {
    let has_init = STATE.lock().unwrap().is_some();
    if has_init {
        println!("graph already initialized");
        return Ok(());
    }
    let cg = parse_from_dot("data/kaspa.dot")?;
    let mut node_id_to_name = HashMap::new();
    let mut father_name = Vec::new();
    gen_node_id_to_name(&cg.nodes, &mut node_id_to_name, &mut father_name);
    let mut edge_from_to = HashMap::new();
    for edge in &cg.edges {
        let entry = edge_from_to
            .entry(edge.from.clone())
            .or_insert_with(HashSet::new);
        entry.insert(edge.to.clone());
    }
    let state = AppState {
        graph: cg,
        node_id_to_name,
        edge_from_to,
        node_id_to_new_name: HashMap::new(),
        node_set: HashSet::new(),
    };
    STATE.lock().unwrap().replace(state);
    println!("init graph done");
    Ok(())
}

pub async fn init_graph() -> Json<ResponseStatus> {
    process_resp(init_graph_inner())
}

fn gen_node_id_to_name(
    nodes: &[HtmlNode],
    node_id_to_name: &mut HashMap<String, String>,
    father_name: &mut Vec<String>,
) {
    for node in nodes {
        let mut name = String::new();
        for father in father_name.iter() {
            name.push_str(&format!(":{}", father));
        }
        name.push_str(&node.text);
        node_id_to_name.insert(node.id.clone(), name);
        father_name.push(node.text.clone());
        gen_node_id_to_name(&node.children, node_id_to_name, father_name);
        father_name.pop();
    }
}

#[derive(serde::Serialize, Debug)]
pub struct ResponseStatus {
    pub code: i32,
    pub msg: String,
    pub data: serde_json::Value,
}

pub fn process_resp<T>(resp: anyhow::Result<T>) -> Json<ResponseStatus>
where
    T: serde::Serialize,
{
    let resp = resp.and_then(|v| serde_json::to_value(v).map_err(Into::into));
    let mut res = ResponseStatus::success();
    match resp {
        Ok(r) => {
            res.data = serde_json::json!(r);
        }
        Err(e) => {
            res.code = RS_CODE_FAILURE;
            res.msg = e.to_string();
        }
    }
    Json(res)
}

impl ResponseStatus {
    pub fn success() -> Self {
        Self {
            code: RS_CODE_SUCCESS,
            msg: "成功".to_owned(),
            data: serde_json::Value::Null,
        }
    }

    pub fn failure() -> Self {
        Self {
            code: RS_CODE_FAILURE,
            msg: "失败".to_owned(),
            data: serde_json::Value::Null,
        }
    }

    pub fn ok(data: serde_json::Value) -> Self {
        Self {
            code: RS_CODE_SUCCESS,
            msg: "成功".to_owned(),
            data,
        }
    }
}

pub const RS_CODE_SUCCESS: i32 = 0;
pub const RS_CODE_FAILURE: i32 = -1;
