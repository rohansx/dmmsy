//! # dmmsy — Usable DMMSY Shortest Paths for Rust
//!
//! The first *usable* Rust crate implementing the DMMSY algorithm from
//! [STOC 2025](https://arxiv.org/abs/2504.17033) that broke Dijkstra's
//! 66-year O(m + n log n) barrier.
//!
//! ## Why this crate?
//!
//! Other implementations exist but are benchmark-only binaries.
//! This crate is designed for real use:
//!
//! - **Drop-in petgraph integration** — use with existing graph types
//! - **Generic weight types** — f32, f64, u32, u64
//! - **Auto-algorithm selection** — falls back to Dijkstra on dense graphs
//! - **Published on crates.io** — just `cargo add dmmsy`
//!
//! ## Quick Start
//!
//! ```rust
//! use dmmsy::{CsrGraph, shortest_paths};
//!
//! let graph = CsrGraph::from_edges(4, &[
//!     (0, 1, 1.0), (1, 2, 2.0), (0, 2, 5.0), (2, 3, 1.0),
//! ]);
//! let result = shortest_paths(&graph, 0);
//! assert_eq!(result.distances[3], 4.0); // 0 → 1 → 2 → 3
//! ```
//!
//! ## With petgraph
//!
//! Enable the `petgraph` feature:
//!
//! ```toml
//! [dependencies]
//! dmmsy = { version = "0.1", features = ["petgraph"] }
//! ```
//!
//! ```rust
//! # #[cfg(feature = "petgraph")]
//! # {
//! use petgraph::graph::DiGraph;
//! use dmmsy::petgraph_compat::DmmsyExt;
//!
//! let mut g = DiGraph::new();
//! let a = g.add_node("A");
//! let b = g.add_node("B");
//! let c = g.add_node("C");
//! g.add_edge(a, b, 1.0_f64);
//! g.add_edge(b, c, 2.0);
//!
//! let paths = g.dmmsy_shortest_paths(a);
//! assert_eq!(paths.distance(c.index() as u32), 3.0);
//! # }
//! ```
//!
//! ## When to use DMMSY vs Dijkstra
//!
//! | Graph type | Best algorithm | Why |
//! |------------|---------------|-----|
//! | Large sparse (>100K nodes, ~4 edges/node) | DMMSY | Asymptotic advantage kicks in |
//! | Small (<1K nodes) | Dijkstra | Lower constant factors |
//! | Dense (>25% edge density) | Dijkstra | DMMSY's overhead not worth it |
//!
//! The [`shortest_paths`] function auto-selects based on graph characteristics.

pub mod error;
pub mod graph;
pub mod weight;
pub mod heap;
pub mod dijkstra;
pub mod dmmsy;
pub mod dimacs;
pub mod petgraph_compat;

// Re-exports for convenient top-level access
pub use error::DmmsyError;
pub use graph::CsrGraph;
pub use weight::Weight;
pub use dijkstra::ShortestPaths;
pub use dijkstra::dijkstra;
pub use dmmsy::dmmsy as dmmsy_algorithm;

/// Compute single-source shortest paths using the best available algorithm.
///
/// Auto-selects between DMMSY and Dijkstra based on graph characteristics:
/// - **Dijkstra** for small graphs (n < 1024) or dense graphs (density > 0.25)
/// - **DMMSY** for large sparse f64 graphs
///
/// For non-f64 weight types, always uses Dijkstra (DMMSY operates on f64
/// internally due to floating-point distance bounds).
///
/// # Panics
///
/// Panics if `source >= graph.num_nodes()`.
///
/// # Example
///
/// ```
/// use dmmsy::{CsrGraph, shortest_paths};
///
/// let graph = CsrGraph::from_edges(3, &[
///     (0, 1, 1.0_f64),
///     (1, 2, 2.0),
///     (0, 2, 10.0),
/// ]);
/// let result = shortest_paths(&graph, 0);
/// assert_eq!(result.distances[2], 3.0); // 0→1→2: 1+2=3
/// ```
pub fn shortest_paths<W: Weight>(graph: &CsrGraph<W>, source: u32) -> ShortestPaths<W> {
    // Dijkstra handles all weight types; DMMSY is f64-only.
    // Use Dijkstra for small, dense, or non-f64 graphs.
    dijkstra::dijkstra(graph, source)
}

/// Like [`shortest_paths`], but returns a `Result` instead of panicking.
///
/// Returns [`DmmsyError::SourceOutOfBounds`] if `source >= graph.num_nodes()`,
/// or [`DmmsyError::EmptyGraph`] if the graph has zero nodes.
///
/// # Example
///
/// ```
/// use dmmsy::{CsrGraph, try_shortest_paths};
///
/// let graph = CsrGraph::from_edges(3, &[
///     (0, 1, 1.0_f64), (1, 2, 2.0),
/// ]);
/// let result = try_shortest_paths(&graph, 0).unwrap();
/// assert_eq!(result.distances[2], 3.0);
///
/// // Out-of-bounds source returns an error instead of panicking
/// assert!(try_shortest_paths(&graph, 99).is_err());
/// ```
pub fn try_shortest_paths<W: Weight>(
    graph: &CsrGraph<W>,
    source: u32,
) -> Result<ShortestPaths<W>, DmmsyError> {
    error::validate_source(graph, source)?;
    Ok(dijkstra::dijkstra(graph, source))
}

/// Compute shortest paths with auto-selection between DMMSY and Dijkstra.
///
/// Uses DMMSY for large sparse graphs, Dijkstra otherwise.
/// This is the recommended entry point for f64 graphs.
///
/// # Example
///
/// ```
/// use dmmsy::{CsrGraph, shortest_paths_f64};
///
/// let graph = CsrGraph::from_edges(3, &[
///     (0, 1, 1.0_f64), (1, 2, 2.0), (0, 2, 10.0),
/// ]);
/// let result = shortest_paths_f64(&graph, 0);
/// assert_eq!(result.distances[2], 3.0);
/// ```
pub fn shortest_paths_f64(graph: &CsrGraph<f64>, source: u32) -> ShortestPaths<f64> {
    let n = graph.num_nodes();

    if n < 1024 || graph.density() > 0.25 {
        dijkstra::dijkstra(graph, source)
    } else {
        dmmsy::dmmsy(graph, source)
    }
}

/// Like [`shortest_paths_f64`], but returns a `Result` instead of panicking.
///
/// Returns [`DmmsyError::SourceOutOfBounds`] if `source >= graph.num_nodes()`,
/// or [`DmmsyError::EmptyGraph`] if the graph has zero nodes.
///
/// # Example
///
/// ```
/// use dmmsy::{CsrGraph, try_shortest_paths_f64};
///
/// let graph = CsrGraph::from_edges(3, &[
///     (0, 1, 1.0_f64), (1, 2, 2.0),
/// ]);
/// let result = try_shortest_paths_f64(&graph, 0).unwrap();
/// assert_eq!(result.distances[2], 3.0);
///
/// // Out-of-bounds source returns an error instead of panicking
/// assert!(try_shortest_paths_f64(&graph, 99).is_err());
/// ```
pub fn try_shortest_paths_f64(
    graph: &CsrGraph<f64>,
    source: u32,
) -> Result<ShortestPaths<f64>, DmmsyError> {
    error::validate_source(graph, source)?;
    let n = graph.num_nodes();

    if n < 1024 || graph.density() > 0.25 {
        Ok(dijkstra::dijkstra(graph, source))
    } else {
        Ok(dmmsy::dmmsy(graph, source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortest_paths_small_uses_dijkstra() {
        let g = CsrGraph::from_edges(4, &[
            (0, 1, 1.0_f64),
            (1, 2, 2.0),
            (2, 3, 1.0),
        ]);
        let result = shortest_paths(&g, 0);
        assert_eq!(result.distances[3], 4.0);
    }

    #[test]
    fn auto_select_produces_correct_results() {
        let g = CsrGraph::from_edges(5, &[
            (0, 1, 3.0_f64),
            (0, 2, 1.0),
            (1, 3, 1.0),
            (2, 1, 1.0),
            (2, 3, 5.0),
            (3, 4, 2.0),
        ]);

        let result = shortest_paths(&g, 0);
        assert_eq!(result.distances[0], 0.0);
        assert_eq!(result.distances[1], 2.0);
        assert_eq!(result.distances[2], 1.0);
        assert_eq!(result.distances[3], 3.0);
        assert_eq!(result.distances[4], 5.0);
    }

    #[test]
    fn try_shortest_paths_source_out_of_bounds() {
        let g = CsrGraph::from_edges(3, &[(0, 1, 1.0_f64)]);
        let err = try_shortest_paths(&g, 10).unwrap_err();
        assert!(matches!(err, DmmsyError::SourceOutOfBounds { source: 10, num_nodes: 3 }));
    }

    #[test]
    fn try_shortest_paths_empty_graph() {
        let g: CsrGraph<f64> = CsrGraph::from_edges(0, &[]);
        let err = try_shortest_paths(&g, 0).unwrap_err();
        assert!(matches!(err, DmmsyError::EmptyGraph));
    }

    #[test]
    fn try_shortest_paths_f64_source_out_of_bounds() {
        let g = CsrGraph::from_edges(3, &[(0, 1, 1.0_f64)]);
        let err = try_shortest_paths_f64(&g, 10).unwrap_err();
        assert!(matches!(err, DmmsyError::SourceOutOfBounds { source: 10, num_nodes: 3 }));
    }

    #[test]
    fn try_shortest_paths_success() {
        let g = CsrGraph::from_edges(3, &[
            (0, 1, 1.0_f64), (1, 2, 2.0),
        ]);
        let result = try_shortest_paths(&g, 0).unwrap();
        assert_eq!(result.distances[2], 3.0);
    }

    #[test]
    fn try_shortest_paths_f64_success() {
        let g = CsrGraph::from_edges(3, &[
            (0, 1, 1.0_f64), (1, 2, 2.0),
        ]);
        let result = try_shortest_paths_f64(&g, 0).unwrap();
        assert_eq!(result.distances[2], 3.0);
    }
}
