use std::collections::VecDeque;

use crate::LayoutOptions;

use super::{LayoutEdge, LayoutNode, index_nodes};

pub(crate) fn layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
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
            place_node(&mut nodes[*index], rank, order, options);
        }
    }
}

fn place_node(node: &mut LayoutNode, rank: usize, order: usize, options: &LayoutOptions) {
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
