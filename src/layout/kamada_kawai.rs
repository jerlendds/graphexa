use crate::LayoutOptions;

use super::{LayoutEdge, LayoutNode, index_nodes};

const DISCONNECTED_DISTANCE: f64 = 1_000_000.0;
const EPSILON: f64 = 0.000001;

pub(crate) fn layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
    match nodes.len() {
        0 => return,
        1 => {
            nodes[0].x = options.center_x;
            nodes[0].y = options.center_y;
            return;
        }
        _ => {}
    }

    initialize_circular(nodes);
    let distances = shortest_path_distances(nodes, edges);
    solve(nodes, &distances, options.iterations.max(1));
    rescale(nodes, options);
}

fn initialize_circular(nodes: &mut [LayoutNode]) {
    let len = nodes.len();
    for (index, node) in nodes.iter_mut().enumerate() {
        let theta = (index as f64 / len as f64) * std::f64::consts::TAU;
        node.x = theta.cos();
        node.y = theta.sin();
    }
}

fn shortest_path_distances(nodes: &[LayoutNode], edges: &[LayoutEdge]) -> Vec<Vec<f64>> {
    let node_index = index_nodes(nodes);
    let mut distances = vec![vec![DISCONNECTED_DISTANCE; nodes.len()]; nodes.len()];

    for (index, row) in distances.iter_mut().enumerate() {
        row[index] = 0.0;
    }

    for edge in edges {
        if let (Some(source), Some(target)) =
            (node_index.get(&edge.source), node_index.get(&edge.target))
        {
            let weight = edge.weight.max(EPSILON);
            distances[*source][*target] = distances[*source][*target].min(weight);
            distances[*target][*source] = distances[*target][*source].min(weight);
        }
    }

    for pivot in 0..nodes.len() {
        for source in 0..nodes.len() {
            for target in 0..nodes.len() {
                let through_pivot = distances[source][pivot] + distances[pivot][target];
                if through_pivot < distances[source][target] {
                    distances[source][target] = through_pivot;
                }
            }
        }
    }

    distances
}

fn solve(nodes: &mut [LayoutNode], distances: &[Vec<f64>], iterations: usize) {
    let step = 0.08;

    for _ in 0..iterations {
        let snapshot = nodes
            .iter()
            .map(|node| (node.x, node.y))
            .collect::<Vec<_>>();
        let mut max_movement: f64 = 0.0;

        for i in 0..nodes.len() {
            let mut dx_total = 0.0;
            let mut dy_total = 0.0;

            for j in 0..nodes.len() {
                if i == j || distances[i][j] >= DISCONNECTED_DISTANCE {
                    continue;
                }

                let dx = snapshot[i].0 - snapshot[j].0;
                let dy = snapshot[i].1 - snapshot[j].1;
                let actual = (dx * dx + dy * dy).sqrt().max(EPSILON);
                let target = distances[i][j];
                let spring = 1.0 / (target * target).max(EPSILON);
                let force = spring * (actual - target);

                dx_total -= force * dx / actual;
                dy_total -= force * dy / actual;
            }

            let dx = dx_total * step;
            let dy = dy_total * step;
            nodes[i].x += dx;
            nodes[i].y += dy;
            max_movement = max_movement.max(dx.abs() + dy.abs());
        }

        if max_movement < 1e-9 {
            break;
        }
    }
}

fn rescale(nodes: &mut [LayoutNode], options: &LayoutOptions) {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for node in nodes.iter() {
        min_x = min_x.min(node.x);
        max_x = max_x.max(node.x);
        min_y = min_y.min(node.y);
        max_y = max_y.max(node.y);
    }

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let span = (max_x - min_x).max(max_y - min_y).max(EPSILON);

    for node in nodes {
        node.x = options.center_x + ((node.x - center_x) / span) * 2.0 * options.scale;
        node.y = options.center_y + ((node.y - center_y) / span) * 2.0 * options.scale;
    }
}
