use crate::LayoutOptions;

use super::LayoutNode;

pub(crate) fn layout(nodes: &mut [LayoutNode], options: &LayoutOptions) {
    match nodes.len() {
        0 => return,
        1 => {
            nodes[0].x = options.center_x;
            nodes[0].y = options.center_y;
            return;
        }
        _ => {}
    }

    if options.equidistant {
        let chord = 1.0;
        let step = 0.5;
        let mut theta = options.resolution;
        theta += chord / (step * theta.max(0.000001));
        for node in nodes.iter_mut() {
            let radius = step * theta;
            theta += chord / radius.max(0.000001);
            node.x = radius * theta.cos();
            node.y = radius * theta.sin();
        }
    } else {
        for (index, node) in nodes.iter_mut().enumerate() {
            let distance = index as f64;
            let angle = options.resolution * distance;
            node.x = distance * angle.cos();
            node.y = distance * angle.sin();
        }
    }

    super::rescale::layout(nodes, options);
}
