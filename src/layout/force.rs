use crate::LayoutOptions;

use super::{LayoutEdge, LayoutNode, index_nodes, radial};

pub(crate) fn layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
    radial::layout(nodes, edges, options);

    if nodes.len() < 2 {
        return;
    }

    let node_index = index_nodes(nodes);
    let spring_length = options.node_width + options.spacing_x;
    let repulsion = spring_length * spring_length;
    let step = 0.02;

    for _ in 0..options.iterations {
        let mut delta = vec![(0.0, 0.0); nodes.len()];

        for a in 0..nodes.len() {
            for b in (a + 1)..nodes.len() {
                let dx = nodes[a].x - nodes[b].x;
                let dy = nodes[a].y - nodes[b].y;
                let distance_sq = (dx * dx + dy * dy).max(0.01);
                let distance = distance_sq.sqrt();
                let force = repulsion / distance_sq;
                let fx = (dx / distance) * force;
                let fy = (dy / distance) * force;
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
                let dx = nodes[*target].x - nodes[*source].x;
                let dy = nodes[*target].y - nodes[*source].y;
                let distance = (dx * dx + dy * dy).sqrt().max(0.01);
                let force = (distance - spring_length) * 0.08;
                let fx = (dx / distance) * force;
                let fy = (dy / distance) * force;
                delta[*source].0 += fx;
                delta[*source].1 += fy;
                delta[*target].0 -= fx;
                delta[*target].1 -= fy;
            }
        }

        for (index, node) in nodes.iter_mut().enumerate() {
            node.x += delta[index].0 * step;
            node.y += delta[index].1 * step;
        }
    }
}
