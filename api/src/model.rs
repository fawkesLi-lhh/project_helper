
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default)]
pub struct Edge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default)]
pub struct Graph {
    pub id: String,
    pub nodes: Vec<HtmlNode>,
    pub edges: Vec<Edge>,
    pub edges_not_node: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default)]
pub struct HtmlNodeRaw {
    pub text: String,
    pub id: String,
    pub children: Vec<HtmlNodeRaw>,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default)]
pub struct HtmlNode {
    pub text: String,
    pub id: String,
    pub children: Vec<HtmlNode>,
}

