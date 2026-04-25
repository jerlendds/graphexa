use std::collections::HashSet;

use crate::LayoutOptions;

use super::{LayoutNode, index_nodes};

pub(crate) fn layout(nodes: &mut [LayoutNode], options: &LayoutOptions) {
    if nodes.is_empty() {
        return;
    }
    if nodes.len() == 1 {
        nodes[0].x = options.center_x;
        nodes[0].y = options.center_y;
        return;
    }

    let shells = resolve_shells(nodes, options);
    let radius_bump = options.scale / shells.len().max(1) as f64;
    let rotate = options
        .rotate
        .unwrap_or(std::f64::consts::PI / shells.len().max(1) as f64);
    let mut first_theta = rotate;
    let mut radius = if shells.first().is_some_and(|shell| shell.len() == 1) {
        0.0
    } else {
        radius_bump
    };

    for shell in shells {
        for (order, index) in shell.iter().enumerate() {
            let theta =
                (order as f64 / shell.len().max(1) as f64) * std::f64::consts::TAU + first_theta;
            nodes[*index].x = options.center_x + radius * theta.cos();
            nodes[*index].y = options.center_y + radius * theta.sin();
        }
        radius += radius_bump;
        first_theta += rotate;
    }
}

fn resolve_shells(nodes: &[LayoutNode], options: &LayoutOptions) -> Vec<Vec<usize>> {
    let node_index = index_nodes(nodes);
    let mut used = HashSet::new();
    let mut shells = options
        .nlist
        .as_ref()
        .map(|nlist| {
            nlist
                .iter()
                .map(|shell| {
                    shell
                        .iter()
                        .filter_map(|id| node_index.get(id).copied())
                        .inspect(|index| {
                            used.insert(*index);
                        })
                        .collect::<Vec<_>>()
                })
                .filter(|shell| !shell.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec![(0..nodes.len()).collect()]);

    let missing = (0..nodes.len())
        .filter(|index| !used.contains(index))
        .collect::<Vec<_>>();
    if options.nlist.is_some() && !missing.is_empty() {
        shells.push(missing);
    }

    shells
}
