use crate::LayoutOptions;

use super::LayoutNode;

pub(crate) fn layout(nodes: &mut [LayoutNode], options: &LayoutOptions) {
    match nodes.len() {
        0 => {}
        1 => {
            nodes[0].x = options.center_x;
            nodes[0].y = options.center_y;
        }
        len => {
            for (index, node) in nodes.iter_mut().enumerate() {
                let theta = (index as f64 / len as f64) * std::f64::consts::TAU;
                node.x = options.center_x + options.scale * theta.cos();
                node.y = options.center_y + options.scale * theta.sin();
            }
        }
    }
}
