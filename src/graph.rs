//! Compressed Sparse Row (CSR) graph representation.
//!
//! [`CsrGraph`] stores graphs in a cache-friendly layout optimized for
//! sequential edge traversal — the dominant access pattern in shortest
//! path algorithms.
//!
//! Adapted from danalec's `CSRGraph` in DMMSY-SSSP-rs with added
//! generic weights, builder API, and petgraph integration support.

use crate::error::DmmsyError;
use crate::weight::Weight;

/// A single directed edge (target, weight).
#[derive(Clone, Copy, Debug)]
pub struct Edge<W: Weight = f64> {
    /// Target node ID.
    pub v: u32,
    /// Edge weight.
    pub w: W,
}

/// A directed weighted graph in Compressed Sparse Row format.
///
/// Generic over the weight type `W` (defaults to `f64`).
/// Optimized for cache-friendly sequential neighbor traversal.
///
/// # Example
///
/// ```
/// use dmmsy::CsrGraph;
///
/// let graph = CsrGraph::from_edges(4, &[
///     (0, 1, 1.0), (0, 2, 5.0), (1, 2, 2.0), (2, 3, 1.0),
/// ]);
/// assert_eq!(graph.num_nodes(), 4);
/// assert_eq!(graph.num_edges(), 4);
/// ```
#[derive(Debug, Clone)]
pub struct CsrGraph<W: Weight = f64> {
    /// Number of nodes.
    num_nodes: usize,
    /// `offset[i]` = index into `edges` where node i's edges start.
    /// Length: num_nodes + 1.
    pub(crate) offset: Vec<u32>,
    /// Packed edge array (target + weight), sorted by source node.
    pub(crate) edges: Vec<Edge<W>>,
    /// Mean edge weight. Used by DMMSY to compute the distance bound.
    pub(crate) mean_weight: f64,
}

impl<W: Weight> CsrGraph<W> {
    /// Build a CSR graph from an edge list.
    ///
    /// Edges are `(source, target, weight)` tuples. The graph will have
    /// exactly `num_nodes` nodes (0..num_nodes).
    ///
    /// # Panics
    ///
    /// Panics if any edge references a node >= `num_nodes`.
    pub fn from_edges(num_nodes: usize, edges: &[(u32, u32, W)]) -> Self {
        for &(src, tgt, _) in edges {
            assert!(
                (src as usize) < num_nodes,
                "source node {} >= num_nodes {}",
                src, num_nodes
            );
            assert!(
                (tgt as usize) < num_nodes,
                "target node {} >= num_nodes {}",
                tgt, num_nodes
            );
        }

        // Count edges per source node
        let mut counts = vec![0u32; num_nodes];
        for &(src, _, _) in edges {
            counts[src as usize] += 1;
        }

        // Build offset array (prefix sum)
        let mut offset = vec![0u32; num_nodes + 1];
        for i in 0..num_nodes {
            offset[i + 1] = offset[i] + counts[i];
        }

        // Place edges into CSR arrays
        let num_edges = edges.len();
        let mut csr_edges = vec![Edge { v: 0, w: W::ZERO }; num_edges];
        let mut cursor = offset[..num_nodes].to_vec();

        let mut weight_sum = 0.0f64;
        for &(src, tgt, w) in edges {
            let idx = cursor[src as usize] as usize;
            csr_edges[idx] = Edge { v: tgt, w };
            cursor[src as usize] += 1;
            weight_sum += w.to_f64();
        }

        let mean_weight = if num_edges > 0 {
            weight_sum / num_edges as f64
        } else {
            0.0
        };

        CsrGraph {
            num_nodes,
            offset,
            edges: csr_edges,
            mean_weight,
        }
    }

    /// Build a CSR graph from an edge list, returning an error on invalid input.
    ///
    /// Like [`from_edges`](Self::from_edges), but returns
    /// [`DmmsyError::EdgeOutOfBounds`] instead of panicking when an edge
    /// references a node `>= num_nodes`.
    ///
    /// # Example
    ///
    /// ```
    /// use dmmsy::CsrGraph;
    ///
    /// // Valid edges succeed
    /// let graph = CsrGraph::try_from_edges(3, &[(0, 1, 1.0_f64), (1, 2, 2.0)]).unwrap();
    /// assert_eq!(graph.num_nodes(), 3);
    ///
    /// // Out-of-bounds edge returns an error
    /// assert!(CsrGraph::try_from_edges(3, &[(0, 99, 1.0_f64)]).is_err());
    /// ```
    pub fn try_from_edges(num_nodes: usize, edges: &[(u32, u32, W)]) -> Result<Self, DmmsyError> {
        crate::error::validate_edges(num_nodes, edges)?;
        Ok(Self::from_edges(num_nodes, edges))
    }

    /// Iterate over the neighbors of `node`, yielding `(target, weight)` pairs.
    #[inline]
    pub fn neighbors(&self, node: u32) -> impl Iterator<Item = (u32, W)> + '_ {
        let start = self.offset[node as usize] as usize;
        let end = self.offset[node as usize + 1] as usize;
        self.edges[start..end].iter().map(|e| (e.v, e.w))
    }

    /// Number of nodes.
    #[inline]
    pub fn num_nodes(&self) -> usize {
        self.num_nodes
    }

    /// Number of directed edges.
    #[inline]
    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }

    /// Graph density: `num_edges / num_nodes²`.
    #[inline]
    pub fn density(&self) -> f64 {
        if self.num_nodes == 0 {
            return 0.0;
        }
        self.num_edges() as f64 / (self.num_nodes as f64 * self.num_nodes as f64)
    }

    /// Out-degree of a node.
    #[inline]
    pub fn degree(&self, node: u32) -> usize {
        let start = self.offset[node as usize] as usize;
        let end = self.offset[node as usize + 1] as usize;
        end - start
    }

    /// Mean edge weight (used internally by DMMSY for bound computation).
    #[inline]
    pub fn mean_weight(&self) -> f64 {
        self.mean_weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() {
        let g: CsrGraph<f64> = CsrGraph::from_edges(0, &[]);
        assert_eq!(g.num_nodes(), 0);
        assert_eq!(g.num_edges(), 0);
    }

    #[test]
    fn simple_graph() {
        let g = CsrGraph::from_edges(4, &[
            (0, 1, 1.0_f64), (0, 2, 5.0), (1, 2, 2.0), (2, 3, 1.0),
        ]);
        assert_eq!(g.num_nodes(), 4);
        assert_eq!(g.num_edges(), 4);

        let n0: Vec<_> = g.neighbors(0).collect();
        assert_eq!(n0.len(), 2);
        assert!(n0.contains(&(1, 1.0)));
        assert!(n0.contains(&(2, 5.0)));
    }

    #[test]
    fn mean_weight_computed() {
        let g = CsrGraph::from_edges(3, &[
            (0, 1, 10.0_f64), (1, 2, 20.0),
        ]);
        assert!((g.mean_weight() - 15.0).abs() < 1e-10);
    }

    #[test]
    fn integer_weights() {
        let g = CsrGraph::from_edges(3, &[(0, 1, 10u32), (1, 2, 20u32)]);
        let n: Vec<_> = g.neighbors(0).collect();
        assert_eq!(n, vec![(1, 10u32)]);
    }

    #[test]
    #[should_panic(expected = "source node")]
    fn out_of_bounds_source() {
        let _g = CsrGraph::from_edges(2, &[(5, 0, 1.0_f64)]);
    }
}
