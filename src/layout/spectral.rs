use crate::LayoutOptions;

use super::{LayoutEdge, LayoutNode, index_nodes};

pub(crate) fn layout(nodes: &mut [LayoutNode], edges: &[LayoutEdge], options: &LayoutOptions) {
    match nodes.len() {
        0 => return,
        1 => {
            nodes[0].x = options.center_x;
            nodes[0].y = options.center_y;
            return;
        }
        2 => {
            nodes[0].x = options.center_x - options.scale;
            nodes[0].y = options.center_y;
            nodes[1].x = options.center_x + options.scale;
            nodes[1].y = options.center_y;
            return;
        }
        _ => {}
    }

    let laplacian = laplacian(nodes, edges);
    let (_, eigenvectors) = jacobi(laplacian, 80);
    for (index, node) in nodes.iter_mut().enumerate() {
        node.x = eigenvectors[index][1];
        node.y = eigenvectors[index].get(2).copied().unwrap_or(0.0);
    }
    super::rescale::layout(nodes, options);
}

fn laplacian(nodes: &[LayoutNode], edges: &[LayoutEdge]) -> Vec<Vec<f64>> {
    let node_index = index_nodes(nodes);
    let mut matrix = vec![vec![0.0; nodes.len()]; nodes.len()];
    for edge in edges {
        if let (Some(source), Some(target)) =
            (node_index.get(&edge.source), node_index.get(&edge.target))
        {
            let weight = edge.weight.max(0.0);
            matrix[*source][*source] += weight;
            matrix[*target][*target] += weight;
            matrix[*source][*target] -= weight;
            matrix[*target][*source] -= weight;
        }
    }
    matrix
}

fn jacobi(mut matrix: Vec<Vec<f64>>, iterations: usize) -> (Vec<f64>, Vec<Vec<f64>>) {
    let size = matrix.len();
    let mut vectors = vec![vec![0.0; size]; size];
    for (index, row) in vectors.iter_mut().enumerate() {
        row[index] = 1.0;
    }

    for _ in 0..iterations {
        let (p, q, max_value) = max_off_diagonal(&matrix);
        if max_value < 1e-10 {
            break;
        }

        let angle = 0.5 * (2.0 * matrix[p][q]).atan2(matrix[q][q] - matrix[p][p]);
        let cosine = angle.cos();
        let sine = angle.sin();

        for row in 0..size {
            let mp = matrix[row][p];
            let mq = matrix[row][q];
            matrix[row][p] = cosine * mp - sine * mq;
            matrix[row][q] = sine * mp + cosine * mq;
        }
        for col in 0..size {
            let mp = matrix[p][col];
            let mq = matrix[q][col];
            matrix[p][col] = cosine * mp - sine * mq;
            matrix[q][col] = sine * mp + cosine * mq;
        }
        for row in vectors.iter_mut() {
            let vp = row[p];
            let vq = row[q];
            row[p] = cosine * vp - sine * vq;
            row[q] = sine * vp + cosine * vq;
        }
    }

    let mut order = (0..size).collect::<Vec<_>>();
    order.sort_by(|left, right| matrix[*left][*left].total_cmp(&matrix[*right][*right]));
    let values = order
        .iter()
        .map(|index| matrix[*index][*index])
        .collect::<Vec<_>>();
    let ordered_vectors = vectors
        .into_iter()
        .map(|row| order.iter().map(|index| row[*index]).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    (values, ordered_vectors)
}

fn max_off_diagonal(matrix: &[Vec<f64>]) -> (usize, usize, f64) {
    let mut best = (0, 1, 0.0);
    for row in 0..matrix.len() {
        for col in row + 1..matrix.len() {
            let value = matrix[row][col].abs();
            if value > best.2 {
                best = (row, col, value);
            }
        }
    }
    best
}
