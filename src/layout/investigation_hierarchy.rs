use crate::LayoutOptions;

use super::{LayoutEdge, LayoutNode, layered};

pub(crate) fn layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
    layered::layout(nodes, edges, options);
}
