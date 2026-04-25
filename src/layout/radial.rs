use crate::LayoutOptions;

use super::{LayoutEdge, LayoutNode, index_nodes};

pub(crate) fn layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
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
