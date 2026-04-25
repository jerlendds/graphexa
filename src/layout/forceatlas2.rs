use crate::LayoutOptions;

use super::{LayoutEdge, LayoutNode, index_nodes};

pub(crate) fn layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
    if nodes.is_empty() {
        return;
    }

    initialize_positions(nodes, options.seed.unwrap_or(1));

    let node_index = index_nodes(nodes);
    let mut mass = vec![1.0; nodes.len()];
    for edge in edges {
        if let Some(source) = node_index.get(&edge.source) {
            mass[*source] += edge.weight.max(0.0);
        }
        if let Some(target) = node_index.get(&edge.target) {
            mass[*target] += edge.weight.max(0.0);
        }
    }

    let mut previous_update = vec![(0.0, 0.0); nodes.len()];
    let mut speed = 1.0;
    let mut speed_efficiency = 1.0;

    for _ in 0..options.max_iter {
        let mut update = vec![(0.0, 0.0); nodes.len()];
        apply_repulsion(nodes, &mass, &mut update, options);
        apply_attraction(nodes, edges, &node_index, &mass, &mut update, options);
        apply_gravity(nodes, &mass, &mut update, options);

        let (swing, traction) = swing_and_traction(&mass, &update, &previous_update);
        let (next_speed, next_efficiency) = estimate_factor(
            nodes.len(),
            swing,
            traction,
            speed,
            speed_efficiency,
            options.jitter_tolerance,
        );
        speed = next_speed;
        speed_efficiency = next_efficiency;

        let mut movement = 0.0;
        for (index, node) in nodes.iter_mut().enumerate() {
            let update_len = vector_len(update[index]).max(0.000001);
            let swinging =
                mass[index] * vector_len(vector_sub(update[index], previous_update[index]));
            let factor = speed / (1.0 + (speed * swinging).sqrt());
            let dx = update[index].0 * factor / update_len.min(1.0).max(0.000001);
            let dy = update[index].1 * factor / update_len.min(1.0).max(0.000001);
            node.x += dx;
            node.y += dy;
            movement += dx.abs() + dy.abs();
        }

        previous_update = update;
        if movement < 1e-10 {
            break;
        }
    }
}

fn initialize_positions(nodes: &mut [LayoutNode], seed: u64) {
    let has_positions = nodes
        .iter()
        .any(|node| node.x.abs() > f64::EPSILON || node.y.abs() > f64::EPSILON);
    if has_positions {
        return;
    }

    let mut rng = SplitMix64::new(seed);
    for node in nodes {
        node.x = rng.next_unit() - 0.5;
        node.y = rng.next_unit() - 0.5;
    }
}

fn apply_repulsion(
    nodes: &[LayoutNode],
    mass: &[f64],
    update: &mut [(f64, f64)],
    options: &LayoutOptions,
) {
    for a in 0..nodes.len() {
        for b in (a + 1)..nodes.len() {
            let dx = nodes[a].x - nodes[b].x;
            let dy = nodes[a].y - nodes[b].y;
            let distance_sq = (dx * dx + dy * dy).max(0.000001);
            let distance = distance_sq.sqrt();
            let force = options.scaling_ratio * mass[a] * mass[b] / distance_sq;
            let fx = dx / distance * force;
            let fy = dy / distance * force;
            update[a].0 += fx;
            update[a].1 += fy;
            update[b].0 -= fx;
            update[b].1 -= fy;
        }
    }
}

fn apply_attraction(
    nodes: &[LayoutNode],
    edges: &[LayoutEdge],
    node_index: &std::collections::HashMap<String, usize>,
    mass: &[f64],
    update: &mut [(f64, f64)],
    options: &LayoutOptions,
) {
    for edge in edges {
        if let (Some(source), Some(target)) =
            (node_index.get(&edge.source), node_index.get(&edge.target))
        {
            let dx = nodes[*source].x - nodes[*target].x;
            let dy = nodes[*source].y - nodes[*target].y;
            let distance = (dx * dx + dy * dy).sqrt().max(0.000001);
            let mut force = if options.linlog {
                (1.0 + distance).ln()
            } else {
                distance
            } * edge.weight.max(0.0);

            if options.distributed_action {
                force /= mass[*source].max(0.000001);
            }

            let fx = dx / distance * force;
            let fy = dy / distance * force;
            update[*source].0 -= fx;
            update[*source].1 -= fy;
            update[*target].0 += fx;
            update[*target].1 += fy;
        }
    }
}

fn apply_gravity(
    nodes: &[LayoutNode],
    mass: &[f64],
    update: &mut [(f64, f64)],
    options: &LayoutOptions,
) {
    let (center_x, center_y) = centroid(nodes);

    for (index, node) in nodes.iter().enumerate() {
        let dx = node.x - center_x;
        let dy = node.y - center_y;

        if options.strong_gravity {
            update[index].0 -= options.gravity * mass[index] * dx;
            update[index].1 -= options.gravity * mass[index] * dy;
            continue;
        }

        let distance = (dx * dx + dy * dy).sqrt();
        if distance > 0.0 {
            update[index].0 -= options.gravity * mass[index] * dx / distance;
            update[index].1 -= options.gravity * mass[index] * dy / distance;
        }
    }
}

fn centroid(nodes: &[LayoutNode]) -> (f64, f64) {
    let mut x = 0.0;
    let mut y = 0.0;
    for node in nodes {
        x += node.x;
        y += node.y;
    }

    (x / nodes.len() as f64, y / nodes.len() as f64)
}

fn swing_and_traction(
    mass: &[f64],
    update: &[(f64, f64)],
    previous_update: &[(f64, f64)],
) -> (f64, f64) {
    let mut swing = 0.0;
    let mut traction = 0.0;

    for index in 0..mass.len() {
        swing += mass[index] * vector_len(vector_sub(update[index], previous_update[index]));
        traction +=
            0.5 * mass[index] * vector_len(vector_add(update[index], previous_update[index]));
    }

    (swing.max(0.000001), traction.max(0.000001))
}

fn estimate_factor(
    node_count: usize,
    swing: f64,
    traction: f64,
    speed: f64,
    mut speed_efficiency: f64,
    jitter_tolerance: f64,
) -> (f64, f64) {
    let opt_jitter = 0.05 * (node_count as f64).sqrt();
    let min_jitter = opt_jitter.sqrt();
    let max_jitter: f64 = 10.0;
    let min_speed_efficiency = 0.05;
    let other = max_jitter.min(opt_jitter * traction / (node_count * node_count) as f64);
    let mut jitter = jitter_tolerance * min_jitter.max(other);

    if swing / traction > 2.0 {
        if speed_efficiency > min_speed_efficiency {
            speed_efficiency *= 0.5;
        }
        jitter = jitter.max(jitter_tolerance);
    }

    let target_speed = jitter * speed_efficiency * traction / swing;

    if swing > jitter * traction {
        if speed_efficiency > min_speed_efficiency {
            speed_efficiency *= 0.7;
        }
    } else if speed < 1000.0 {
        speed_efficiency *= 1.3;
    }

    let max_rise = 0.5;
    let next_speed = speed + (target_speed - speed).min(max_rise * speed);
    (next_speed.max(0.000001), speed_efficiency)
}

fn vector_len(vector: (f64, f64)) -> f64 {
    (vector.0 * vector.0 + vector.1 * vector.1).sqrt()
}

fn vector_add(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 + b.0, a.1 + b.1)
}

fn vector_sub(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 - b.0, a.1 - b.1)
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_unit(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        value ^= value >> 31;
        (value as f64) / (u64::MAX as f64)
    }
}
