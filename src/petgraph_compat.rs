//! petgraph integration (requires `petgraph` feature).
//!
//! Provides conversion from petgraph's `DiGraph` to [`CsrGraph`] and an
//! extension trait for running DMMSY shortest paths directly on petgraph types.
//!
//! # Example
//!
//! ```rust,ignore
//! use petgraph::graph::DiGraph;
//! use dmmsy::CsrGraph;
//! use dmmsy::petgraph_compat::DmmsyExt;
//!
//! let mut g = DiGraph::new();
//! let a = g.add_node("A");
//! let b = g.add_node("B");
//! let c = g.add_node("C");
//! g.add_edge(a, b, 1.0_f64);
//! g.add_edge(b, c, 2.0);
//! g.add_edge(a, c, 5.0);
//!
//! let paths = g.dmmsy_shortest_paths(a);
//! assert_eq!(paths.distance(c.index() as u32), 3.0);
//! ```

#[cfg(feature = "petgraph")]
use petgraph::graph::{DiGraph, NodeIndex};
#[cfg(feature = "petgraph")]
use petgraph::visit::EdgeRef;

#[cfg(feature = "petgraph")]
use crate::dijkstra::ShortestPaths;
#[cfg(feature = "petgraph")]
use crate::graph::CsrGraph;

/// Convert a petgraph `DiGraph` to a `CsrGraph<f64>`.
///
/// Edge weights are converted to `f64` via the `Into<f64>` trait.
/// Node data is ignored (only graph structure matters for shortest paths).
#[cfg(feature = "petgraph")]
impl<N, E> From<&DiGraph<N, E>> for CsrGraph<f64>
where
    E: Into<f64> + Copy,
{
    fn from(graph: &DiGraph<N, E>) -> Self {
        let edges: Vec<_> = graph
            .edge_references()
            .map(|e| {
                (
                    e.source().index() as u32,
                    e.target().index() as u32,
                    (*e.weight()).into(),
                )
            })
            .collect();
        CsrGraph::from_edges(graph.node_count(), &edges)
    }
}

/// Extension trait that adds DMMSY shortest paths to petgraph `DiGraph`.
///
/// This provides a convenient way to use DMMSY with existing petgraph code
/// without manually converting graph types.
#[cfg(feature = "petgraph")]
pub trait DmmsyExt<E> {
    /// Compute single-source shortest paths using the best available algorithm.
    ///
    /// Converts the graph to CSR format and runs
    /// [`shortest_paths`](crate::shortest_paths).
    fn dmmsy_shortest_paths(&self, source: NodeIndex) -> ShortestPaths<f64>;
}

#[cfg(feature = "petgraph")]
impl<N, E: Into<f64> + Copy> DmmsyExt<E> for DiGraph<N, E> {
    fn dmmsy_shortest_paths(&self, source: NodeIndex) -> ShortestPaths<f64> {
        let csr: CsrGraph<f64> = self.into();
        crate::shortest_paths_f64(&csr, source.index() as u32)
    }
}

// When the petgraph feature is not enabled, provide a stub module
// so that `use dmmsy::petgraph_compat` doesn't fail in docs.
#[cfg(not(feature = "petgraph"))]
mod _stub {
    // This module intentionally left empty.
    // Enable the `petgraph` feature for petgraph integration.
}
