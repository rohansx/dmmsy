//! Load and query a DIMACS road network.
//!
//! Download road network data from:
//! http://www.dis.uniroma1.it/challenge9/download.shtml
//!
//! Place the .gr file in the `data/` directory, then run:
//! ```sh
//! cargo run --example road_network --release
//! ```

use dmmsy::{dimacs, shortest_paths, Weight};
use std::time::Instant;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "data/USA-road-d.NY.gr".to_string());

    println!("Loading graph from: {}", path);
    let start = Instant::now();

    let graph = match dimacs::from_file(&path) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Error loading graph: {}", e);
            eprintln!();
            eprintln!("Download DIMACS road network data from:");
            eprintln!("  http://www.dis.uniroma1.it/challenge9/download.shtml");
            eprintln!();
            eprintln!("Usage: cargo run --example road_network --release -- path/to/file.gr");
            std::process::exit(1);
        }
    };

    let load_time = start.elapsed();
    println!("Loaded: {} nodes, {} edges in {:.2?}", graph.num_nodes(), graph.num_edges(), load_time);
    println!("Density: {:.8}", graph.density());

    println!("\nComputing shortest paths from node 0...");
    let start = Instant::now();
    let result = shortest_paths(&graph, 0);
    let compute_time = start.elapsed();

    let reachable = result.distances.iter().filter(|d| !d.is_infinite()).count();
    let max_dist = result.distances.iter().filter(|d| !d.is_infinite()).max().unwrap_or(&0);

    println!("Done in {:.2?}", compute_time);
    println!("Reachable nodes: {} / {}", reachable, graph.num_nodes());
    println!("Max distance: {}", max_dist);

    println!("\nSample distances from node 0:");
    for &node in &[100, 1000, 10000, 50000, 100000] {
        if node < graph.num_nodes() {
            let dist = result.distances[node];
            if dist.is_infinite() {
                println!("  Node {}: unreachable", node);
            } else {
                println!("  Node {}: {}", node, dist);
            }
        }
    }
}
