use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::{HashMap, VecDeque};
use wasm_bindgen::prelude::*;

const DEFAULT_NODE_WIDTH: f64 = 180.0;
const DEFAULT_NODE_HEIGHT: f64 = 72.0;
const DEFAULT_SPACING_X: f64 = 120.0;
const DEFAULT_SPACING_Y: f64 = 96.0;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutOptions {
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
    #[serde(default = "default_direction")]
    pub direction: String,
    #[serde(default = "default_spacing_x")]
    pub spacing_x: f64,
    #[serde(default = "default_spacing_y")]
    pub spacing_y: f64,
    #[serde(default = "default_node_width")]
    pub node_width: f64,
    #[serde(default = "default_node_height")]
    pub node_height: f64,
    #[serde(default)]
    pub center_x: f64,
    #[serde(default)]
    pub center_y: f64,
    #[serde(default = "default_iterations")]
    pub iterations: usize,
}

#[derive(Debug, Clone)]
struct LayoutNode {
    id: String,
    width: f64,
    height: f64,
    x: f64,
    y: f64,
}

#[derive(Debug, Clone)]
struct LayoutEdge {
    source: String,
    target: String,
}

#[wasm_bindgen]
pub fn graphexa_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

#[wasm_bindgen]
pub fn layout_react_flow(
    nodes_json: &str,
    edges_json: &str,
    options_json: Option<String>,
) -> Result<String, JsValue> {
    let mut node_values: Vec<Value> = parse_json(nodes_json, "nodes")?;
    let edge_values: Vec<Value> = parse_json(edges_json, "edges")?;
    let options = parse_options(options_json)?;

    let mut layout_nodes = read_nodes(&node_values, &options)?;
    let layout_edges = read_edges(&edge_values);
    apply_layout(&mut layout_nodes, &layout_edges, &options);
    write_positions(&mut node_values, &layout_nodes);

    serde_json::to_string(&node_values).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn layout_react_flow_graph(
    graph_json: &str,
    options_json: Option<String>,
) -> Result<String, JsValue> {
    let mut graph: Value = parse_json(graph_json, "graph")?;
    let nodes_json = graph
        .get("nodes")
        .ok_or_else(|| JsValue::from_str("graph.nodes is required"))?
        .to_string();
    let edges_json = graph
        .get("edges")
        .ok_or_else(|| JsValue::from_str("graph.edges is required"))?
        .to_string();
    let layouted_nodes_json = layout_react_flow(&nodes_json, &edges_json, options_json)?;
    let layouted_nodes: Value = serde_json::from_str(&layouted_nodes_json)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;

    if let Some(object) = graph.as_object_mut() {
        object.insert("nodes".to_owned(), layouted_nodes);
    }

    serde_json::to_string(&graph).map_err(|error| JsValue::from_str(&error.to_string()))
}

fn apply_layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
    match options.algorithm.as_str() {
        "radial" => radial_layout(nodes, edges, options),
        "force" => force_layout(nodes, edges, options),
        "grid" => grid_layout(nodes, options),
        _ => layered_layout(nodes, edges, options),
    }
}

fn layered_layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
    let node_index = index_nodes(nodes);
    let mut incoming = vec![0usize; nodes.len()];
    let mut outgoing = vec![Vec::new(); nodes.len()];

    for edge in edges {
        if let (Some(source), Some(target)) =
            (node_index.get(&edge.source), node_index.get(&edge.target))
        {
            outgoing[*source].push(*target);
            incoming[*target] += 1;
        }
    }

    let mut queue = VecDeque::new();
    let mut ranks = vec![0usize; nodes.len()];

    for (index, count) in incoming.iter().enumerate() {
        if *count == 0 {
            queue.push_back(index);
        }
    }

    if queue.is_empty() {
        for index in 0..nodes.len() {
            queue.push_back(index);
        }
    }

    while let Some(index) = queue.pop_front() {
        for target in &outgoing[index] {
            ranks[*target] = ranks[*target].max(ranks[index] + 1);
            incoming[*target] = incoming[*target].saturating_sub(1);
            if incoming[*target] == 0 {
                queue.push_back(*target);
            }
        }
    }

    let mut layers: Vec<Vec<usize>> = Vec::new();
    for (index, rank) in ranks.iter().enumerate() {
        if layers.len() <= *rank {
            layers.resize_with(*rank + 1, Vec::new);
        }
        layers[*rank].push(index);
    }

    for (rank, layer) in layers.iter().enumerate() {
        for (order, index) in layer.iter().enumerate() {
            place_layered_node(&mut nodes[*index], rank, order, options);
        }
    }
}

fn grid_layout(nodes: &mut [LayoutNode], options: &LayoutOptions) {
    if nodes.is_empty() {
        return;
    }

    let columns = (nodes.len() as f64).sqrt().ceil() as usize;
    for (index, node) in nodes.iter_mut().enumerate() {
        let row = index / columns;
        let column = index % columns;
        node.x = column as f64 * (node.width + options.spacing_x);
        node.y = row as f64 * (node.height + options.spacing_y);
    }
}

fn radial_layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
    if nodes.is_empty() {
        return;
    }

    let node_index = index_nodes(nodes);
    let mut degree = vec![0usize; nodes.len()];
    for edge in edges {
        if let Some(source) = node_index.get(&edge.source) {
            degree[*source] += 1;
        }
        if let Some(target) = node_index.get(&edge.target) {
            degree[*target] += 1;
        }
    }

    let radius =
        ((nodes.len() as f64) * (options.node_width + options.spacing_x)) / std::f64::consts::TAU;
    let mut ordered = (0..nodes.len()).collect::<Vec<_>>();
    ordered.sort_by_key(|index| std::cmp::Reverse(degree[*index]));

    for (order, index) in ordered.iter().enumerate() {
        let angle = (order as f64 / nodes.len() as f64) * std::f64::consts::TAU;
        nodes[*index].x = options.center_x + radius * angle.cos();
        nodes[*index].y = options.center_y + radius * angle.sin();
    }
}

fn force_layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
    radial_layout(nodes, edges, options);

    if nodes.len() < 2 {
        return;
    }

    let node_index = index_nodes(nodes);
    let spring_length = options.node_width + options.spacing_x;
    let repulsion = spring_length * spring_length;
    let step = 0.02;

    for _ in 0..options.iterations {
        let mut delta = vec![(0.0, 0.0); nodes.len()];

        for a in 0..nodes.len() {
            for b in (a + 1)..nodes.len() {
                let dx = nodes[a].x - nodes[b].x;
                let dy = nodes[a].y - nodes[b].y;
                let distance_sq = (dx * dx + dy * dy).max(0.01);
                let distance = distance_sq.sqrt();
                let force = repulsion / distance_sq;
                let fx = (dx / distance) * force;
                let fy = (dy / distance) * force;
                delta[a].0 += fx;
                delta[a].1 += fy;
                delta[b].0 -= fx;
                delta[b].1 -= fy;
            }
        }

        for edge in edges {
            if let (Some(source), Some(target)) =
                (node_index.get(&edge.source), node_index.get(&edge.target))
            {
                let dx = nodes[*target].x - nodes[*source].x;
                let dy = nodes[*target].y - nodes[*source].y;
                let distance = (dx * dx + dy * dy).sqrt().max(0.01);
                let force = (distance - spring_length) * 0.08;
                let fx = (dx / distance) * force;
                let fy = (dy / distance) * force;
                delta[*source].0 += fx;
                delta[*source].1 += fy;
                delta[*target].0 -= fx;
                delta[*target].1 -= fy;
            }
        }

        for (index, node) in nodes.iter_mut().enumerate() {
            node.x += delta[index].0 * step;
            node.y += delta[index].1 * step;
        }
    }
}

fn place_layered_node(node: &mut LayoutNode, rank: usize, order: usize, options: &LayoutOptions) {
    let primary = rank as f64;
    let secondary = order as f64;
    if options.direction.eq_ignore_ascii_case("RIGHT")
        || options.direction.eq_ignore_ascii_case("LR")
    {
        node.x = primary * (node.width + options.spacing_x);
        node.y = secondary * (node.height + options.spacing_y);
    } else {
        node.x = secondary * (node.width + options.spacing_x);
        node.y = primary * (node.height + options.spacing_y);
    }
}

fn read_nodes(values: &[Value], options: &LayoutOptions) -> Result<Vec<LayoutNode>, JsValue> {
    values
        .iter()
        .map(|value| {
            let id = value
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| JsValue::from_str("Every React Flow node must have a string id"))?;
            let position = value.get("position");

            Ok(LayoutNode {
                id: id.to_owned(),
                width: number_at(value, "width").unwrap_or(options.node_width),
                height: number_at(value, "height").unwrap_or(options.node_height),
                x: position
                    .and_then(|position| number_at(position, "x"))
                    .unwrap_or(0.0),
                y: position
                    .and_then(|position| number_at(position, "y"))
                    .unwrap_or(0.0),
            })
        })
        .collect()
}

fn read_edges(values: &[Value]) -> Vec<LayoutEdge> {
    values
        .iter()
        .filter_map(|value| {
            Some(LayoutEdge {
                source: value.get("source")?.as_str()?.to_owned(),
                target: value.get("target")?.as_str()?.to_owned(),
            })
        })
        .collect()
}

fn write_positions(values: &mut [Value], layout_nodes: &[LayoutNode]) {
    for (value, layout_node) in values.iter_mut().zip(layout_nodes) {
        if let Some(object) = value.as_object_mut() {
            let mut position = Map::new();
            position.insert("x".to_owned(), number_value(layout_node.x));
            position.insert("y".to_owned(), number_value(layout_node.y));
            object.insert("position".to_owned(), Value::Object(position));
        }
    }
}

fn index_nodes(nodes: &[LayoutNode]) -> HashMap<String, usize> {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.id.clone(), index))
        .collect()
}

fn parse_json<T>(source: &str, label: &str) -> Result<T, JsValue>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(source)
        .map_err(|error| JsValue::from_str(&format!("Invalid {label} JSON: {error}")))
}

fn parse_options(options_json: Option<String>) -> Result<LayoutOptions, JsValue> {
    match options_json {
        Some(options) if !options.trim().is_empty() => parse_json(&options, "options"),
        _ => Ok(LayoutOptions {
            algorithm: default_algorithm(),
            direction: default_direction(),
            spacing_x: default_spacing_x(),
            spacing_y: default_spacing_y(),
            node_width: default_node_width(),
            node_height: default_node_height(),
            center_x: 0.0,
            center_y: 0.0,
            iterations: default_iterations(),
        }),
    }
}

fn number_at(value: &Value, key: &str) -> Option<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

fn number_value(value: f64) -> Value {
    serde_json::Number::from_f64(value)
        .map(Value::Number)
        .unwrap_or(Value::Null)
}

fn default_algorithm() -> String {
    "layered".to_owned()
}

fn default_direction() -> String {
    "DOWN".to_owned()
}

fn default_spacing_x() -> f64 {
    DEFAULT_SPACING_X
}

fn default_spacing_y() -> f64 {
    DEFAULT_SPACING_Y
}

fn default_node_width() -> f64 {
    DEFAULT_NODE_WIDTH
}

fn default_node_height() -> f64 {
    DEFAULT_NODE_HEIGHT
}

fn default_iterations() -> usize {
    120
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layered_layout_preserves_node_data_and_sets_positions() {
        let nodes = r#"[{"id":"a","type":"entity","data":{"label":"A"},"position":{"x":9,"y":9}},{"id":"b","data":{"label":"B"},"position":{"x":0,"y":0}}]"#;
        let edges = r#"[{"id":"a-b","source":"a","target":"b"}]"#;
        let result = layout_react_flow(nodes, edges, None).unwrap();
        let layouted: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(layouted[0]["data"]["label"], "A");
        assert_eq!(layouted[0]["position"]["x"], 0.0);
        assert_eq!(
            layouted[1]["position"]["y"],
            DEFAULT_NODE_HEIGHT + DEFAULT_SPACING_Y
        );
    }

    #[test]
    fn grid_layout_places_nodes_in_rows() {
        let nodes = r#"[{"id":"a"},{"id":"b"},{"id":"c"},{"id":"d"}]"#;
        let edges = "[]";
        let options = Some(
            r#"{"algorithm":"grid","nodeWidth":100,"nodeHeight":50,"spacingX":10,"spacingY":20}"#
                .to_owned(),
        );
        let result = layout_react_flow(nodes, edges, options).unwrap();
        let layouted: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(layouted[0]["position"]["x"], 0.0);
        assert_eq!(layouted[1]["position"]["x"], 110.0);
        assert_eq!(layouted[2]["position"]["y"], 70.0);
    }

    #[test]
    fn graph_wrapper_returns_edges_unchanged() {
        let graph =
            r#"{"nodes":[{"id":"a"},{"id":"b"}],"edges":[{"id":"e","source":"a","target":"b"}]}"#;
        let result = layout_react_flow_graph(graph, None).unwrap();
        let layouted: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(layouted["edges"][0]["id"], "e");
        assert_eq!(
            layouted["nodes"][1]["position"]["y"],
            DEFAULT_NODE_HEIGHT + DEFAULT_SPACING_Y
        );
    }
}
