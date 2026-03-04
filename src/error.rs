//! Error types for the dmmsy crate.
//!
//! Provides [`DmmsyError`] for fallible API entry points like
//! [`try_shortest_paths`](crate::try_shortest_paths) and
//! [`try_shortest_paths_f64`](crate::try_shortest_paths_f64).

use crate::graph::CsrGraph;
use crate::weight::Weight;
use core::fmt;

/// Errors that can occur when running shortest path algorithms.
///
/// The `try_*` variants of the public API functions return this error type
/// instead of panicking. The original functions (`shortest_paths`,
/// `shortest_paths_f64`, `dijkstra`, `dmmsy`) still panic for backwards
/// compatibility — use the `try_*` wrappers if you need graceful error handling.
///
/// # Example
///
/// ```
/// use dmmsy::{CsrGraph, DmmsyError, try_shortest_paths};
///
/// let graph = CsrGraph::from_edges(3, &[(0, 1, 1.0_f64)]);
/// match try_shortest_paths(&graph, 99) {
///     Err(DmmsyError::SourceOutOfBounds { source, num_nodes }) => {
///         assert_eq!(source, 99);
///         assert_eq!(num_nodes, 3);
///     }
///     _ => panic!("expected SourceOutOfBounds"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DmmsyError {
    /// The source node index is out of bounds for the given graph.
    SourceOutOfBounds {
        /// The source node that was requested.
        source: u32,
        /// The number of nodes in the graph.
        num_nodes: usize,
    },

    /// The graph has zero nodes; shortest paths are undefined.
    EmptyGraph,

    /// An edge references a node index that is out of bounds.
    EdgeOutOfBounds {
        /// The offending node index from the edge.
        node: u32,
        /// The number of nodes in the graph.
        num_nodes: usize,
    },
}

impl fmt::Display for DmmsyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DmmsyError::SourceOutOfBounds { source, num_nodes } => {
                write!(
                    f,
                    "source node {} is out of bounds for graph with {} nodes",
                    source, num_nodes
                )
            }
            DmmsyError::EmptyGraph => {
                write!(f, "cannot compute shortest paths on an empty graph (0 nodes)")
            }
            DmmsyError::EdgeOutOfBounds { node, num_nodes } => {
                write!(
                    f,
                    "edge references node {} which is out of bounds for graph with {} nodes",
                    node, num_nodes
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DmmsyError {}

/// Validate that a source node is within bounds for the given graph.
///
/// Returns `Ok(())` if valid, or an appropriate `DmmsyError` if not.
pub(crate) fn validate_source<W: Weight>(
    graph: &CsrGraph<W>,
    source: u32,
) -> Result<(), DmmsyError> {
    let n = graph.num_nodes();
    if n == 0 {
        return Err(DmmsyError::EmptyGraph);
    }
    if (source as usize) >= n {
        return Err(DmmsyError::SourceOutOfBounds {
            source,
            num_nodes: n,
        });
    }
    Ok(())
}

/// Validate edges for a graph construction, returning an error instead of panicking.
///
/// Checks that all source and target node indices are within bounds.
pub(crate) fn validate_edges<W: Weight>(
    num_nodes: usize,
    edges: &[(u32, u32, W)],
) -> Result<(), DmmsyError> {
    for &(src, tgt, _) in edges {
        if (src as usize) >= num_nodes {
            return Err(DmmsyError::EdgeOutOfBounds {
                node: src,
                num_nodes,
            });
        }
        if (tgt as usize) >= num_nodes {
            return Err(DmmsyError::EdgeOutOfBounds {
                node: tgt,
                num_nodes,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_source_out_of_bounds() {
        let err = DmmsyError::SourceOutOfBounds {
            source: 10,
            num_nodes: 5,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("10"));
        assert!(msg.contains("5"));
    }

    #[test]
    fn display_empty_graph() {
        let err = DmmsyError::EmptyGraph;
        let msg = format!("{}", err);
        assert!(msg.contains("empty graph"));
    }

    #[test]
    fn display_edge_out_of_bounds() {
        let err = DmmsyError::EdgeOutOfBounds {
            node: 99,
            num_nodes: 10,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("99"));
        assert!(msg.contains("10"));
    }

    #[test]
    fn validate_source_ok() {
        let g = CsrGraph::from_edges(3, &[(0u32, 1, 1.0_f64)]);
        assert!(validate_source(&g, 0).is_ok());
        assert!(validate_source(&g, 2).is_ok());
    }

    #[test]
    fn validate_source_out_of_bounds() {
        let g = CsrGraph::from_edges(3, &[(0u32, 1, 1.0_f64)]);
        assert!(matches!(
            validate_source(&g, 5),
            Err(DmmsyError::SourceOutOfBounds { source: 5, num_nodes: 3 })
        ));
    }

    #[test]
    fn validate_source_empty_graph() {
        let g: CsrGraph<f64> = CsrGraph::from_edges(0, &[]);
        assert!(matches!(validate_source(&g, 0), Err(DmmsyError::EmptyGraph)));
    }

    #[test]
    fn validate_edges_ok() {
        assert!(validate_edges(3, &[(0u32, 1, 1.0_f64), (1, 2, 2.0)]).is_ok());
    }

    #[test]
    fn validate_edges_src_out_of_bounds() {
        assert!(matches!(
            validate_edges(3, &[(5u32, 1, 1.0_f64)]),
            Err(DmmsyError::EdgeOutOfBounds { node: 5, num_nodes: 3 })
        ));
    }

    #[test]
    fn validate_edges_tgt_out_of_bounds() {
        assert!(matches!(
            validate_edges(3, &[(0u32, 10, 1.0_f64)]),
            Err(DmmsyError::EdgeOutOfBounds { node: 10, num_nodes: 3 })
        ));
    }
}
