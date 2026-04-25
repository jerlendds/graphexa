use crate::LayoutOptions;

use super::LayoutNode;

pub(crate) fn layout(nodes: &mut [LayoutNode], options: &LayoutOptions) {
    if nodes.is_empty() {
        return;
    }

    let mean_x = nodes.iter().map(|node| node.x).sum::<f64>() / nodes.len() as f64;
    let mean_y = nodes.iter().map(|node| node.y).sum::<f64>() / nodes.len() as f64;
    let mut limit: f64 = 0.0;

    for node in nodes.iter() {
        limit = limit
            .max((node.x - mean_x).abs())
            .max((node.y - mean_y).abs());
    }

    let factor = if limit > 0.0 {
        options.scale / limit
    } else {
        1.0
    };
    for node in nodes {
        node.x = options.center_x + (node.x - mean_x) * factor;
        node.y = options.center_y + (node.y - mean_y) * factor;
    }
}
