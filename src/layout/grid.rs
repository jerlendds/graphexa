use crate::LayoutOptions;

use super::LayoutNode;

pub(crate) fn layout(nodes: &mut [LayoutNode], options: &LayoutOptions) {
    if nodes.is_empty() {
        return;
    }

    let columns = (nodes.len() as f64).sqrt().ceil() as usize;
    for (index, node) in nodes.iter_mut().enumerate() {
        let row = index / columns;
        let column = index % columns;
        node.x = column as f64 * (node.width + options.spacing_x);
        node.y = row as f64 * (node.height + options.spacing_y);
    }
}
