use std::collections::VecDeque;

use crate::LayoutOptions;

use super::{LayoutEdge, LayoutNode, index_nodes};

pub(crate) fn layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
    if nodes.is_empty() {
        return;
    }

    let node_index = index_nodes(nodes);
    let start = options
        .start
        .as_ref()
        .and_then(|id| node_index.get(id))
        .copied()
        .unwrap_or(0);

    let adjacency = build_adjacency(nodes.len(), edges, &node_index);
    let layers = bfs_layers(start, &adjacency, nodes.len());
    let positions = multipartite_positions(&layers, options);

    for (index, (x, y)) in positions.into_iter().enumerate() {
        nodes[index].x = x;
        nodes[index].y = y;
    }
}

fn build_adjacency(
    node_count: usize,
    edges: &[LayoutEdge],
    node_index: &std::collections::HashMap<String, usize>,
) -> Vec<Vec<usize>> {
    let mut adjacency = vec![Vec::new(); node_count];

    for edge in edges {
        if let (Some(source), Some(target)) =
            (node_index.get(&edge.source), node_index.get(&edge.target))
        {
            adjacency[*source].push(*target);
            adjacency[*target].push(*source);
        }
    }

    adjacency
}

fn bfs_layers(start: usize, adjacency: &[Vec<usize>], node_count: usize) -> Vec<Vec<usize>> {
    let mut layers = Vec::new();
    let mut visited = vec![false; node_count];
    let mut queue = VecDeque::from([(start, 0usize)]);
    visited[start] = true;

    while let Some((node, depth)) = queue.pop_front() {
        if layers.len() <= depth {
            layers.push(Vec::new());
        }
        layers[depth].push(node);

        for neighbor in &adjacency[node] {
            if !visited[*neighbor] {
                visited[*neighbor] = true;
                queue.push_back((*neighbor, depth + 1));
            }
        }
    }

    for index in 0..node_count {
        if !visited[index] {
            if layers.is_empty() {
                layers.push(Vec::new());
            }
            layers[0].push(index);
        }
    }

    layers
}

fn multipartite_positions(layers: &[Vec<usize>], options: &LayoutOptions) -> Vec<(f64, f64)> {
    let node_count = layers.iter().map(Vec::len).sum::<usize>();
    let mut positions = vec![(0.0, 0.0); node_count];

    if node_count == 1 {
        let node = layers.iter().find_map(|layer| layer.first()).copied();
        if let Some(index) = node {
            positions[index] = (options.center_x, options.center_y);
        }
        return positions;
    }

    let denominator = (node_count - 1) as f64;
    let mut order = 0usize;

    for layer in layers {
        for node in layer {
            let primary = normalized_axis(order, denominator, options.scale);
            let secondary = centered_layer_offset(layer.len(), layer, *node, options.scale);

            positions[*node] = if options.align.eq_ignore_ascii_case("horizontal") {
                (options.center_x + secondary, options.center_y + primary)
            } else {
                (options.center_x + primary, options.center_y + secondary)
            };

            order += 1;
        }
    }

    positions
}

fn normalized_axis(order: usize, denominator: f64, scale: f64) -> f64 {
    -scale + (2.0 * scale * order as f64 / denominator)
}

fn centered_layer_offset(layer_len: usize, layer: &[usize], node: usize, scale: f64) -> f64 {
    if layer_len <= 1 {
        return 0.0;
    }

    let index = layer
        .iter()
        .position(|candidate| *candidate == node)
        .unwrap_or(0);
    -scale + (2.0 * scale * index as f64 / (layer_len - 1) as f64)
}
