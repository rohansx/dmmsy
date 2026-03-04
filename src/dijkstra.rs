//! Standard Dijkstra's algorithm with 4-ary heap.
//!
//! Uses the same [`Fast4AryHeap`](crate::heap::Fast4AryHeap) as the DMMSY
//! algorithm, ensuring benchmarks compare algorithms (not data structures).

use crate::graph::CsrGraph;
use crate::heap::Fast4AryHeap;
use crate::weight::Weight;

/// Result of a shortest path computation.
#[derive(Debug, Clone)]
pub struct ShortestPaths<W: Weight> {
    /// Distance from source to each node. `W::INFINITY` if unreachable.
    pub distances: Vec<W>,
    /// Predecessor of each node on the shortest path tree.
    /// `u32::MAX` if no predecessor (source or unreachable).
    pub predecessors: Vec<u32>,
}

impl<W: Weight> ShortestPaths<W> {
    /// Returns the shortest distance to `node`.
    #[inline]
    pub fn distance(&self, node: u32) -> W {
        self.distances[node as usize]
    }

    /// Returns true if `node` is reachable from the source.
    #[inline]
    pub fn is_reachable(&self, node: u32) -> bool {
        !self.distances[node as usize].is_infinite()
    }

    /// Reconstruct the shortest path from source to `target`.
    ///
    /// Returns `None` if `target` is unreachable.
    /// The returned path includes both source and target.
    pub fn path_to(&self, target: u32) -> Option<Vec<u32>> {
        if !self.is_reachable(target) {
            return None;
        }
        let mut path = Vec::new();
        let mut current = target;
        while current != u32::MAX {
            path.push(current);
            current = self.predecessors[current as usize];
        }
        path.reverse();
        Some(path)
    }
}

/// Compute single-source shortest paths using Dijkstra's algorithm.
///
/// Uses a cache-friendly 4-ary heap. Works with any [`Weight`] type.
///
/// # Example
///
/// ```
/// use dmmsy::{CsrGraph, dijkstra};
///
/// let graph = CsrGraph::from_edges(4, &[
///     (0, 1, 1.0), (1, 2, 2.0), (0, 2, 5.0), (2, 3, 1.0),
/// ]);
/// let result = dijkstra(&graph, 0);
/// assert_eq!(result.distances[3], 4.0); // 0→1→2→3: 1+2+1=4
/// ```
pub fn dijkstra<W: Weight>(graph: &CsrGraph<W>, source: u32) -> ShortestPaths<W> {
    let n = graph.num_nodes();
    assert!((source as usize) < n, "source {} >= num_nodes {}", source, n);

    if n == 0 {
        return ShortestPaths { distances: vec![], predecessors: vec![] };
    }

    let n32 = n as u32;
    let mut dist = vec![W::INFINITY; n];
    let mut pred = vec![u32::MAX; n];

    // Use the Fast4AryHeap with f64 priorities (matching DMMSY's heap)
    let mut h = Fast4AryHeap::new(n32);
    let mut sz = 0u32;
    let mut dcnt = 0u32;

    dist[source as usize] = W::ZERO;
    h.push_dec(&mut sz, &mut dcnt, source, 0.0);

    while sz > 0 {
        let mut du_f64 = 0.0;
        let mut u = 0u32;
        h.pop_min(&mut sz, &mut du_f64, &mut u);

        let d_u = dist[u as usize];

        // Skip if we already found a shorter path
        if d_u.to_f64() < du_f64 {
            continue;
        }

        for (v, w) in graph.neighbors(u) {
            let new_dist = d_u + w;
            if new_dist < dist[v as usize] {
                dist[v as usize] = new_dist;
                pred[v as usize] = u;
                h.push_dec(&mut sz, &mut dcnt, v, new_dist.to_f64());
            }
        }
    }

    ShortestPaths {
        distances: dist,
        predecessors: pred,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_path() {
        let g = CsrGraph::from_edges(4, &[
            (0, 1, 1.0_f64), (1, 2, 2.0), (0, 2, 5.0), (2, 3, 1.0),
        ]);
        let result = dijkstra(&g, 0);
        assert_eq!(result.distances[0], 0.0);
        assert_eq!(result.distances[1], 1.0);
        assert_eq!(result.distances[2], 3.0);
        assert_eq!(result.distances[3], 4.0);
    }

    #[test]
    fn unreachable_node() {
        let g = CsrGraph::from_edges(3, &[(0, 1, 1.0_f64)]);
        let result = dijkstra(&g, 0);
        assert!(result.distances[2].is_infinite());
        assert!(!result.is_reachable(2));
    }

    #[test]
    fn path_reconstruction() {
        let g = CsrGraph::from_edges(4, &[
            (0, 1, 1.0_f64), (1, 2, 2.0), (2, 3, 1.0),
        ]);
        let result = dijkstra(&g, 0);
        assert_eq!(result.path_to(3), Some(vec![0, 1, 2, 3]));
        assert_eq!(result.path_to(0), Some(vec![0]));
    }

    #[test]
    fn integer_weights() {
        let g = CsrGraph::from_edges(3, &[
            (0, 1, 10u32), (1, 2, 20u32), (0, 2, 50u32),
        ]);
        let result = dijkstra(&g, 0);
        assert_eq!(result.distances[2], 30);
    }

    #[test]
    fn parallel_edges() {
        let g = CsrGraph::from_edges(2, &[
            (0, 1, 10.0_f64), (0, 1, 3.0),
        ]);
        let result = dijkstra(&g, 0);
        assert_eq!(result.distances[1], 3.0);
    }

    #[test]
    #[should_panic(expected = "source")]
    fn source_out_of_bounds() {
        let g: CsrGraph<f64> = CsrGraph::from_edges(2, &[]);
        dijkstra(&g, 5);
    }
}
