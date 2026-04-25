use std::collections::HashMap;

use crate::LayoutOptions;

use super::{LayoutEdge, LayoutNode, grid, index_nodes};

pub(crate) fn layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
    if nodes.is_empty() || nodes.iter().all(|node| node.has_position) {
        return;
    }

    if nodes.iter().all(|node| !node.has_position) {
        grid::layout(nodes, options);
        return;
    }

    let node_index = index_nodes(nodes);
    let mut adjacency = vec![Vec::new(); nodes.len()];
    for edge in edges {
        if let (Some(source), Some(target)) =
            (node_index.get(&edge.source), node_index.get(&edge.target))
        {
            adjacency[*source].push(*target);
            adjacency[*target].push(*source);
        }
    }

    let fallback_origin = fallback_origin(nodes, options);
    let mut anchor_counts: HashMap<usize, usize> = HashMap::new();
    let anchor_layouts = anchor_layouts(nodes, &adjacency, options);
    let mut fallback_count = 0usize;
    let mut placements = Vec::new();

    for index in 0..nodes.len() {
        if nodes[index].has_position {
            continue;
        }

        if let Some(anchor) = adjacency[index]
            .iter()
            .copied()
            .find(|neighbor| nodes[*neighbor].has_position)
        {
            let count = anchor_counts.entry(anchor).or_insert(0);
            let layout = anchor_layouts
                .get(&anchor)
                .copied()
                .unwrap_or_else(|| AnchorLayout::new(1, options.node_width, options.node_height));
            let position = avoid_overlaps(
                anchored_position(&nodes[anchor], *count, layout, options),
                index,
                nodes,
                &placements,
                options,
            );
            *count += 1;
            placements.push((index, position));
        } else {
            let position = avoid_overlaps(
                fallback_position(fallback_origin, fallback_count, options),
                index,
                nodes,
                &placements,
                options,
            );
            fallback_count += 1;
            placements.push((index, position));
        }
    }

    for (index, (x, y)) in placements {
        nodes[index].x = x;
        nodes[index].y = y;
    }
}

#[derive(Clone, Copy)]
struct AnchorLayout {
    total: usize,
    max_width: f64,
    max_height: f64,
}

impl AnchorLayout {
    fn new(total: usize, max_width: f64, max_height: f64) -> Self {
        Self {
            total,
            max_width,
            max_height,
        }
    }
}

fn anchor_layouts(
    nodes: &[LayoutNode],
    adjacency: &[Vec<usize>],
    options: &LayoutOptions,
) -> HashMap<usize, AnchorLayout> {
    let mut layouts: HashMap<usize, AnchorLayout> = HashMap::new();

    for index in 0..nodes.len() {
        if nodes[index].has_position {
            continue;
        }

        if let Some(anchor) = adjacency[index]
            .iter()
            .copied()
            .find(|neighbor| nodes[*neighbor].has_position)
        {
            let layout = layouts
                .entry(anchor)
                .or_insert_with(|| AnchorLayout::new(0, options.node_width, options.node_height));
            layout.total += 1;
            layout.max_width = layout.max_width.max(nodes[index].width);
            layout.max_height = layout.max_height.max(nodes[index].height);
        }
    }

    layouts
}

fn anchored_position(
    anchor: &LayoutNode,
    order: usize,
    layout: AnchorLayout,
    options: &LayoutOptions,
) -> (f64, f64) {
    let columns = if layout.total >= 10 {
        5
    } else {
        layout.total.max(1)
    };
    let row = order / columns;
    let column = order % columns;
    let horizontal_step = layout.max_width + options.spacing_x;
    let vertical_step = layout.max_height + options.spacing_y;

    (
        anchor.x + anchor.width + options.spacing_x + column as f64 * horizontal_step,
        anchor.y + row as f64 * vertical_step,
    )
}

fn avoid_overlaps(
    mut position: (f64, f64),
    node_index: usize,
    nodes: &[LayoutNode],
    placements: &[(usize, (f64, f64))],
    options: &LayoutOptions,
) -> (f64, f64) {
    let max_passes = nodes.len().saturating_mul(4).max(8);

    for _ in 0..max_passes {
        let mut next_y = position.1;

        for obstacle in nodes.iter().filter(|node| node.has_position) {
            if bounds_overlap_at(
                &nodes[node_index],
                position,
                obstacle,
                (obstacle.x, obstacle.y),
                options,
            ) {
                next_y = next_y.max(obstacle.y + obstacle.height + options.spacing_y.max(0.0));
            }
        }

        for (placed_index, placed_position) in placements {
            if bounds_overlap_at(
                &nodes[node_index],
                position,
                &nodes[*placed_index],
                *placed_position,
                options,
            ) {
                next_y = next_y.max(
                    placed_position.1 + nodes[*placed_index].height + options.spacing_y.max(0.0),
                );
            }
        }

        if next_y <= position.1 + 0.000001 {
            return position;
        }

        position.1 = next_y;
    }

    position
}

fn bounds_overlap_at(
    node: &LayoutNode,
    position: (f64, f64),
    obstacle: &LayoutNode,
    obstacle_position: (f64, f64),
    options: &LayoutOptions,
) -> bool {
    let padding_x = options.spacing_x.max(0.0) * 0.25;
    let padding_y = options.spacing_y.max(0.0) * 0.25;

    position.0 < obstacle_position.0 + obstacle.width + padding_x
        && position.0 + node.width + padding_x > obstacle_position.0
        && position.1 < obstacle_position.1 + obstacle.height + padding_y
        && position.1 + node.height + padding_y > obstacle_position.1
}

fn fallback_origin(nodes: &[LayoutNode], options: &LayoutOptions) -> (f64, f64) {
    let max_x = nodes
        .iter()
        .filter(|node| node.has_position)
        .map(|node| node.x)
        .fold(options.center_x, f64::max);
    let min_y = nodes
        .iter()
        .filter(|node| node.has_position)
        .map(|node| node.y)
        .fold(options.center_y, f64::min);

    (max_x + options.node_width + options.spacing_x, min_y)
}

fn fallback_position(origin: (f64, f64), order: usize, options: &LayoutOptions) -> (f64, f64) {
    let columns = ((order + 1) as f64).sqrt().ceil().max(1.0) as usize;
    let row = order / columns;
    let column = order % columns;

    (
        origin.0 + column as f64 * (options.node_width + options.spacing_x),
        origin.1 + row as f64 * (options.node_height + options.spacing_y),
    )
}
