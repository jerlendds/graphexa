mod bfs;
mod circular;
mod force;
mod forceatlas2;
mod grid;
mod investigation_hierarchy;
mod investigation_hub_rings;
mod investigation_organic;
mod investigation_orthogonal;
mod kamada_kawai;
mod layered;
mod multipartite;
mod planar;
mod radial;
mod random;
mod rescale;
mod shell;
mod spectral;
mod spiral;
mod spring;
mod transform_incremental;
mod transform_locked;

use std::collections::HashMap;

use crate::LayoutOptions;

#[derive(Debug, Clone)]
pub(crate) struct LayoutNode {
    pub(crate) id: String,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) subset: Option<String>,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) has_position: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct LayoutEdge {
    pub(crate) source: String,
    pub(crate) target: String,
    pub(crate) weight: f64,
}

pub(crate) fn apply_layout(
    nodes: &mut [LayoutNode],
    edges: &[LayoutEdge],
    options: &LayoutOptions,
) -> Result<(), String> {
    match options.algorithm.as_str() {
        "bfs" => bfs::layout(nodes, edges, options),
        "circular" => circular::layout(nodes, options),
        "forceatlas2" => forceatlas2::layout(nodes, edges, options),
        "investigation_hierarchy" | "investigation-hierarchy" => {
            investigation_hierarchy::layout(nodes, edges, options)
        }
        "investigation_hub_rings" | "investigation-hub-rings" => {
            investigation_hub_rings::layout(nodes, edges, options)
        }
        "investigation_organic" | "investigation-organic" => {
            investigation_organic::layout(nodes, edges, options)
        }
        "investigation_orthogonal" | "investigation-orthogonal" => {
            investigation_orthogonal::layout(nodes, edges, options)
        }
        "radial" => radial::layout(nodes, edges, options),
        "kamada_kawai" | "kamada-kawai" => kamada_kawai::layout(nodes, edges, options),
        "planar" => return planar::layout(nodes, edges, options),
        "random" => random::layout(nodes, options),
        "rescale" => rescale::layout(nodes, options),
        "shell" => shell::layout(nodes, options),
        "spectral" => spectral::layout(nodes, edges, options),
        "spiral" => spiral::layout(nodes, options),
        "spring" | "fruchterman_reingold" | "fruchterman-reingold" => {
            spring::layout(nodes, edges, options)?
        }
        "multipartite" => return multipartite::layout(nodes, options),
        "transform_incremental" | "transform-incremental" => {
            transform_incremental::layout(nodes, edges, options)
        }
        "transform_locked" | "transform-locked" => transform_locked::layout(nodes, edges, options),
        "force" => force::layout(nodes, edges, options),
        "grid" => grid::layout(nodes, options),
        _ => layered::layout(nodes, edges, options),
    };

    Ok(())
}

pub(crate) fn index_nodes(nodes: &[LayoutNode]) -> HashMap<String, usize> {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.id.clone(), index))
        .collect()
}
