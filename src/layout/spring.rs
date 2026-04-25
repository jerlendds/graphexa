use crate::LayoutOptions;

use super::{LayoutEdge, LayoutNode, index_nodes};

pub(crate) fn layout(
    nodes: &mut [LayoutNode],
    edges: &[LayoutEdge],
    options: &LayoutOptions,
) -> Result<(), String> {
    if !matches!(options.method.as_str(), "auto" | "force" | "energy") {
        return Err("the method must be either auto, force, or energy.".to_owned());
    }
    if nodes.is_empty() {
        return Ok(());
    }
    if nodes.len() == 1 {
        nodes[0].x = options.center_x;
        nodes[0].y = options.center_y;
        return Ok(());
    }

    initialize_positions(nodes, options.seed.unwrap_or(1));
    let node_index = index_nodes(nodes);
    let k = options
        .k
        .unwrap_or_else(|| 1.0 / (nodes.len() as f64).sqrt());
    let mut temperature = 0.1;

    for _ in 0..options.iterations {
        let mut delta = vec![(0.0, 0.0); nodes.len()];

        for a in 0..nodes.len() {
            for b in (a + 1)..nodes.len() {
                let dx = nodes[a].x - nodes[b].x;
                let dy = nodes[a].y - nodes[b].y;
                let distance = (dx * dx + dy * dy).sqrt().max(0.01);
                let force = k * k / distance;
                let fx = dx / distance * force;
                let fy = dy / distance * force;
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
                let dx = nodes[*source].x - nodes[*target].x;
                let dy = nodes[*source].y - nodes[*target].y;
                let distance = (dx * dx + dy * dy).sqrt().max(0.01);
                let force = distance * distance / k * edge.weight.max(0.0);
                let fx = dx / distance * force;
                let fy = dy / distance * force;
                delta[*source].0 -= fx;
                delta[*source].1 -= fy;
                delta[*target].0 += fx;
                delta[*target].1 += fy;
            }
        }

        let mut movement = 0.0;
        for (index, node) in nodes.iter_mut().enumerate() {
            let length = (delta[index].0 * delta[index].0 + delta[index].1 * delta[index].1)
                .sqrt()
                .max(0.01);
            let dx = delta[index].0 / length * length.min(temperature);
            let dy = delta[index].1 / length * length.min(temperature);
            node.x += dx;
            node.y += dy;
            movement += dx.abs() + dy.abs();
        }

        temperature *= 0.95;
        if movement / (nodes.len() as f64) < options.threshold {
            break;
        }
    }

    super::rescale::layout(nodes, options);
    Ok(())
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
        node.x = rng.next_unit();
        node.y = rng.next_unit();
    }
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
