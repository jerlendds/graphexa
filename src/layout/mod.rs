mod bfs;
mod circular;
mod force;
mod forceatlas2;
mod grid;
mod kamada_kawai;
mod layered;
mod radial;

use std::collections::HashMap;

use crate::LayoutOptions;

#[derive(Debug, Clone)]
pub(crate) struct LayoutNode {
    pub(crate) id: String,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) x: f64,
    pub(crate) y: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct LayoutEdge {
    pub(crate) source: String,
    pub(crate) target: String,
    pub(crate) weight: f64,
}

pub(crate) fn apply_layout(
    nodes: &mut [LayoutNode],
    edges: &[LayoutEdge],
    options: &LayoutOptions,
) {
    match options.algorithm.as_str() {
        "bfs" => bfs::layout(nodes, edges, options),
        "circular" => circular::layout(nodes, options),
        "forceatlas2" => forceatlas2::layout(nodes, edges, options),
        "radial" => radial::layout(nodes, edges, options),
        "kamada_kawai" | "kamada-kawai" => kamada_kawai::layout(nodes, edges, options),
        "force" => force::layout(nodes, edges, options),
        "grid" => grid::layout(nodes, options),
        _ => layered::layout(nodes, edges, options),
    }
}

pub(crate) fn index_nodes(nodes: &[LayoutNode]) -> HashMap<String, usize> {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.id.clone(), index))
        .collect()
}
