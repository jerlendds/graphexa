use crate::LayoutOptions;

use super::{LayoutEdge, LayoutNode, grid};

pub(crate) fn layout(nodes: &mut [LayoutNode], _edges: &[LayoutEdge], options: &LayoutOptions) {
    grid::layout(nodes, options);
}
