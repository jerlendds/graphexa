mod layout;

use serde::Deserialize;
use serde_json::{Map, Value};
use wasm_bindgen::prelude::*;

use layout::{LayoutEdge, LayoutNode, apply_layout};

pub const DEFAULT_NODE_WIDTH: f64 = 180.0;
pub const DEFAULT_NODE_HEIGHT: f64 = 72.0;
pub const DEFAULT_SPACING_X: f64 = 120.0;
pub const DEFAULT_SPACING_Y: f64 = 96.0;

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
    #[serde(default = "default_max_iter")]
    pub max_iter: usize,
    #[serde(default = "default_jitter_tolerance")]
    pub jitter_tolerance: f64,
    #[serde(default = "default_scaling_ratio")]
    pub scaling_ratio: f64,
    #[serde(default = "default_gravity")]
    pub gravity: f64,
    #[serde(default)]
    pub distributed_action: bool,
    #[serde(default)]
    pub strong_gravity: bool,
    #[serde(default)]
    pub linlog: bool,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default = "default_align")]
    pub align: String,
    #[serde(default = "default_scale")]
    pub scale: f64,
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
                weight: number_at(value, "weight")
                    .or_else(|| value.get("data").and_then(|data| number_at(data, "weight")))
                    .unwrap_or(1.0),
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
            max_iter: default_max_iter(),
            jitter_tolerance: default_jitter_tolerance(),
            scaling_ratio: default_scaling_ratio(),
            gravity: default_gravity(),
            distributed_action: false,
            strong_gravity: false,
            linlog: false,
            seed: None,
            start: None,
            align: default_align(),
            scale: default_scale(),
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

fn default_max_iter() -> usize {
    100
}

fn default_jitter_tolerance() -> f64 {
    1.0
}

fn default_scaling_ratio() -> f64 {
    2.0
}

fn default_gravity() -> f64 {
    1.0
}

fn default_align() -> String {
    "vertical".to_owned()
}

fn default_scale() -> f64 {
    1.0
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

    #[test]
    fn bfs_layout_uses_start_node_and_vertical_alignment() {
        let nodes = r#"[{"id":"0"},{"id":"1"},{"id":"2"},{"id":"3"}]"#;
        let edges = r#"[{"source":"0","target":"1"},{"source":"1","target":"2"},{"source":"2","target":"3"}]"#;
        let options = Some(r#"{"algorithm":"bfs","start":"0","scale":1}"#.to_owned());
        let result = layout_react_flow(nodes, edges, options).unwrap();
        let layouted: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(layouted[0]["position"]["x"], -1.0);
        assert_eq!(layouted[1]["position"]["x"], -0.33333333333333337);
        assert_eq!(layouted[2]["position"]["x"], 0.33333333333333326);
        assert_eq!(layouted[3]["position"]["x"], 1.0);
        assert_eq!(layouted[3]["position"]["y"], 0.0);
    }

    #[test]
    fn circular_layout_places_nodes_on_scaled_circle() {
        let nodes = r#"[{"id":"a"},{"id":"b"},{"id":"c"},{"id":"d"}]"#;
        let result = layout_react_flow(
            nodes,
            "[]",
            Some(r#"{"algorithm":"circular","scale":2}"#.to_owned()),
        )
        .unwrap();
        let layouted: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(layouted[0]["position"]["x"], 2.0);
        assert!((layouted[1]["position"]["y"].as_f64().unwrap() - 2.0).abs() < 0.000001);
        assert!((layouted[2]["position"]["x"].as_f64().unwrap() + 2.0).abs() < 0.000001);
    }

    #[test]
    fn forceatlas2_layout_moves_connected_nodes() {
        let nodes = r#"[{"id":"a"},{"id":"b"},{"id":"c"}]"#;
        let edges = r#"[{"source":"a","target":"b","weight":2},{"source":"b","target":"c"}]"#;
        let result = layout_react_flow(
            nodes,
            edges,
            Some(r#"{"algorithm":"forceatlas2","maxIter":5,"seed":42}"#.to_owned()),
        )
        .unwrap();
        let layouted: Value = serde_json::from_str(&result).unwrap();

        assert!(layouted[0]["position"]["x"].as_f64().unwrap().is_finite());
        assert!(layouted[1]["position"]["y"].as_f64().unwrap().is_finite());
        assert_ne!(layouted[0]["position"], layouted[1]["position"]);
    }

    #[test]
    fn kamada_kawai_layout_rescales_weighted_path() {
        let nodes = r#"[{"id":"0"},{"id":"1"},{"id":"2"},{"id":"3"}]"#;
        let edges = r#"[{"source":"0","target":"1","weight":1},{"source":"1","target":"2","weight":1},{"source":"2","target":"3","weight":1}]"#;
        let result = layout_react_flow(
            nodes,
            edges,
            Some(r#"{"algorithm":"kamada_kawai","iterations":20,"scale":1}"#.to_owned()),
        )
        .unwrap();
        let layouted: Value = serde_json::from_str(&result).unwrap();

        assert!(layouted[0]["position"]["x"].as_f64().unwrap().is_finite());
        assert!(layouted[3]["position"]["y"].as_f64().unwrap().is_finite());
        assert_ne!(layouted[0]["position"], layouted[3]["position"]);
    }
}
