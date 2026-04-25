use std::collections::HashSet;

use crate::LayoutOptions;

use super::{LayoutEdge, LayoutNode, index_nodes};

pub(crate) fn layout(
    nodes: &mut [LayoutNode],
    edges: &[LayoutEdge],
    options: &LayoutOptions,
) -> Result<(), String> {
    if nodes.is_empty() {
        return Ok(());
    }

    let indexed_edges = indexed_edges(nodes, edges);
    if !passes_planarity_guards(nodes.len(), &indexed_edges) {
        return Err("G is not planar.".to_owned());
    }

    let adjacency = adjacency(nodes.len(), &indexed_edges);
    let levels = graph_levels(&adjacency);
    let max_level = levels.iter().copied().max().unwrap_or(0);

    for level in 0..=max_level {
        let layer = levels
            .iter()
            .enumerate()
            .filter_map(|(index, candidate_level)| (*candidate_level == level).then_some(index))
            .collect::<Vec<_>>();
        let y = if max_level == 0 {
            0.0
        } else {
            -options.scale + (2.0 * options.scale * level as f64 / max_level as f64)
        };

        for (order, index) in layer.iter().enumerate() {
            let x = if layer.len() <= 1 {
                0.0
            } else {
                -options.scale + (2.0 * options.scale * order as f64 / (layer.len() - 1) as f64)
            };
            nodes[*index].x = options.center_x + x;
            nodes[*index].y = options.center_y + y;
        }
    }

    Ok(())
}

fn indexed_edges(nodes: &[LayoutNode], edges: &[LayoutEdge]) -> Vec<(usize, usize)> {
    let node_index = index_nodes(nodes);
    let mut seen = HashSet::new();

    edges
        .iter()
        .filter_map(|edge| {
            let source = *node_index.get(&edge.source)?;
            let target = *node_index.get(&edge.target)?;
            if source == target {
                return None;
            }
            let key = if source < target {
                (source, target)
            } else {
                (target, source)
            };
            seen.insert(key).then_some(key)
        })
        .collect()
}

fn passes_planarity_guards(node_count: usize, edges: &[(usize, usize)]) -> bool {
    if node_count < 5 {
        return true;
    }

    if edges.len() > 3 * node_count - 6 {
        return false;
    }

    if is_bipartite(node_count, edges) && node_count >= 3 && edges.len() > 2 * node_count - 4 {
        return false;
    }

    !contains_k5(node_count, edges) && !contains_k33(node_count, edges)
}

fn adjacency(node_count: usize, edges: &[(usize, usize)]) -> Vec<Vec<usize>> {
    let mut adjacency = vec![Vec::new(); node_count];
    for (source, target) in edges {
        adjacency[*source].push(*target);
        adjacency[*target].push(*source);
    }
    adjacency
}

fn graph_levels(adjacency: &[Vec<usize>]) -> Vec<usize> {
    let mut levels = vec![usize::MAX; adjacency.len()];
    let mut queue = std::collections::VecDeque::new();

    for start in 0..adjacency.len() {
        if levels[start] != usize::MAX {
            continue;
        }

        levels[start] = 0;
        queue.push_back(start);

        while let Some(node) = queue.pop_front() {
            for neighbor in &adjacency[node] {
                if levels[*neighbor] == usize::MAX {
                    levels[*neighbor] = levels[node] + 1;
                    queue.push_back(*neighbor);
                }
            }
        }
    }

    levels
}

fn is_bipartite(node_count: usize, edges: &[(usize, usize)]) -> bool {
    let adjacency = adjacency(node_count, edges);
    let mut color = vec![None; node_count];
    let mut queue = std::collections::VecDeque::new();

    for start in 0..node_count {
        if color[start].is_some() {
            continue;
        }
        color[start] = Some(false);
        queue.push_back(start);

        while let Some(node) = queue.pop_front() {
            let next_color = !color[node].unwrap_or(false);
            for neighbor in &adjacency[node] {
                match color[*neighbor] {
                    Some(existing) if existing != next_color => return false,
                    Some(_) => {}
                    None => {
                        color[*neighbor] = Some(next_color);
                        queue.push_back(*neighbor);
                    }
                }
            }
        }
    }

    true
}

fn contains_k5(node_count: usize, edges: &[(usize, usize)]) -> bool {
    if node_count < 5 {
        return false;
    }

    let edge_set = edge_set(edges);
    for a in 0..node_count - 4 {
        for b in a + 1..node_count - 3 {
            for c in b + 1..node_count - 2 {
                for d in c + 1..node_count - 1 {
                    for e in d + 1..node_count {
                        let group = [a, b, c, d, e];
                        if complete_subgraph(&group, &edge_set) {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

fn contains_k33(node_count: usize, edges: &[(usize, usize)]) -> bool {
    if node_count < 6 {
        return false;
    }

    let edge_set = edge_set(edges);
    for a in 0..node_count - 5 {
        for b in a + 1..node_count - 4 {
            for c in b + 1..node_count - 3 {
                let left = [a, b, c];
                for d in 0..node_count - 2 {
                    if left.contains(&d) {
                        continue;
                    }
                    for e in d + 1..node_count - 1 {
                        if left.contains(&e) {
                            continue;
                        }
                        for f in e + 1..node_count {
                            if left.contains(&f) {
                                continue;
                            }
                            let right = [d, e, f];
                            if complete_bipartite_subgraph(&left, &right, &edge_set) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

fn complete_subgraph(group: &[usize], edge_set: &HashSet<(usize, usize)>) -> bool {
    for (offset, source) in group.iter().enumerate() {
        for target in &group[offset + 1..] {
            if !edge_set.contains(&ordered_edge(*source, *target)) {
                return false;
            }
        }
    }

    true
}

fn complete_bipartite_subgraph(
    left: &[usize],
    right: &[usize],
    edge_set: &HashSet<(usize, usize)>,
) -> bool {
    left.iter().all(|source| {
        right
            .iter()
            .all(|target| edge_set.contains(&ordered_edge(*source, *target)))
    })
}

fn edge_set(edges: &[(usize, usize)]) -> HashSet<(usize, usize)> {
    edges
        .iter()
        .map(|(source, target)| ordered_edge(*source, *target))
        .collect()
}

fn ordered_edge(source: usize, target: usize) -> (usize, usize) {
    if source < target {
        (source, target)
    } else {
        (target, source)
    }
}
