use std::collections::{BTreeMap, HashSet};

use serde_json::Value;

use crate::LayoutOptions;

use super::{LayoutNode, index_nodes};

pub(crate) fn layout(nodes: &mut [LayoutNode], options: &LayoutOptions) -> Result<(), String> {
    if nodes.is_empty() {
        return Ok(());
    }
    if !options.align.eq_ignore_ascii_case("vertical")
        && !options.align.eq_ignore_ascii_case("horizontal")
    {
        return Err("align must be either vertical or horizontal.".to_owned());
    }

    let layers = resolve_layers(nodes, options)?;
    let width = layers.len();

    for (layer_index, layer) in layers.iter().enumerate() {
        let height = layer.len();
        for (order, node_index) in layer.iter().enumerate() {
            let mut x = layer_index as f64 - (width.saturating_sub(1)) as f64 / 2.0;
            let mut y = order as f64 - (height.saturating_sub(1)) as f64 / 2.0;
            if options.align.eq_ignore_ascii_case("horizontal") {
                std::mem::swap(&mut x, &mut y);
            }
            nodes[*node_index].x = x;
            nodes[*node_index].y = y;
        }
    }

    super::rescale::layout(nodes, options);
    Ok(())
}

fn resolve_layers(
    nodes: &[LayoutNode],
    options: &LayoutOptions,
) -> Result<Vec<Vec<usize>>, String> {
    if let Some(Value::Object(layer_map)) = &options.subset_key {
        return layers_from_map(nodes, layer_map);
    }

    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, node) in nodes.iter().enumerate() {
        let subset = node.subset.as_ref().ok_or_else(|| {
            let key = options
                .subset_key
                .as_ref()
                .and_then(Value::as_str)
                .unwrap_or("subset");
            format!("all nodes need a subset_key attribute: {key}")
        })?;
        groups.entry(subset.clone()).or_default().push(index);
    }

    Ok(groups.into_values().collect())
}

fn layers_from_map(
    nodes: &[LayoutNode],
    layer_map: &serde_json::Map<String, Value>,
) -> Result<Vec<Vec<usize>>, String> {
    let node_index = index_nodes(nodes);
    let mut seen = HashSet::new();
    let mut ordered = layer_map.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.0.cmp(right.0));

    let mut layers = Vec::new();
    for (_, value) in ordered {
        let ids = value
            .as_array()
            .ok_or_else(|| "subset_key dict values must be arrays of node ids".to_owned())?;
        let mut layer = Vec::new();
        for id in ids {
            let id = id
                .as_str()
                .ok_or_else(|| "subset_key node ids must be strings".to_owned())?;
            let index = *node_index
                .get(id)
                .ok_or_else(|| format!("subset_key references unknown node id: {id}"))?;
            seen.insert(index);
            layer.push(index);
        }
        layers.push(layer);
    }

    if seen.len() != nodes.len() {
        return Err("all nodes must be in one subset of `subset_key` dict".to_owned());
    }

    Ok(layers)
}
