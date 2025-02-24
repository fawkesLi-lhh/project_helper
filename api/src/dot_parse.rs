use anyhow::{Context, Result};
use graphviz_rust::dot_structures::*;
use graphviz_rust::parse;
use std::collections::HashSet;
use std::fs::read_to_string;
use std::io::Write;

#[auto_context::auto_context]
pub fn write_to_file(data: &str, path: &str) -> Result<()> {
    let mut file = std::fs::File::create(path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

#[auto_context::auto_context]
fn check_node(node: Node) -> Result<crate::model::HtmlNode> {
    //println!("check_node {:?}", node.id);
    match node.id.0 {
        Id::Escaped(_id) => {}
        _ => {
            return Err(anyhow::anyhow!("check node id"));
        }
    }
    if node.attributes.len() < 2 {
        return Err(anyhow::anyhow!("check node attributes"));
    }
    let Attribute(id1, id2) = node.attributes[1].clone();
    match id1 {
        Id::Plain(_id) => {}
        _ => {
            return Err(anyhow::anyhow!("check external attribute id"));
        }
    }
    match id2 {
        Id::Html(html) => {
            return check_html(html);
        }
        _ => {
            return Err(anyhow::anyhow!("check internal attribute id"));
        }
    }
}

#[auto_context::auto_context]
fn check_vertex(vertex: Vertex) -> Result<String> {
    let (id, port) = match vertex {
        Vertex::N(NodeId(id, port)) => (id, port),
        _ => {
            return Err(anyhow::anyhow!("check vertex type"));
        }
    };
    let id = match id {
        Id::Plain(id) => id,
        _ => {
            return Err(anyhow::anyhow!("check vertex id"));
        }
    };
    let port_id = port
        .ok_or(anyhow::anyhow!("check vertex port1"))?
        .0
        .ok_or(anyhow::anyhow!("check vertex port2"))?;
    let port_id = match port_id {
        Id::Escaped(id) => id,
        _ => {
            return Err(anyhow::anyhow!("check vertex port id"));
        }
    };
    let port_id = port_id.replace("\"", "");
    Ok(format!("{}:{}", id, port_id))
}

#[auto_context::auto_context]
fn check_edge(edge: Edge) -> Result<crate::model::Edge> {
    let edge = match edge.ty {
        EdgeTy::Pair(from, to) => {
            let from_id = check_vertex(from)?;
            let to_id = check_vertex(to)?;
            crate::model::Edge {
                from: from_id,
                to: to_id,
            }
        }
        _ => {
            return Err(anyhow::anyhow!("check edge type"));
        }
    };
    Ok(edge)
}

#[auto_context::auto_context]
fn get_filter_text_str(id: &str) -> String {
    let mut id = id.replace("\\n", "");
    id = id.replace("\\t", "");
    id = id.replace("Text", "");
    id = id.replace("\"", "");
    id = id.replace("(", "");
    id = id.replace(")", "");
    id = id.replace(",", "");
    id = id.replace(";", "");
    id = id.replace("Element", "");
    id
}

#[auto_context::auto_context]
fn get_filter_str(id: &str) -> String {
    let mut id = id.replace("\\n", "");
    id = id.replace("\\t", "");
    id = id.replace("Text", "");
    id = id.replace(")", "");
    id = id.replace(" ", "");
    id = id.replace("\"", "");
    id = id.replace("<", "");
    id = id.replace(">", "");
    id = id.replace("(", "");
    id = id.replace(")", "");
    id = id.replace(",", "");
    id = id.replace(";", "");
    id = id.replace("Element", "");
    id = id.replace("html", "");
    id = id.replace("head", "");
    id = id.replace("tbody", "");
    id = id.replace("body", "");
    id = id.replace("table", "");
    id = id.replace("tbody", "");
    id = id.replace("tr", "");
    id = id.replace("td", "");
    id = id.replace("width", "");
    id
}

#[auto_context::auto_context]
fn check_useless_id(id: &str) -> bool {
    if id.contains("Text") {
        let id = get_filter_str(id);
        id.is_empty()
    } else {
        let id = get_filter_str(id);
        !(id.contains("port") || id.contains("id"))
    }
}

#[auto_context::auto_context]
fn get_id_from_raw(raw: &str) -> Option<String> {
    let mut ans = None;
    for i in raw.split(" ") {
        if i.contains("id=") {
            let tt = i.split("=").collect::<Vec<&str>>();
            if tt.len() > 1 {
                ans = Some(get_filter_text_str(tt[1]));
            }
        }
    }

    ans
}

#[auto_context::auto_context]
fn check_html_node<'a>(
    node: ego_tree::NodeRef<'a, scraper::Node>,
    html_node: &mut crate::model::HtmlNodeRaw,
    fa_children: &mut Vec<crate::model::HtmlNodeRaw>,
) -> Result<()> {
    //println!("check_html_node {:?}", node.value());
    html_node.raw = format!("{:?}", node.value());
    html_node.id = get_id_from_raw(&html_node.raw).unwrap_or_default();
    for child in node.children() {
        let mut child_html_node = crate::model::HtmlNodeRaw::default();
        check_html_node(child, &mut child_html_node, &mut html_node.children)?;
        if !check_useless_id(&child_html_node.raw) {
            html_node.children.push(child_html_node);
        }
    }
    let flag = check_useless_id(&html_node.raw);
    if flag {
        fa_children.extend(html_node.children.clone());
    }

    let mut new_children = Vec::new();
    for child in html_node.children.clone() {
        if child.raw.contains("Text") {
            html_node.text.push_str(&get_filter_text_str(&child.raw));
            html_node.text.push(':');
        } else if child.raw.contains("port") && !child.raw.contains("id") {
            html_node.text.push_str(&child.text);
        } else {
            new_children.push(child);
        }
    }
    html_node.children = new_children;
    if html_node.id.is_empty() {
        let idd = html_node
            .children
            .get(0)
            .map(|v| v.id.clone())
            .map(|v| v.split(":").map(|v| v.to_string()).collect::<Vec<String>>())
            .and_then(|v| v.get(0).cloned())
            .unwrap_or_default();
        html_node.id = idd;
    }

    Ok(())
}

#[auto_context::auto_context]
fn raw_html_node_to_html_node(raw: crate::model::HtmlNodeRaw) -> crate::model::HtmlNode {
    let mut ans = crate::model::HtmlNode::default();
    ans.id = raw.id;
    ans.text = raw.text;
    ans.children = raw
        .children
        .into_iter()
        .map(|v| raw_html_node_to_html_node(v))
        .collect();
    ans
}

#[auto_context::auto_context]
fn check_html(raw_html: String) -> Result<crate::model::HtmlNode> {
    let html = scraper::Html::parse_document(&raw_html);
    let tree = html.tree;
    let root = tree.root();
    let mut html_node = crate::model::HtmlNodeRaw::default();
    let mut fa_children = Vec::new();
    check_html_node(root, &mut html_node, &mut fa_children)?;
    html_node.children.extend(fa_children);
    Ok(raw_html_node_to_html_node(html_node))
}

fn check_graph_node(node: &crate::model::HtmlNode, node_set: &mut HashSet<String>) -> Result<()> {
    node_set.insert(node.id.clone());
    for child in node.children.iter() {
        check_graph_node(child, node_set)?;
    }
    Ok(())
}

#[auto_context::auto_context]
fn check_graph(graph: &mut crate::model::Graph) -> Result<()> {
    let mut edge_set = HashSet::new();
    for edge in graph.edges.iter() {
        edge_set.insert(edge.from.clone());
        edge_set.insert(edge.to.clone());
    }
    let mut node_set = HashSet::new();
    for node in graph.nodes.iter() {
        check_graph_node(node, &mut node_set)?;
    }
    let mut edge_not_node = HashSet::new();
    for edge in edge_set.iter() {
        if !node_set.contains(edge) {
            edge_not_node.insert(edge.clone());
        }
    }
    graph.edges_not_node = edge_not_node.into_iter().collect();
    Ok(())
}

#[auto_context::auto_context]
pub fn parse_from_dot(dot_path: &str) -> Result<crate::model::Graph> {
    let raw_dot = read_to_string(dot_path)?;
    let g: Graph =
        parse(&raw_dot).map_err(|e| anyhow::anyhow!("Failed to parse dot file: {}", e))?;
    let mut cg = crate::model::Graph::default();
    if let Graph::DiGraph { id, stmts, .. } = g {
        cg.id = format!("{}", id).replace("\"", "");
        for stmt in stmts {
            match stmt {
                Stmt::Node(node) => {
                    let node_html = check_node(node)?;
                    cg.nodes.push(node_html)
                }
                Stmt::Edge(edge) => {
                    let edge = check_edge(edge)?;
                    cg.edges.push(edge);
                }
                Stmt::Subgraph(_subgraph) => {}
                Stmt::GAttribute(_graph_attributes) => {}
                _ => {
                    return Err(anyhow::anyhow!("check"));
                }
            }
        }
    }
    check_graph(&mut cg)?;
    // write_to_file(&serde_json::to_string(&cg)?, "data/kaspa.json")?;
    Ok(cg)
}
