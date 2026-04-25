use crate::LayoutOptions;

use super::{LayoutEdge, LayoutNode, radial};

pub(crate) fn layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
    radial::layout(nodes, edges, options);
}
