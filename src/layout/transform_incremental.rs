use crate::LayoutOptions;

use super::{LayoutEdge, LayoutNode, index_nodes, transform_locked};

pub(crate) fn layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
    let movable = nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| (!node.has_position).then_some(index))
        .collect::<Vec<_>>();

    if movable.is_empty() {
        return;
    }

    transform_locked::layout(nodes, edges, options);

    let node_index = index_nodes(nodes);
    let largest_node = nodes.iter().fold(
        options.node_width.max(options.node_height),
        |largest, node| largest.max(node.width).max(node.height),
    );
    let spring_length = largest_node + options.spacing_x.max(options.spacing_y);
    let repulsion = spring_length * spring_length;
    let step = 0.015;

    for _ in 0..options.iterations {
        let mut delta = vec![(0.0, 0.0); nodes.len()];

        for movable_index in &movable {
            for other_index in 0..nodes.len() {
                if *movable_index == other_index {
                    continue;
                }

                let dx = nodes[*movable_index].x - nodes[other_index].x;
                let dy = nodes[*movable_index].y - nodes[other_index].y;
                let distance_sq = (dx * dx + dy * dy).max(0.01);
                let distance = distance_sq.sqrt();
                let force = repulsion / distance_sq;
                delta[*movable_index].0 += (dx / distance) * force;
                delta[*movable_index].1 += (dy / distance) * force;
            }
        }

        for edge in edges {
            if let (Some(source), Some(target)) =
                (node_index.get(&edge.source), node_index.get(&edge.target))
            {
                let source_movable = !nodes[*source].has_position;
                let target_movable = !nodes[*target].has_position;
                if !source_movable && !target_movable {
                    continue;
                }

                let dx = nodes[*target].x - nodes[*source].x;
                let dy = nodes[*target].y - nodes[*source].y;
                let distance = (dx * dx + dy * dy).sqrt().max(0.01);
                let force = (distance - spring_length) * 0.08 * edge.weight.max(0.0);
                let fx = (dx / distance) * force;
                let fy = (dy / distance) * force;

                if source_movable {
                    delta[*source].0 += fx;
                    delta[*source].1 += fy;
                }
                if target_movable {
                    delta[*target].0 -= fx;
                    delta[*target].1 -= fy;
                }
            }
        }

        for index in &movable {
            nodes[*index].x += delta[*index].0 * step;
            nodes[*index].y += delta[*index].1 * step;
        }

        separate_overlaps(nodes, &movable, options);
    }

    let final_passes = movable.len().saturating_mul(2).clamp(8, 64);
    for _ in 0..final_passes {
        if !separate_overlaps(nodes, &movable, options) {
            break;
        }
    }

    settle_overlaps_down(nodes, &movable, options);
}

fn separate_overlaps(nodes: &mut [LayoutNode], movable: &[usize], options: &LayoutOptions) -> bool {
    let mut moved = false;

    for movable_offset in 0..movable.len() {
        let a = movable[movable_offset];
        for b in 0..nodes.len() {
            if a == b {
                continue;
            }

            let b_movable = !nodes[b].has_position;
            if b_movable
                && movable
                    .iter()
                    .position(|index| *index == b)
                    .unwrap_or(usize::MAX)
                    < movable_offset
            {
                continue;
            }

            let overlap = overlap_vector(&nodes[a], &nodes[b], a, b, options);
            let Some((push_x, push_y)) = overlap else {
                continue;
            };

            if b_movable {
                nodes[a].x += push_x * 0.5;
                nodes[a].y += push_y * 0.5;
                nodes[b].x -= push_x * 0.5;
                nodes[b].y -= push_y * 0.5;
            } else {
                nodes[a].x += push_x;
                nodes[a].y += push_y;
            }
            moved = true;
        }
    }

    moved
}

fn overlap_vector(
    a: &LayoutNode,
    b: &LayoutNode,
    a_index: usize,
    b_index: usize,
    options: &LayoutOptions,
) -> Option<(f64, f64)> {
    let padding_x = options.spacing_x.max(0.0) * 0.5;
    let padding_y = options.spacing_y.max(0.0) * 0.5;
    let a_center_x = a.x + a.width * 0.5;
    let a_center_y = a.y + a.height * 0.5;
    let b_center_x = b.x + b.width * 0.5;
    let b_center_y = b.y + b.height * 0.5;
    let dx = a_center_x - b_center_x;
    let dy = a_center_y - b_center_y;
    let min_x = (a.width + b.width) * 0.5 + padding_x;
    let min_y = (a.height + b.height) * 0.5 + padding_y;
    let overlap_x = min_x - dx.abs();
    let overlap_y = min_y - dy.abs();

    if overlap_x <= 0.0 || overlap_y <= 0.0 {
        return None;
    }

    if overlap_x < overlap_y {
        let direction = non_zero_direction(dx, a_index, b_index);
        Some((direction * overlap_x, 0.0))
    } else {
        let direction = non_zero_direction(dy, a_index, b_index);
        Some((0.0, direction * overlap_y))
    }
}

fn non_zero_direction(value: f64, a_index: usize, b_index: usize) -> f64 {
    if value.abs() > 0.000001 {
        value.signum()
    } else if a_index <= b_index {
        -1.0
    } else {
        1.0
    }
}

fn settle_overlaps_down(nodes: &mut [LayoutNode], movable: &[usize], options: &LayoutOptions) {
    let mut ordered = movable.to_vec();
    ordered.sort_by(|a, b| {
        nodes[*a]
            .y
            .total_cmp(&nodes[*b].y)
            .then_with(|| nodes[*a].x.total_cmp(&nodes[*b].x))
    });

    let fixed = nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| node.has_position.then_some(index))
        .collect::<Vec<_>>();
    let mut settled = Vec::new();

    for index in ordered {
        for _ in 0..nodes.len().saturating_mul(2).max(4) {
            let mut next_y = nodes[index].y;

            for other in fixed.iter().chain(settled.iter()) {
                if bounds_overlap(&nodes[index], &nodes[*other], options) {
                    next_y = next_y
                        .max(nodes[*other].y + nodes[*other].height + options.spacing_y.max(0.0));
                }
            }

            if next_y <= nodes[index].y + 0.000001 {
                break;
            }

            nodes[index].y = next_y;
        }

        settled.push(index);
    }
}

fn bounds_overlap(a: &LayoutNode, b: &LayoutNode, options: &LayoutOptions) -> bool {
    let padding_x = options.spacing_x.max(0.0) * 0.25;
    let padding_y = options.spacing_y.max(0.0) * 0.25;

    a.x < b.x + b.width + padding_x
        && a.x + a.width + padding_x > b.x
        && a.y < b.y + b.height + padding_y
        && a.y + a.height + padding_y > b.y
}
