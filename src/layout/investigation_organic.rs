use crate::LayoutOptions;

use super::{LayoutEdge, LayoutNode, force};

pub(crate) fn layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
    force::layout(nodes, edges, options);
}
