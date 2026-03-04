//! Basic usage of the dmmsy crate.
//!
//! Demonstrates building a graph and computing shortest paths.

use dmmsy::{CsrGraph, shortest_paths};

fn main() {
    // Build a simple directed weighted graph:
    //
    //   0 --1.0--> 1 --2.0--> 2 --1.0--> 3
    //   |                      ^
    //   +--------5.0-----------+
    //
    let graph = CsrGraph::from_edges(4, &[
        (0, 1, 1.0_f64),
        (1, 2, 2.0),
        (0, 2, 5.0),
        (2, 3, 1.0),
    ]);

    println!("Graph: {} nodes, {} edges", graph.num_nodes(), graph.num_edges());
    println!("Density: {:.6}", graph.density());

    // Compute shortest paths from node 0
    let result = shortest_paths(&graph, 0);

    for node in 0..graph.num_nodes() {
        let dist = result.distances[node];
        let reachable = result.is_reachable(node as u32);
        println!(
            "  Node {}: distance = {:.1}, reachable = {}",
            node, dist, reachable
        );
    }

    // Reconstruct path from 0 to 3
    if let Some(path) = result.path_to(3) {
        let path_str: Vec<String> = path.iter().map(|n| n.to_string()).collect();
        println!("\nShortest path 0 → 3: {}", path_str.join(" → "));
        println!("Total distance: {:.1}", result.distances[3]);
    }
}
